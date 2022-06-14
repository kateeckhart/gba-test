use crate::util::BitIter;
use crate::RomFile;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::ptr;

fn write_dispcnt(value: u16) {
    unsafe {
        ptr::write_volatile(0x4000000 as *mut u16, value);
    }
}

#[derive(Debug)]
enum HuffmanTreeEntry {
    Leaf(u16),
    Internal { left: Box<Self>, right: Box<Self> },
}

impl HuffmanTreeEntry {
    fn from_code_lengths(lengths: &[u8]) -> HuffmanTreeEntry {
        let mut root = HuffmanTreeEntry::Leaf(0);
        let mut length_count = [0; 16];
        for length in lengths {
            length_count[*length as usize] += 1;
        }
        length_count[0] = 0;

        let mut current_code = 0;
        let mut next_code_length = [0; 16];
        for bit in 1..16 {
            current_code = (current_code + length_count[bit - 1]) << 1;
            next_code_length[bit] = current_code;
        }

        for (i, length) in lengths.iter().enumerate() {
            if *length == 0 {
                continue;
            }
            root.replace_entry(next_code_length[*length as usize], *length, i as u16);
            next_code_length[*length as usize] += 1;
        }

        root
    }

    fn next_bit(&self, bit: bool) -> &HuffmanTreeEntry {
        match self {
            HuffmanTreeEntry::Leaf(_) => panic!("Invalid huffman code"),
            HuffmanTreeEntry::Internal { left, right } => {
                if !bit {
                    left
                } else {
                    right
                }
            }
        }
    }

    fn decode_from_bits<T>(&self, bits: &mut BitIter<T>) -> u16
    where
        T: Iterator<Item = u8>,
    {
        let mut decoder = self;

        while let HuffmanTreeEntry::Internal { .. } = decoder {
            decoder = decoder.next_bit(bits.next().unwrap());
        }

        *match decoder {
            HuffmanTreeEntry::Leaf(x) => x,
            _ => unreachable!(),
        }
    }

    fn replace_entry(&mut self, code: u16, in_code_len: u8, value: u16) {
        let mut current_entry = self;
        for code_len in (0..in_code_len).rev() {
            let bit = (code >> code_len) & 1;

            if matches!(current_entry, HuffmanTreeEntry::Leaf(_)) {
                *current_entry = HuffmanTreeEntry::Internal {
                    left: Box::new(HuffmanTreeEntry::Leaf(0)),
                    right: Box::new(HuffmanTreeEntry::Leaf(0)),
                };
            }

            match current_entry {
                HuffmanTreeEntry::Internal { left, right } => {
                    if bit == 0 {
                        current_entry = left;
                    } else {
                        current_entry = right;
                    }
                }
                _ => unreachable!(),
            }
        }

        *current_entry = HuffmanTreeEntry::Leaf(value)
    }
}

struct BlockDecodeState {
    vram_data: u16,
    vram_parity: bool,
    vram_addr: *mut u16,
    last_block: bool,
    literal_code_lengths: Vec<u8>,
    distance_code_lengths: Vec<u8>,
}

fn decode_hufman_block<T>(state: &mut BlockDecodeState, bits: &mut BitIter<T>)
where
    T: Iterator<Item = u8>,
{
    let literal_hufman_tree = HuffmanTreeEntry::from_code_lengths(&state.literal_code_lengths);
    let distance_hufman_tree = HuffmanTreeEntry::from_code_lengths(&state.distance_code_lengths);

    'block_loop: loop {
        let decoded = literal_hufman_tree.decode_from_bits(bits);

        match decoded {
            0..=255 => write_to_vram(state, decoded as u8),
            256 => break 'block_loop,
            257..=264 => lz77_copy(state, bits, &distance_hufman_tree, decoded - 254),
            265..=268 => {
                let mut length = decoded - 265;
                length <<= 1;
                let extra = bits.take_into_u16(1).unwrap();
                length |= extra;
                length += 11;
                lz77_copy(state, bits, &distance_hufman_tree, length);
            }
            269..=272 => {
                let mut length = decoded - 269;
                length <<= 2;
                let extra = bits.take_into_u16(2).unwrap();
                length |= extra;
                length += 19;
                lz77_copy(state, bits, &distance_hufman_tree, length);
            }
            273..=276 => {
                let mut length = decoded - 273;
                length <<= 3;
                let extra = bits.take_into_u16(3).unwrap();
                length |= extra;
                length += 35;
                lz77_copy(state, bits, &distance_hufman_tree, length);
            }
            277..=280 => {
                let mut length = decoded - 277;
                length <<= 4;
                let extra = bits.take_into_u16(4).unwrap();
                length |= extra;
                length += 67;
                lz77_copy(state, bits, &distance_hufman_tree, length);
            }
            281..=284 => {
                let mut length = decoded - 281;
                length <<= 5;
                let extra = bits.take_into_u16(5).unwrap();
                length |= extra;
                length += 131;
                lz77_copy(state, bits, &distance_hufman_tree, length);
            }
            285 => lz77_copy(state, bits, &distance_hufman_tree, 258),
            _ => unreachable!(),
        };
    }
}

fn lz77_copy<T>(
    state: &mut BlockDecodeState,
    bits: &mut BitIter<T>,
    distance_tree: &HuffmanTreeEntry,
    length: u16,
) where
    T: Iterator<Item = u8>,
{
    let distance_sym = distance_tree.decode_from_bits(bits);

    let distance = match distance_sym {
        0..=3 => distance_sym + 1,
        4..=5 => {
            let mut distance = distance_sym - 4;
            distance <<= 1;
            let extra = bits.take_into_u16(1).unwrap();
            distance |= extra;
            distance += 5;
            distance
        }
        6..=7 => {
            let mut distance = distance_sym - 6;
            distance <<= 2;
            let extra = bits.take_into_u16(2).unwrap();
            distance |= extra;
            distance += 9;
            distance
        }
        8..=9 => {
            let mut distance = distance_sym - 8;
            distance <<= 3;
            let extra = bits.take_into_u16(3).unwrap();
            distance |= extra;
            distance += 17;
            distance
        }
        10..=11 => {
            let mut distance = distance_sym - 10;
            distance <<= 4;
            let extra = bits.take_into_u16(4).unwrap();
            distance |= extra;
            distance += 33;
            distance
        }
        12..=13 => {
            let mut distance = distance_sym - 12;
            distance <<= 5;
            let extra = bits.take_into_u16(5).unwrap();
            distance |= extra;
            distance += 65;
            distance
        }
        14..=15 => {
            let mut distance = distance_sym - 14;
            distance <<= 6;
            let extra = bits.take_into_u16(6).unwrap();
            distance |= extra;
            distance += 129;
            distance
        }
        16..=17 => {
            let mut distance = distance_sym - 16;
            distance <<= 7;
            let extra = bits.take_into_u16(7).unwrap();
            distance |= extra;
            distance += 257;
            distance
        }
        18..=19 => {
            let mut distance = distance_sym - 18;
            distance <<= 8;
            let extra = bits.take_into_u16(8).unwrap();
            distance |= extra;
            distance += 513;
            distance
        }
        20..=21 => {
            let mut distance = distance_sym - 20;
            distance <<= 9;
            let extra = bits.take_into_u16(9).unwrap();
            distance |= extra;
            distance += 1025;
            distance
        }
        22..=23 => {
            let mut distance = distance_sym - 22;
            distance <<= 10;
            let extra = bits.take_into_u16(10).unwrap();
            distance |= extra;
            distance += 2049;
            distance
        }
        24..=25 => {
            let mut distance = distance_sym - 24;
            distance <<= 11;
            let extra = bits.take_into_u16(11).unwrap();
            distance |= extra;
            distance += 4097;
            distance
        }
        26..=27 => {
            let mut distance = distance_sym - 26;
            distance <<= 12;
            let extra = bits.take_into_u16(12).unwrap();
            distance |= extra;
            distance += 8193;
            distance
        }
        28..=29 => {
            let mut distance = distance_sym - 28;
            distance <<= 13;
            let extra = bits.take_into_u16(13).unwrap();
            distance |= extra;
            distance += 16385;
            distance
        }
        _ => unreachable!(),
    };

    // Due to the need to write two bytes at a time distance of 1 needs special handeling
    if distance == 1 {
        let rept_byte = if state.vram_parity {
            state.vram_data as u8
        } else {
            unsafe {
                let loc = (state.vram_addr as *const u8).sub(1);
                ptr::read_volatile(loc)
            }
        };

        for _ in 0..length {
            write_to_vram(state, rept_byte);
        }
    } else {
        for _ in 0..length {
            unsafe {
                let real_distance = (distance as usize) - (state.vram_parity as usize);
                let loc = (state.vram_addr as *const u8).sub(real_distance);
                let value = ptr::read_volatile(loc);
                write_to_vram(state, value);
            }
        }
    }
}

fn write_to_vram(state: &mut BlockDecodeState, in_value: u8) {
    let value = in_value as u16;
    if !state.vram_parity {
        state.vram_data = value;
    } else {
        state.vram_data |= value << 8;
        unsafe {
            ptr::write_volatile(state.vram_addr, state.vram_data);
            state.vram_addr = state.vram_addr.offset(1)
        }
    }
    state.vram_parity = !state.vram_parity;
}

static CODE_LENGTH_LENGTH_ORDER: &[u8] = &[
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

fn handle_len_len<T>(bits: &mut BitIter<T>, len_tree: &HuffmanTreeEntry, dest: &mut Vec<u8>)
where
    T: Iterator<Item = u8>,
{
    let len_symbol = len_tree.decode_from_bits(bits) as u8;

    match len_symbol {
        0..=15 => dest.push(len_symbol),
        16 => {
            let last_len = *dest.last().unwrap();

            let repeat_count = bits.take_into_u8(2).unwrap() + 3;

            for _ in 0..repeat_count {
                dest.push(last_len);
            }
        }
        17 => {
            let repeat_count = bits.take_into_u8(3).unwrap() + 3;

            for _ in 0..repeat_count {
                dest.push(0);
            }
        }
        18 => {
            let repeat_count = bits.take_into_u8(7).unwrap() + 11;

            for _ in 0..repeat_count {
                dest.push(0);
            }
        }
        _ => unreachable!(),
    }
}

pub fn display_bitmap_file(file: RomFile) {
    write_dispcnt(0xF03);

    let mut state = BlockDecodeState {
        vram_data: 0,
        vram_parity: false,
        vram_addr: 0x6000000 as *mut u16,
        last_block: false,
        literal_code_lengths: Vec::with_capacity(288),
        distance_code_lengths: Vec::with_capacity(32),
    };

    let mut bits = BitIter::new(file.as_bytes().iter().copied());

    while !state.last_block {
        state.literal_code_lengths.clear();
        state.distance_code_lengths.clear();

        state.last_block = bits.next().unwrap();
        let block_type = bits.take_into_u8(2).unwrap();

        match block_type {
            0 => {
                bits.skip_to_byte_start();
                let length = bits.take_into_usize(16).unwrap();
                let bytes = bits.skip_to_byte_start();
                bytes.nth(1).unwrap();

                for _ in 0..length {
                    let next_byte = bytes.next().unwrap();
                    write_to_vram(&mut state, next_byte);
                }
            }
            1 => {
                for _ in 0..=143 {
                    state.literal_code_lengths.push(8);
                }

                for _ in 144..=255 {
                    state.literal_code_lengths.push(9);
                }

                for _ in 256..=279 {
                    state.literal_code_lengths.push(7);
                }

                for _ in 280..=287 {
                    state.literal_code_lengths.push(8);
                }

                for _ in 0..=31 {
                    state.distance_code_lengths.push(5);
                }

                decode_hufman_block(&mut state, &mut bits);
            }
            2 => {
                let literal_len = bits.take_into_usize(5).unwrap() + 257;
                let distance_len = bits.take_into_usize(5).unwrap() + 1;
                let len_len = bits.take_into_usize(4).unwrap() + 4;

                let mut len_code_len = [0; 19];
                for index in CODE_LENGTH_LENGTH_ORDER.iter().take(len_len) {
                    let len = bits.take_into_u8(3).unwrap();

                    len_code_len[*index as usize] = len;
                }

                let len_tree = HuffmanTreeEntry::from_code_lengths(&len_code_len);

                while state.literal_code_lengths.len() != literal_len {
                    handle_len_len(&mut bits, &len_tree, &mut state.literal_code_lengths);
                }

                while state.distance_code_lengths.len() != distance_len {
                    handle_len_len(&mut bits, &len_tree, &mut state.distance_code_lengths);
                }

                decode_hufman_block(&mut state, &mut bits);
            }
            _ => panic!("Unknown deflate block type"),
        }
    }
}

/*pub fn display_bitmap_file(mut file: RomFile) {
    write_dispcnt(0x80); // Force blank
    file.seek(SeekFrom::Start(0)).unwrap();
    let file_loc = file.as_bytes() as *const _ as *const u8 as usize;
    assert!(file.as_bytes().len() == 0x12C00);

    unsafe {
        ptr::write_volatile(0x40000D4 as *mut usize, file_loc); // File source
        ptr::write_volatile(0x40000D8 as *mut usize, 0x6000000); //Vram dest
        ptr::write_volatile(0x40000DC as *mut u16, 0x4B00); // 75KB
        ptr::write_volatile(0x40000DE as *mut u16, 0x8400); // Start dma
    }

    write_dispcnt(0xF03)
}*/
