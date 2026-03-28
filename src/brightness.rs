#[cfg(target_arch = "aarch64")]
const GFX_GAMMA_SIGNATURES: &[&str] = &[
    "CA 92 06 F8 29 01 40 F9 C8 E2 06 F8 48 02 80 52 A8 03 16 38 28 0C 80 52 BF E3 1A 38 C9 92 02 F8 C8 12 03 78 E8 4D 82 52 01 E4 00 2F 00 10 2C 1E 68 50 A7 72 02 10 2E 1E",
];

pub fn patch_gfx_gamma() -> Result<(), &'static str> {
    for signature in GFX_GAMMA_SIGNATURES.iter() {
        if let Some(addr) = resolve_signature(signature) {
            let movk_addr = unsafe { addr.offset(48) };
            let fmov_addr = unsafe { addr.offset(52) };

            let fmov_bytes = unsafe { std::slice::from_raw_parts(fmov_addr, 4) };

            if fmov_bytes != [0x02, 0x10, 0x2E, 0x1E] {
                continue;
            }

            unsafe {
                use region::{protect, Protection};

                protect(movk_addr, 8, Protection::READ_WRITE_EXECUTE)
                    .map_err(|_| "Memory protection failed")?;

                std::ptr::write_unaligned(fmov_addr as *mut u32, 0x1E220102);

                std::ptr::write_unaligned(movk_addr as *mut u32, 0x52800148);

                clear_cache::clear_cache(movk_addr, movk_addr.add(4));
                clear_cache::clear_cache(fmov_addr, fmov_addr.add(4));

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