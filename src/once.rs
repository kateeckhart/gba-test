use crate::volatile::VolatileBool;
use core::mem::MaybeUninit;
use core::ptr;
use core::ops;
use core::cell::UnsafeCell;

pub struct Lazy<T, F = fn() -> T> {
    builder: UnsafeCell<Option<F>>,
    value: UnsafeCell<MaybeUninit<T>>,
    building: VolatileBool,
    built: VolatileBool,
}

unsafe impl<T: Sync, F: Send> Sync for Lazy<T, F> {}

impl<T, F> Lazy<T, F> {
    pub const fn new(fun: F) -> Self {
        Lazy {
            builder: UnsafeCell::new(Some(fun)),
            value: UnsafeCell::new(MaybeUninit::uninit()),
            building: VolatileBool::new(false),
            built: VolatileBool::new(false),
        }
    }
}

impl<T, F: FnOnce() -> T> ops::Deref for Lazy<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if self.built.read() {
            unsafe {
                return &(*(*self.value.get()).as_ptr())
            }
        }

        let building = self.building.swap(true);

        if building {
            panic!("Double init of lazy");
        }

        let builder = unsafe {
            (*self.builder.get()).take()
        }.unwrap();

        let value = builder();
        unsafe {
            (*self.value.get()) = MaybeUninit::new(value)
        }
        self.built.write(true);

        &**self
    }
}

impl<T, F> ops::Drop for Lazy<T, F> {
    fn drop(&mut self) {
        if self.built.read() {
            unsafe {
                ptr::drop_in_place((*self.value.get()).as_mut_ptr());
            }
        }
    }
}
