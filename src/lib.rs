use once_cell::sync::Lazy;
use std::{
    alloc::{GlobalAlloc, Layout},
    mem::size_of,
    ptr,
};

use libc::{c_char, c_void, size_t};

enum PSMHandleTag {}
type PSMHandle = *mut PSMHandleTag;

extern "C" {
    fn PSMinit(
        name: *const c_char,
        init_size: size_t,
        p_req_address: *mut c_void,
        handle: *mut PSMHandle,
    );
    fn PSMdeinit(handle: PSMHandle);
    fn PSMalloc(handle: PSMHandle, alloc_size: size_t) -> *mut c_void;
    fn PSMfree(handle: PSMHandle, paddress: *mut c_void);
    fn PSMgetUser(handle: PSMHandle) -> *mut *mut c_void;
    // TODO: fn PSMgeterror(handle: PSMHandle) -> c_int;
}

pub struct PSMAllocator {
    handle: PSMHandle,
}

unsafe impl Sync for PSMAllocator {}
unsafe impl Send for PSMAllocator {}

impl PSMAllocator {
    pub unsafe fn new(shmem_file: &str, size: size_t, req_addr: *mut c_void) -> Self {
        let mut cpath = [0; 0x1000];
        let path_len = shmem_file.as_bytes().len();
        if path_len >= cpath.len() {
            panic!("shmem_file path too long");
        }

        cpath[..path_len].copy_from_slice(shmem_file.as_bytes());
        cpath[path_len] = 0;

        let mut handle: PSMHandle = ptr::null_mut();
        PSMinit(
            cpath.as_ptr() as *mut _,
            size,
            req_addr,
            &mut handle as *mut _,
        );

        Self { handle }
    }

    pub unsafe fn get_user(&self) -> *mut *mut c_void {
        PSMgetUser(self.handle)
    }

    pub unsafe fn deinit(&self) {
        PSMdeinit(self.handle)
    }
}

// unsafe impl Allocator for PSMAllocator {
//     fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//         let size = layout.size();
//         let raw = unsafe { PSMalloc(self.handle, size) };

//         let ptr = NonNull::new(raw as *mut _).ok_or(AllocError)?;
//         Ok(NonNull::slice_from_raw_parts(ptr, size))
//     }

//     unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
//         PSMfree(self.handle, ptr.as_ptr() as *mut _)
//     }
// }

unsafe impl GlobalAlloc for PSMAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        let full_size = layout.pad_to_align().size() + size_of::<*mut u8>();

        let block = PSMalloc(self.handle, full_size) as *mut u8;
        if block.is_null() {
            return block;
        }

        let correction = (align - block as usize % align) % align;
        let user_block = block.add(correction);

        let backref = user_block.add(size) as *mut *mut u8;
        ptr::write(backref, block);

        user_block
    }

    unsafe fn dealloc(&self, user_block: *mut u8, layout: Layout) {
        let size = layout.size();

        let backref = user_block.add(size) as *mut *mut u8;
        let block = ptr::read(backref);

        PSMfree(self.handle, block as *mut _)
    }
}

impl Drop for PSMAllocator {
    fn drop(&mut self) {
        unsafe { self.deinit() }
    }
}

static GLOBAL: Lazy<PSMAllocator> =
    Lazy::new(|| unsafe { PSMAllocator::new("test.psm", 0x10000000, ptr::null_mut()) });

pub struct GlobalPSMAllocator;

unsafe impl GlobalAlloc for GlobalPSMAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let res = GLOBAL.alloc(layout);

        if !res.is_null() {
            eprintln!("memory allocation of {} bytes ok", layout.size());
        } else {
            eprintln!("memory allocation failed");
        }

        return res;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        GLOBAL.dealloc(ptr, layout)
    }
}
