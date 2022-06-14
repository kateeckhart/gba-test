use crate::volatile::VolatileBool;
use core::cell::UnsafeCell;
use core::ops;
use core::mem;

pub struct IrqSafeRefCell<T>(UnsafeCell<T>, VolatileBool);

impl<T> IrqSafeRefCell<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value), VolatileBool::new(false))
    }

    pub fn borrow_mut(&self) -> IrqSafeRefMut<'_, T> {
        self.try_borrow_mut().expect("Cell is already acquired")
    }

    pub fn try_borrow_mut(&self) -> Option<IrqSafeRefMut<'_, T>> {
        let locked = self.1.swap(true);
        if locked {
            None
        } else {
            unsafe {
                Some(IrqSafeRefMut(&mut *(self.0.get()), &self.1))
            }
        }
    }
}

unsafe impl<T: Send> Sync for IrqSafeRefCell<T> {}

pub struct IrqSafeRefMut<'a, T>(&'a mut T, &'a VolatileBool);

impl<'a, T> IrqSafeRefMut<'a, T> {
    pub fn map<F: FnOnce(&mut T) -> &mut U, U>(cell: Self, fun: F) -> IrqSafeRefMut<'a, U> {
        let new_ref = fun(cell.0);
        let extended_new_ref = unsafe {
            &mut *(new_ref as *mut U)
        };
        let ret = IrqSafeRefMut(extended_new_ref, cell.1);
        mem::forget(cell);
        ret
    }
}

impl<'a, T> ops::Drop for IrqSafeRefMut<'a, T> {
    fn drop(&mut self) {
        self.1.write(false)
    }
}

impl<'a, T> ops::Deref for IrqSafeRefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0
    }
}

impl<'a, T> ops::DerefMut for IrqSafeRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}
