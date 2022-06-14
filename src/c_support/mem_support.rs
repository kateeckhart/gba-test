use core::alloc::{GlobalAlloc, Layout};
use core::convert::Infallible;

extern "C" {
    fn __malloc_begin(uncallable: Infallible) -> !;
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
}

#[no_mangle]
extern "C" fn malloc_abort() {
    panic!("Malloc abort!")
}

static mut NEXT_FREE_POINTER: *mut () = __malloc_begin as *mut ();

#[no_mangle]
unsafe extern "C" fn sbrk(raw_inc: i32) -> *mut () {
    if raw_inc < 0 {
        return usize::MAX as *mut ();
    }
    let inc = raw_inc as usize;
    if inc > 256 * 1024 {
        panic!("Attempted to alloc more memory than the gba has")
    }

    unsafe {
        let ret = NEXT_FREE_POINTER;
        NEXT_FREE_POINTER = ((NEXT_FREE_POINTER as usize) + inc) as *mut ();
        if NEXT_FREE_POINTER as usize > 0x203FFFF {
            panic!("Out of memory!")
        }
        ret
    }
}

#[global_allocator]
static ALLOCATOR: Alloc = Alloc;

struct Alloc;

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { malloc(layout.size()) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _: Layout) {
        unsafe { free(ptr) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, _: Layout, new_size: usize) -> *mut u8 {
        unsafe { realloc(ptr, new_size) }
    }
}
