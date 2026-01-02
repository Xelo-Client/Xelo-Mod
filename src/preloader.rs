use libc::c_void;

#[link(name = "preloader")]
extern "C" {
    pub fn pl_hook(
        target: *mut c_void,
        detour: *mut c_void,
        original_func: *mut *mut c_void,
        priority: i32,
    ) -> i32;

    pub fn pl_unhook(target: *mut c_void, detour: *mut c_void) -> bool;

    pub fn pl_resolve_signature(signature: *const libc::c_char, module_name: *const libc::c_char) -> libc::uintptr_t;
}

#[macro_export]
macro_rules! hook_fn {
    ($vis:vis fn $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty = $body:block) => {
        hook_fn!($vis fn $name($($arg: $ty),*) -> $ret = $body, priority = 10);
    };
    
    ($vis:vis fn $name:ident($($arg:ident: $ty:ty),*) -> $ret:ty = $body:block, priority = $priority:expr) => {
        mod $name {
            use super::*;
            use std::sync::OnceLock;
            use std::sync::atomic::{AtomicPtr, Ordering};
 
            static ORIGINAL_FN: OnceLock<unsafe extern "C" fn($($ty),*) -> $ret> = OnceLock::new();
            static TARGET_ADDR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
 
            pub unsafe extern "C" fn detour($($arg: $ty),*) -> $ret {
                $body
            }
 
            pub fn call_original($($arg: $ty),*) -> $ret {
                unsafe {
                    let original = ORIGINAL_FN.get().expect("Original function not set");
                    original($($arg),*)
                }
            }
 
            pub fn hook_address(addr: *mut u8) {
                unsafe {
                    let mut original_ptr: *mut c_void = std::ptr::null_mut();
                    let result = crate::preloader::pl_hook(
                        addr as *mut c_void,
                        detour as *mut c_void,
                        &mut original_ptr,
                        $priority,
                    );
 
                    if result == 0 && !original_ptr.is_null() {
                        let original_fn: unsafe extern "C" fn($($ty),*) -> $ret =
                            std::mem::transmute(original_ptr);
                        ORIGINAL_FN.set(original_fn).expect("Failed to set original function");
 
                        TARGET_ADDR.store(addr as *mut c_void, Ordering::Release);
                    } else {
                        panic!("Failed to hook function");
                    }
                }
            }
 
            pub fn self_disable() {
                unsafe {
                    let target = TARGET_ADDR.load(Ordering::Acquire);
                    if !target.is_null() {
                        crate::preloader::pl_unhook(target, detour as *mut c_void);
                    }
                }
            }
        }
    };
}