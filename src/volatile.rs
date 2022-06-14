use core::ptr;
use core::cell::UnsafeCell;
use core::sync::atomic::Ordering;
use core::sync::atomic::compiler_fence;

pub struct VolatileBool(UnsafeCell<bool>);

unsafe impl Sync for VolatileBool {}

extern "C" {
    fn atomic_swap_byte(loc: *mut u8, value: u8) -> u8;
}

impl VolatileBool {
    pub fn read(&self) -> bool {
        compiler_fence(Ordering::SeqCst);
        let ret = unsafe {
            ptr::read_volatile(self.0.get())
        };
        compiler_fence(Ordering::SeqCst);
        ret
    }

    pub fn swap(&self, replace: bool) -> bool {
        compiler_fence(Ordering::SeqCst);
        let replace_num = u8::from(replace);
        let ret = unsafe { atomic_swap_byte(self.0.get() as *mut u8, replace_num) != 0 };
        compiler_fence(Ordering::SeqCst);
        ret
    }

    pub fn write(&self, value: bool) {
        compiler_fence(Ordering::SeqCst);
        unsafe {
            ptr::write_volatile(self.0.get(), value)
        }
        compiler_fence(Ordering::SeqCst);
    }

    pub const fn new(value: bool) -> Self {
        VolatileBool(UnsafeCell::new(value))
    }
}
