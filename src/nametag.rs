#[cfg(target_arch = "aarch64")]
const NAMETAG_SIGNATURE: &str = "? ? 40 F9 \
    ? ? ? EB \
    ? ? ? 54 \
    ? ? 40 F9 \
    ? 81 40 F9 \
    E0 03 ? AA \
    00 01 3F D6 \
    ? ? 00 37 \
    ? ? 40 F9 \
    ? ? ? A9 \
    ? ? ? CB \
    ? ? ? D3 \
    ? ? 00 51 \
    ? ? ? 8A";

const PATCH_OFFSET: isize = 8;
const PATCH_BYTES: [u8; 4] = [0x1F, 0x20, 0x03, 0xD5];

use std::sync::Mutex;

static ORIGINAL_BYTES: Mutex<Option<[u8; 4]>> = Mutex::new(None);
static PATCH_ADDR: Mutex<Option<usize>> = Mutex::new(None);

pub fn patch_nametag() -> Result<(), &'static str> {
    let addr = resolve_signature(NAMETAG_SIGNATURE)
        .ok_or("Signature not found")?;

    let patch_addr = unsafe { addr.offset(PATCH_OFFSET) };

    unsafe {
        use region::{protect, Protection};

        protect(patch_addr, 4, Protection::READ_WRITE_EXECUTE)
            .map_err(|_| "Memory protection failed")?;

        let mut original = [0u8; 4];
        std::ptr::copy_nonoverlapping(
            patch_addr as *const u8,
            original.as_mut_ptr(),
            4,
        );

        *ORIGINAL_BYTES.lock().unwrap() = Some(original);
        *PATCH_ADDR.lock().unwrap() = Some(patch_addr as usize);

        std::ptr::copy_nonoverlapping(
            PATCH_BYTES.as_ptr(),
            patch_addr as *mut u8,
            PATCH_BYTES.len(),
        );

        clear_cache::clear_cache(patch_addr, patch_addr.add(4));

        protect(patch_addr, 4, Protection::READ_EXECUTE).ok();

        Ok(())
    }
}

pub fn unpatch_nametag() -> Result<(), &'static str> {
    let original = ORIGINAL_BYTES.lock().unwrap()
        .ok_or("No original bytes stored")?;

    let patch_addr = PATCH_ADDR.lock().unwrap()
        .ok_or("No patch address stored")? as *mut u8;

    unsafe {
        use region::{protect, Protection};

        protect(patch_addr, 4, Protection::READ_WRITE_EXECUTE)
            .map_err(|_| "Memory protection failed")?;

        std::ptr::copy_nonoverlapping(
            original.as_ptr(),
            patch_addr,
            4,
        );

        clear_cache::clear_cache(patch_addr, patch_addr.add(4));

        protect(patch_addr, 4, Protection::READ_EXECUTE).ok();

        Ok(())
    }
}

fn resolve_signature(signature: &str) -> Option<*const u8> {
    unsafe {
        let sig_cstr = std::ffi::CString::new(signature).ok()?;
        let mod_cstr = std::ffi::CString::new("libminecraftpe.so").ok()?;

        let result = crate::preloader::pl_resolve_signature(
            sig_cstr.as_ptr(),
            mod_cstr.as_ptr(),
        );
        (result != 0).then_some(result as *const u8)
    }
}