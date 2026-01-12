use jni::{
    objects::{JObject, JString},
    sys::{jboolean, JNI_TRUE},
    JNIEnv,
};
use materialbin::{MinecraftVersion, ALL_VERSIONS};
use std::sync::{LazyLock, Mutex};

use crate::LockResultExt;
pub struct Options {
    pub handle_lightmaps: bool,
    pub handle_texturelods: bool,
    pub autofixer_versions: Vec<MinecraftVersion>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            handle_lightmaps: true,
            handle_texturelods: true,
            autofixer_versions: ALL_VERSIONS.to_vec(),
        }
    }
}
pub static OPTS: LazyLock<Mutex<Options>> = LazyLock::new(|| Mutex::new(Options::default()));