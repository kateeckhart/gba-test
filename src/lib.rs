#![feature(default_alloc_error_handler)]
#![feature(allocator_api)]
#![no_std]
#![no_main]
#![warn(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod c_support;
mod debug_print;
mod fast_mem;
mod file;
mod lock;
mod once;
mod util;
mod video;
mod volatile;

use alloc::boxed::Box;
use core::arch::asm;
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

extern "C" {
    fn irq_handle();
}

#[no_mangle]
extern "C" fn main() {
    unsafe {
        ptr::write_volatile(0x4000204 as *mut u16, 0x4017); // Wait state control
        ptr::write_volatile(0x3007FFC as *mut usize, irq_handle as usize); //Init irq handle
        ptr::write_volatile(0x400010E as *mut u16, 0); // Disable timer
        ptr::write_volatile(0x4000200 as *mut u16, 0x40); // Turn on timer 3 irq
        ptr::write_volatile(0x4000208 as *mut u16, 1); // Irq master enable a
        asm!(".align 4",
             "NOP",
             "BX R15",
             ".arm",
             "MRS {x}, CPSR",
             "BIC {x}, {x}, #0xC0",
             "MSR CPSR_c, {x}",
             "ADR {x}, 1f + 5",
             "1: BX {x}",
             ".thumb",
             x = out(reg) _); // Irq master enable b
        ptr::write_volatile(0x400010C as *mut u32, 0xC00000); // min reload, start timer with irqs
    }

    let file_test = RomFile::open("test.txt").unwrap();

    println!("{}", file_test.as_str().unwrap());

    let sp_irq: u32;
    unsafe {
        asm!(".align 4",
             "NOP",
             "BX R15",
             ".arm",
             "MRS R0, CPSR",
             "MSR CPSR_c, #0xD2",
             "MOV R1, R13",
             "MSR CPSR_c, R0",
             "ADR R0, 1f + 5",
             "1: BX R0",
             ".thumb",
             out("r0") _,
             out("r1") sp_irq);
    }

    println!("SP_IRQ: {:x}", sp_irq);

    let file = RomFile::open("img/gba_yeen.img").unwrap();
    fast_mem::call_on_fast_stack(|| video::display_bitmap_file(file));

    #[allow(clippy::empty_loop)]
    loop {}
}
