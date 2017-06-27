// The compiler needs to be instructed that this crate is an allocator in order
// to realize that when this is linked in another allocator like jemalloc should
// not be linked in
#![feature(allocator)]
#![allocator]

// Allocators are not allowed to depend on the standard library which in turn
// requires an allocator in order to avoid circular dependencies. This crate,
// however, can use all of libcore.
#![no_std]

// Let's give a unique name to our custom allocator
#![crate_name = "david_allocator"]
#![crate_type = "rlib"]

// Our system allocator will use hand-written FFI bindings. Note that
// currently the external (crates.io) libc cannot be used because it
// links to the standard library (e.g. `#![no_std]` isn't stable yet),
// so that's why this specifically requires the in-tree version.
extern "C" {
    fn malloc(sz: usize) -> *mut u8;
    fn calloc(sz: usize, sz2: usize) -> *mut u8;
    fn free(ptr: *mut u8);
    fn realloc(ptr: *mut u8, sz: usize) -> *mut u8;
}

// Listed below are the five allocation functions currently required by custom
// allocators. Their signatures and symbol names are not currently typechecked
// by the compiler, but this is a future extension and are required to match
// what is found below.
//
// Note that the standard `malloc` and `realloc` functions do not provide a way
// to communicate alignment so this implementation would need to be improved
// with respect to alignment in that aspect.

static mut NET: usize = 0;
static mut TOTAL: usize = 0;

#[no_mangle]
pub extern fn __rust_allocate(size: usize, _align: usize) -> *mut u8 {
    unsafe {
        NET += size;
        TOTAL += size;
        malloc(size)
    }
}

#[no_mangle]
pub extern fn __rust_allocate_zeroed(size: usize, _align: usize) -> *mut u8 {
    unsafe {
        NET += size;
        TOTAL += size;
        calloc(size,1)
    }
}

#[no_mangle]
pub extern fn __rust_deallocate(ptr: *mut u8, old_size: usize, _align: usize) {
    unsafe {
        NET -= old_size;
        free(ptr)
    }
}

#[no_mangle]
pub extern fn __rust_reallocate(ptr: *mut u8, old_size: usize, size: usize,
                                _align: usize) -> *mut u8 {
    unsafe {
        TOTAL += size;
        NET += size;
        NET -= old_size;
        realloc(ptr, size)
    }
}

#[no_mangle]
pub extern fn __rust_reallocate_inplace(_ptr: *mut u8, old_size: usize,
                                        _size: usize, _align: usize) -> usize {
    old_size // this api is not supported by libc
}

#[no_mangle]
pub extern fn __rust_usable_size(size: usize, _align: usize) -> usize {
    size
}

pub fn net_allocation() -> usize {
    unsafe { NET }
}

pub fn total_allocation() -> usize {
    unsafe { TOTAL }
}
