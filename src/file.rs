use core::cmp::min;
use core::convert::{TryFrom, TryInto};
use core::slice;
use core::str;

extern "C" {
    static ROOT_DIR: [u8; 0];
    static ROOT_DIR_SIZE: usize;
}

fn get_root_dir() -> &'static [u8] {
    unsafe {
        let root_dir_begin = &ROOT_DIR as *const _ as *const u8;
        slice::from_raw_parts(root_dir_begin, ROOT_DIR_SIZE)
    }
}

pub struct RomFile {
    data: &'static [u8],
    offset: usize,
}

#[derive(Debug)]
pub enum OpenError {
    NotFound,
    IsFile,
    IsDir,
}

pub enum SeekFrom {
    Start(u32),
    End(i32),
    Current(i32),
}

#[non_exhaustive]
#[derive(Debug)]
pub enum SeekError {
    NegativeOffset,
}

impl RomFile {
    pub fn raw_open<T>(name: T) -> Result<Self, OpenError>
    where
        T: IntoIterator<Item = u8>,
        T::IntoIter: Clone,
    {
        let root_dir = get_root_dir();
        let mut dir_index = 0;
        let mut name_bytes = name.into_iter();
        let mut name_lookahead = name_bytes.clone();
        while let Some(b'/') = name_lookahead.next() {
            assert!(name_bytes.next().is_some())
        }

        let mut name_begin = name_bytes.clone();
        loop {
            let mut name_size = root_dir[dir_index];
            dir_index += 1;
            if name_size == 0 {
                return Err(OpenError::NotFound);
            }
            let mut this_entry = true;
            while name_size > 0 {
                let char = if let Some(c) = name_bytes.next() {
                    c
                } else {
                    break;
                };
                if char != root_dir[dir_index] {
                    this_entry = false;
                    break;
                }
                dir_index += 1;
                name_size -= 1;
            }
            this_entry = this_entry && name_size == 0;
            dir_index += usize::from(name_size);
            if dir_index % 4 != 0 {
                dir_index += 4 - (dir_index % 4)
            }
            let mut size =
                u32::from_le_bytes(root_dir[dir_index..dir_index + 4].try_into().unwrap()) as usize;
            dir_index += 4;
            let is_dir = size & 0x2000000 == 0x2000000;
            size &= 0x1FFFFFF;
            if this_entry {
                let next_byte = name_bytes.next();
                if next_byte.is_none() && is_dir {
                    return Err(OpenError::IsDir);
                } else if next_byte == Some(b'/') && !is_dir {
                    return Err(OpenError::IsFile);
                } else if next_byte.is_some() && next_byte != Some(b'/') {
                    this_entry = false;
                }
            }
            if this_entry {
                if is_dir {
                    name_begin = name_bytes.clone();
                } else {
                    return Ok(Self {
                        data: &root_dir[dir_index..dir_index + size],
                        offset: 0,
                    });
                }
            } else {
                name_bytes = name_begin.clone();
                dir_index += size;
            }

            if dir_index % 4 != 0 {
                dir_index += 4 - (dir_index % 4)
            }
        }
    }

    pub fn open(name: &str) -> Result<RomFile, OpenError> {
        Self::raw_open(name.bytes())
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> usize {
        let data_end = self.data.len().saturating_sub(self.offset);
        let read_end = min(data_end, buffer.len());
        buffer[..read_end].copy_from_slice(&self.data[self.offset..self.offset + read_end]);
        self.offset += read_end;
        read_end
    }

    pub fn seek(&mut self, pos: SeekFrom) -> Result<u32, SeekError> {
        let new_offset = match pos {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::End(offset) => {
                let size = self.data.len() as i32;
                size.checked_add(offset).and_then(|x| u32::try_from(x).ok())
            }
            SeekFrom::Current(offset) => (self.offset as i32)
                .checked_add(offset)
                .and_then(|x| u32::try_from(x).ok()),
        };
        if let Some(off) = new_offset {
            self.offset = off as usize
        }
        new_offset.ok_or(SeekError::NegativeOffset)
    }

    pub fn as_bytes(&self) -> &'static [u8] {
        &self.data[self.offset..]
    }

    pub fn as_str(&self) -> Result<&'static str, str::Utf8Error> {
        str::from_utf8(self.as_bytes())
    }
}
