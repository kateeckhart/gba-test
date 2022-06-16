use core::ptr;

#[no_mangle]
static mut TIMER_VALUE: u32 = 0;

pub fn get_timer() -> u32 {
    unsafe { ptr::read_volatile(&TIMER_VALUE) }
}
