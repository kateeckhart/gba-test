#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]
#![warn(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod c_support;
mod debug_print;
mod file;
mod lock;
mod once;
mod util;
mod video;
mod volatile;

use core::panic::PanicInfo;
use core::ptr;

use file::RomFile;

#[panic_handler]
fn panic_handle(panic_info: &PanicInfo) -> ! {
    use core::fmt::Write;
    let _ = writeln!(debug_print::DebugPrinter(0), "{}", panic_info);
    #[allow(clippy::empty_loop)]
    loop {}
}

#[no_mangle]
extern "C" fn main() {
    unsafe {
        ptr::write_volatile(0x4000204 as *mut u16, 0x4017); // Wait state control
    }

    println!("Hii");
    println!("Hi2");

    let file_test = RomFile::open("test.txt").unwrap();

    println!("{}", file_test.as_str().unwrap());

    video::display_bitmap_file(RomFile::open("img/gba_yeen.img").unwrap());

    #[allow(clippy::empty_loop)]
    loop {}
}
