use crate::RomFile;
use crate::file::SeekFrom;
use core::ptr;

fn write_dispcnt(value: u16) {
    unsafe {
        ptr::write_volatile(0x4000000 as *mut u16, value);
    }
}

pub fn display_bitmap_file(mut file: RomFile) {
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
}
