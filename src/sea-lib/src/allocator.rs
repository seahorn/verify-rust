use core::alloc::{GlobalAlloc, Layout};
use libc::c_void;
pub struct CAllocator {}


#[global_allocator]
static ALLOCATOR: CAllocator = CAllocator {};


unsafe impl GlobalAlloc for CAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
           crate::seahorn::verifier_error();
        }
        libc::malloc(layout.size()) as *mut u8
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        libc::free(_ptr as *mut c_void);
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        let new_ptr = libc::malloc(new_size) as *mut u8;
        if !new_ptr.is_null() {
            let sz_to_cpy = layout.size().min(new_size);
            core::ptr::copy_nonoverlapping(ptr, new_ptr, sz_to_cpy);
        }
        new_ptr
    }
}

