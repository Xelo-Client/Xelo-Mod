mod autofixer;
#[deny(clippy::indexing_slicing)]
use std::{
    ffi::CStr,
    fs,
    pin::Pin,
    ptr::null_mut,
    str::SplitWhitespace,
    sync::{atomic::AtomicPtr, OnceLock, LockResult, Mutex},
};
mod config;
use config::init_config;
mod aasset;
mod plthook;
mod jniopts;
use crate::plthook::replace_plt_functions;
mod preloader;
mod brightness;
use bhook::hook_fn as bhook_hook_fn;
use core::mem::transmute;
use cxx::CxxString;
use libc::c_void;
use plt_rs::DynamicLibrary;
use tinypatscan::Pattern;

#[cfg(target_arch = "aarch64")]
const RPMC_SIGNATURES: [&str; 3] = [
    "FF ?? 02 D1 FD 7B ?? A9 ?? ?? ?? ?? FA 67 ?? A9 F8 5F ?? A9 F6 57 ?? A9 F4 4F ?? A9 FD ?? 01 91 ?? D0 3B D5 ?? 03 03 2A ?? 03 02 AA ?? 17 40 F9 F3 03 00 AA A8 83 1F F8",
    "FF 83 02 D1 FD 7B 06 A9 FD 83 01 91 F8 5F 07 A9 F6 57 08 A9 F4 4F 09 A9 58 D0 3B D5 F6 03 03 2A 08 17 40 F9 F5 03 02 AA F3 03 00 AA A8 83 1F F8 28 10 40 F9 28 01 00 B4",
    "FF 03 03 D1 FD 7B 07 A9 FD C3 01 91 F9 43 00 F9 F8 5F 09 A9 F6 57 0A A9 F4 4F 0B A9 59 D0 3B D5 F6 03 03 2A 28 17 40 F9 F5 03 02 AA F3 03 00 AA A8 83 1F F8 28 10 40 F9",
    
];

#[repr(transparent)]
pub struct ResourceLocation(*mut c_void);

impl ResourceLocation {
    pub fn from_str(str: &CStr) -> ResourceLocation {
        unsafe { resource_location_init(str.as_ptr(), str.count_bytes()) }
    }
}
impl Drop for ResourceLocation {
    fn drop(&mut self) {
        unsafe { resource_location_free(self.0) }
    }
}
extern "C" {
    fn resource_location_init(strptr: *const libc::c_char, size: libc::size_t) -> ResourceLocation;
    fn resource_location_free(loc: *mut c_void);
}
pub fn setup_logging() {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );
}
#[ctor::ctor]
fn safe_setup() {
    setup_logging();
config::init_config();
    std::panic::set_hook(Box::new(move |_panic_info| {}));
    main();
}
fn main() {
if config::is_better_brightness_enabled() {
        let _ = brightness::patch_gfx_gamma();
    }
    let addr = find_signatures_using_pl_lib().expect("No signature was found");
    
    unsafe {
        rpm_ctor::hook_address(addr as *mut u8);
    };
    hook_aaset();
}


#[derive(Debug)]
struct SimpleMapRange {
    start: usize,
    size: usize,
}

impl SimpleMapRange {
    fn start(&self) -> usize {
        self.start
    }

    fn size(&self) -> usize {
        self.size
    }
}

fn find_minecraft_library_manually() -> Result<SimpleMapRange, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("/proc/self/maps")?;
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((addr_start, addr_end)) = parse_range(line.split_whitespace()) else {
            continue;
        };
        let start = usize::from_str_radix(addr_start, 16)?;
        let end = usize::from_str_radix(addr_end, 16)?;
        log::info!("Found libminecraftpe.so at: {:x}-{:x}", start, end);
        return Ok(SimpleMapRange {
            start,
            size: end - start,
        });
    }

    Err("libminecraftpe.so not found in memory maps".into())
}
fn parse_range(mut line: SplitWhitespace) -> Option<(&str, &str)> {
    let addr_range = line.next()?;
    let perms = line.next()?;
    let pathname = line.last()?;
    if perms.contains('x') && pathname.ends_with("libminecraftpe.so") {
        return addr_range.split_once('-');
    }
    None
}

fn find_signatures_using_pl_lib() -> Option<*const u8> {
    for &signature in &RPMC_SIGNATURES {
        if let Some(addr) = resolve_pl_signature(signature, "libminecraftpe.so") {
            #[cfg(target_arch = "arm")]
            let addr = unsafe { addr.offset(1) };
            return Some(addr);
        }
    }
    None
}
macro_rules! cast_array {
    ($($func_name:literal -> $hook:expr),
        *,
    ) => {
        [
            $(($func_name, $hook as *const u8)),*,
        ]
    }
}

pub fn hook_aaset() {
    let lib_entry = find_lib("libminecraftpe").expect("Cannot find minecraftpe");
    let dyn_lib = DynamicLibrary::initialize(lib_entry).expect("Failed to find mc info");
    let asset_fn_list = cast_array! {
        "AAssetManager_open" -> aasset::open,
        "AAsset_read" -> aasset::read,
        "AAsset_close" -> aasset::close,
        "AAsset_seek" -> aasset::seek,
        "AAsset_seek64" -> aasset::seek64,
        "AAsset_getLength" -> aasset::len,
        "AAsset_getLength64" -> aasset::len64,
        "AAsset_getRemainingLength" -> aasset::rem,
        "AAsset_getRemainingLength64" -> aasset::rem64,
        "AAsset_openFileDescriptor" -> aasset::fd_dummy,
        "AAsset_openFileDescriptor64" -> aasset::fd_dummy64,
        "AAsset_getBuffer" -> aasset::get_buffer,
        "AAsset_isAllocated" -> aasset::is_alloc,
    };
    replace_plt_functions(&dyn_lib, asset_fn_list);
}

fn resolve_pl_signature(signature: &str, module_name: &str) -> Option<*const u8> {
    unsafe {
        let sig_cstr = std::ffi::CString::new(signature).unwrap();
        let mod_cstr = std::ffi::CString::new(module_name).unwrap();
        let result = preloader::pl_resolve_signature(sig_cstr.as_ptr(), mod_cstr.as_ptr());
        if result == 0 {
            None
        } else {
            Some(result as *const u8)
        }
    }
}

fn find_lib<'a>(target_name: &str) -> Option<plt_rs::LoadedLibrary<'a>> {
    let loaded_modules = plt_rs::collect_modules();
    loaded_modules
        .into_iter()
        .find(|lib| lib.name().contains(target_name))
}
pub static PACKM_OBJ: AtomicPtr<libc::c_void> = AtomicPtr::new(null_mut());
pub static RPM_LOAD: OnceLock<RpmLoadFn> = OnceLock::new();

hook_fn! {
    fn rpm_ctor(this: *mut libc::c_void,unk1: usize,unk2: usize,needs_init: bool) -> *mut libc::c_void = {
        use std::sync::atomic::Ordering;
        log::info!("rpm ctor called");
        let result = call_original(this, unk1, unk2, needs_init);
        log::info!("RPM pointer has been obtained");
        crate::PACKM_OBJ.store(this, Ordering::Release);
        crate::RPM_LOAD.set(crate::get_load(this)).unwrap();
        self_disable();
        log::info!("hook exit");
        result
    }
}

type RpmLoadFn = unsafe extern "C" fn(*mut c_void, ResourceLocation, Pin<&mut CxxString>) -> bool;
unsafe fn get_load(packm_ptr: *mut c_void) -> RpmLoadFn {
    let vptr = *transmute::<*mut c_void, *mut *mut *const u8>(packm_ptr);
    transmute::<*const u8, RpmLoadFn>(*vptr.offset(2))
}

pub trait LockResultExt {
    type Guard;
    fn ignore_poison(self) -> Self::Guard;
}

impl<Guard> LockResultExt for LockResult<Guard> {
    type Guard = Guard;
    fn ignore_poison(self) -> Guard {
        self.unwrap_or_else(|e| e.into_inner())
    }
}