#[cfg(target_arch = "aarch64")]
const GFX_GAMMA_SIGNATURES: &[&str] = &[
    "68 E1 0D F8 48 02 80 52 A8 03 16 38 28 0C 80 52 BF E3 1A 38 69 91 09 F8 68 11 0A 78 E8 4D 82 52 01 E4 00 2F 00 10 2C 1E 68 50 A7 72 02 10 2E 1E",
];

pub fn patch_gfx_gamma() -> Result<(), &'static str> {
    for signature in GFX_GAMMA_SIGNATURES.iter() {
        if let Some(addr) = resolve_signature(signature) {
            let movk_addr = unsafe { addr.offset(40) };
            let max_addr = unsafe { addr.offset(44) };

            let max_bytes = unsafe { std::slice::from_raw_parts(max_addr, 4) };

            if max_bytes != [0x02, 0x10, 0x2E, 0x1E] {
                continue;
            }

            unsafe {
                use region::{protect, Protection};

                protect(movk_addr, 8, Protection::READ_WRITE_EXECUTE)
                    .map_err(|_| "Memory protection failed")?;

                              std::ptr::write_unaligned(max_addr as *mut u32, 0x1E220102);

                std::ptr::write_unaligned(movk_addr as *mut u32, 0x52800148);

                clear_cache::clear_cache(movk_addr, movk_addr.add(4));
                clear_cache::clear_cache(max_addr, max_addr.add(4));

                protect(movk_addr, 8, Protection::READ_EXECUTE).ok();
            }

            return Ok(());
        }
    }

    Err("Signature not found")
}

fn resolve_signature(signature: &str) -> Option<*const u8> {
    unsafe {
        let sig_cstr = std::ffi::CString::new(signature).ok()?;
        let mod_cstr = std::ffi::CString::new("libminecraftpe.so").ok()?;

        let result = crate::preloader::pl_resolve_signature(sig_cstr.as_ptr(), mod_cstr.as_ptr());
        if result == 0 {
            None
        } else {
            Some(result as *const u8)
        }
    }
}