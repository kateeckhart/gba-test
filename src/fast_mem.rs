use crate::volatile::{VolatileBool, VolatileMutPtr, VolatileUsize};
use alloc::alloc::{AllocError, Allocator, Layout};
use alloc::boxed::Box;
use core::arch::asm;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ptr::{self, NonNull};
use core::slice;

#[link_section = ".fast_bss"]
static FAST_STACK: [u32; 0x400] = [0; 0x400];

static FAST_STACK_USED: VolatileBool = VolatileBool::new(false);

type FAST_HEAP_TYPE = UnsafeCell<[u32; 0x800]>;

#[link_section = ".fast_bss"]
static mut FAST_HEAP: FAST_HEAP_TYPE = UnsafeCell::new([0; 0x800]);

#[link_section = ".fast_bss"]
static FAST_HEAP_COUNT: VolatileUsize = VolatileUsize::new(0);

#[link_section = ".fast_data"]
static FAST_HEAP_CURRENT: VolatileMutPtr<u8> =
    unsafe { VolatileMutPtr::new(FAST_HEAP.get() as *mut u8) };

pub struct FastAllocator {
    _priv: PhantomData<*mut ()>,
}

impl FastAllocator {
    pub fn new() -> Self {
        let count = FAST_HEAP_COUNT.read();
        assert!(count != usize::MAX);
        FAST_HEAP_COUNT.write(count + 1);
        Self { _priv: PhantomData }
    }
}

impl Clone for FastAllocator {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl Drop for FastAllocator {
    fn drop(&mut self) {
        let count = FAST_HEAP_COUNT.read();
        FAST_HEAP_COUNT.write(count - 1);
        if count == 1 {
            unsafe { FAST_HEAP_CURRENT.write(FAST_HEAP.get() as *mut u8) }
        }
    }
}

unsafe impl Allocator for FastAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let current = FAST_HEAP_CURRENT.read();
        let fast_heap_loc = unsafe { FAST_HEAP.get() as usize };
        let left = mem::size_of::<FAST_HEAP_TYPE>() - ((current as usize) - (fast_heap_loc));
        let used = current.align_offset(layout.align()) + layout.size();

        if used > left {
            return Err(AllocError);
        }

        unsafe {
            let ret = current.add(current.align_offset(layout.align()));
            let next_current = ret.add(layout.size());
            FAST_HEAP_CURRENT.write(next_current);
            Ok(NonNull::new_unchecked(slice::from_raw_parts_mut(
                ret,
                layout.size(),
            )))
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {}
}

#[no_mangle]
extern "C" fn fast_stack_tramp(fun: Box<Box<dyn FnOnce() -> ()>>) {
    fun();
}

fn raw_call_on_fast_stack(fun: Box<Box<dyn FnOnce() -> ()>>) {
    assert!(!FAST_STACK_USED.swap(true), "Already on fast stack");
    unsafe {
        asm!("MOV R5, R13",
         "MOV R13, R1",
         "BL fast_stack_tramp",
         "MOV R13, R5",
         in("r0") Box::into_raw(fun),
         in("r1") (&FAST_STACK as *const _ as usize) + FAST_STACK.len(),
         out("r5") _,
         clobber_abi("C"))
    }
    FAST_STACK_USED.write(false);
}

pub fn call_on_fast_stack<T: FnOnce() -> R, R>(fun: T) -> R {
    let mut ret_place = MaybeUninit::uninit();
    let ret_place_ptr = ret_place.as_mut_ptr() as *mut ();
    let mut fun_place = MaybeUninit::new(fun);
    let fun_place_ptr = fun_place.as_mut_ptr() as *mut ();
    let wrapped_function: Box<dyn FnOnce()> = Box::new(move || unsafe {
        let fun = ptr::read(fun_place_ptr as *mut T);
        ptr::write(ret_place_ptr as *mut R, fun())
    });
    raw_call_on_fast_stack(Box::new(wrapped_function));
    unsafe { ret_place.assume_init() }
}
