use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;

extern "C" {
    fn atomic_swap_byte(loc: *mut u8, value: u8) -> u8;
}

pub struct VolatileBool(UnsafeCell<bool>);

unsafe impl Sync for VolatileBool {}

impl VolatileBool {
    pub fn read(&self) -> bool {
        compiler_fence(Ordering::SeqCst);
        let ret = unsafe { ptr::read_volatile(self.0.get()) };
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
        unsafe { ptr::write_volatile(self.0.get(), value) }
        compiler_fence(Ordering::SeqCst);
    }

    pub const fn new(value: bool) -> Self {
        VolatileBool(UnsafeCell::new(value))
    }
}

pub struct VolatileMutPtr<T>(UnsafeCell<*mut T>);
unsafe impl<T> Sync for VolatileMutPtr<T> {}

impl<T> VolatileMutPtr<T> {
    pub fn read(&self) -> *mut T {
        unsafe { ptr::read_volatile(self.0.get()) }
    }

    pub fn write(&self, value: *mut T) {
        unsafe { ptr::write_volatile(self.0.get(), value) }
    }

    pub const fn new(value: *mut T) -> Self {
        VolatileMutPtr(UnsafeCell::new(value))
    }
}

pub struct VolatileUsize(UnsafeCell<usize>);
unsafe impl Sync for VolatileUsize {}

impl VolatileUsize {
    pub fn read(&self) -> usize {
        unsafe { ptr::read_volatile(self.0.get()) }
    }

    pub fn write(&self, value: usize) {
        unsafe { ptr::write_volatile(self.0.get(), value) }
    }

    pub const fn new(value: usize) -> Self {
        VolatileUsize(UnsafeCell::new(value))
    }
}
