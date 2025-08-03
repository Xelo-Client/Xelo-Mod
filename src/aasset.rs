use crate::ResourceLocation;
use crate::config::{is_no_hurt_cam_enabled, is_no_fog_enabled, is_java_cubemap_enabled, is_particles_disabler_enabled, is_java_clouds_enabled, is_classic_skins_enabled, is_cape_physics_enabled, is_night_vision_enabled, is_xelo_title_enabled, is_client_capes_enabled};
use libc::{off64_t, off_t};
use materialbin::{CompiledMaterialDefinition, MinecraftVersion};
use ndk::asset::Asset;
use ndk_sys::{AAsset, AAssetManager};
use once_cell::sync::Lazy;
use scroll::Pread;
use serde_json::{Value, Map};
use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::{CStr, CString, OsStr},
    io::{self, Cursor, Read, Seek, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

#[derive(PartialEq, Eq, Hash)]
struct AAssetPtr(*const ndk_sys::AAsset);
unsafe impl Send for AAssetPtr {}

static MC_VERSION: OnceLock<Option<MinecraftVersion>> = OnceLock::new();

static WANTED_ASSETS: Lazy<Mutex<HashMap<AAssetPtr, Cursor<Vec<u8>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

const LEGACY_CUBEMAP_MATERIAL_BIN: &[u8] = include_bytes!("java_cubemap/LegacyCubemap.material.bin");
const RENDER_CHUNK_MATERIAL_BIN: &[u8] = include_bytes!("no_fog_materials/RenderChunk.material.bin");

const CAPE_TEXTURE_PATH: &str = "/storage/emulated/0/Android/data/com.origin.launcher/files/origin_mods/xelo_cape.png";

const TITLE_PNG: &[u8] = include_bytes!("minecraft_title_5.png");

const MOBS_JSON: &[u8] = include_bytes!("cape_physics/mobs.json");
const PLAYER_ANIMATION_JSON: &[u8] = include_bytes!("cape_physics/player.animation.json");

const RENDER_CHUNK_NV_MATERIAL_BIN: &[u8] = include_bytes!("nightvision_materials/RenderChunk.material.bin");

const CUSTOM_SPLASHES_JSON: &str = r#"{"splashes":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"]}"#;

const CUSTOM_FIRST_PERSON_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:first_person"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_first_person":{},"minecraft:camera_render_first_person_objects":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,0,0]},"minecraft:camera_direct_look":{"pitch_min":-89.9,"pitch_max":89.9},"minecraft:camera_perspective_option":{"view_mode":"first_person"},"minecraft:update_player_from_camera":{"look_mode":"along_camera"},"minecraft:extend_player_rendering":{},"minecraft:camera_player_sleep_vignette":{},"minecraft:vr_comfort_move":{},"minecraft:default_input_camera":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{}}}}"#;
const CUSTOM_THIRD_PERSON_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:third_person"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_third_person":{},"minecraft:camera_render_player_model":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,2,5]},"minecraft:camera_look_at_player":{},"minecraft:camera_orbit":{"azimuth_smoothing_spring":0,"polar_angle_smoothing_spring":0,"distance_smoothing_spring":0,"polar_angle_min":0.1,"polar_angle_max":179.9,"radius":4},"minecraft:camera_avoidance":{"relax_distance_smoothing_spring":0,"distance_constraint_min":0.25},"minecraft:camera_perspective_option":{"view_mode":"third_person"},"minecraft:update_player_from_camera":{"look_mode":"along_camera"},"minecraft:camera_player_sleep_vignette":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{},"minecraft:extend_player_rendering":{}}}}"#;
const CUSTOM_THIRD_PERSON_FRONT_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:third_person_front"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_third_person":{},"minecraft:camera_render_player_model":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,2,5]},"minecraft:camera_look_at_player":{},"minecraft:camera_orbit":{"azimuth_smoothing_spring":0,"polar_angle_smoothing_spring":0,"distance_smoothing_spring":0,"polar_angle_min":0.1,"polar_angle_max":179.9,"radius":4,"invert_x_input":true},"minecraft:camera_avoidance":{"relax_distance_smoothing_spring":0,"distance_constraint_min":0.25},"minecraft:camera_perspective_option":{"view_mode":"third_person_front"},"minecraft:update_player_from_camera":{"look_mode":"at_camera"},"minecraft:camera_player_sleep_vignette":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{},"minecraft:extend_player_rendering":{}}}}"#;

const CUSTOM_LOADING_MESSAGES_JSON: &str = r#"{"beginner_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"mid_game_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"late_game_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"creative_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"editor_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"realms_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"addons_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"store_progress_tooltips":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"]}"#;

const CUSTOM_SKINS_JSON: &str = r#"{"skins":[{"localization_name":"Steve","geometry":"geometry.humanoid.custom","texture":"steve.png","type":"free"},{"localization_name":"Alex","geometry":"geometry.humanoid.customSlim","texture":"alex.png","type":"free"}],"serialize_name":"Standard","localization_name":"Standard"}"#;

const RENDER_JSON: &str = r#"{"format_version":"1.8.0","render_controllers":{"controller.render.player.cape":{"rebuild_animation_matrices":true,"geometry":"Geometry.cape","materials":[{"*":"Material.cape"}],"textures":["Texture.cape"]}}}"#;

const CLASSIC_STEVE_TEXTURE: &[u8] = include_bytes!("s.png");
const CLASSIC_ALEX_TEXTURE: &[u8] = include_bytes!("a.png");

const JAVA_CLOUDS_TEXTURE: &[u8] = include_bytes!("Diskksks.png");

fn get_current_mcver(man: ndk::asset::AssetManager) -> Option<MinecraftVersion> {
    let mut file = match get_uitext(man) {
        Some(asset) => asset,
        None => {
            log::error!("Shader fixing is disabled as no mc version was found");
            return None;
        }
    };
    let mut buf = Vec::with_capacity(file.length());
    if let Err(e) = file.read_to_end(&mut buf) {
        log::error!("Something is wrong with AssetManager, mc detection failed: {e}");
        return None;
    };
    for version in materialbin::ALL_VERSIONS {
        if buf
            .pread_with::<CompiledMaterialDefinition>(0, version)
            .is_ok()
        {
            log::info!("Mc version is {version}");
            return Some(version);
        };
    }
    None
}

fn get_uitext(man: ndk::asset::AssetManager) -> Option<Asset> {
    const NEW: &CStr = c"assets/renderer/materials/UIText.material.bin";
    const OLD: &CStr = c"renderer/materials/UIText.material.bin";
    for path in [NEW, OLD] {
        if let Some(asset) = man.open(path) {
            return Some(asset);
        }
    }
    None
}

macro_rules! folder_list {
    ($( apk: $apk_folder:literal -> pack: $pack_folder:expr),
        *,
    ) => {
        [
            $(($apk_folder, $pack_folder)),*,
        ]
    }
}

fn get_no_fog_material_data(filename: &str) -> Option<&'static [u8]> {
    if !is_no_fog_enabled() {
        return None;
    }
    
    match filename {
        "RenderChunk.material.bin" => Some(RENDER_CHUNK_MATERIAL_BIN),
        _ => None,
    }
}

fn get_nightvision_material_data(filename: &str) -> Option<&'static [u8]> {
    if !is_night_vision_enabled() {
        return None;
    }
    
    match filename {
        "RenderChunk.material.bin" => Some(RENDER_CHUNK_NV_MATERIAL_BIN),
        _ => None,
    }
}

fn get_java_cubemap_material_data(filename: &str) -> Option<&'static [u8]> {
    if !is_java_cubemap_enabled() {
        return None;
    }
    
    match filename {
        "LegacyCubemap.material.bin" => Some(LEGACY_CUBEMAP_MATERIAL_BIN),
        _ => None,
    }
}

fn get_title_png_data(filename: &str) -> Option<&'static [u8]> {
    if !is_xelo_title_enabled() {
        return None;
    }
    
    match filename {
        "title.png" => Some(TITLE_PNG),
        _ => None,
    }
}

// Enhanced particles disabler - blocks entire particles folder and all particle-related files
fn is_particles_folder_to_block(c_path: &Path) -> bool {
    if !is_particles_disabler_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    
    // Block any file related to particles
    let particle_patterns = [
        "/particles/",
        "particles/",
        "/particle/",
        "particle/",
        "_particle",
        "particle_",
        ".particle.",
        "particles.",
        "/effects/",
        "effects/",
        "_effect",
        "effect_",
        ".effect.",
        "effects.",
    ];
    
    particle_patterns.iter().any(|pattern| {
        path_str.contains(pattern)
    }) || path_str.starts_with("particles") || path_str.ends_with(".particle") || path_str.ends_with("_particle.json")
}

fn is_cape_invisible_texture_file(c_path: &Path) -> bool {
    if !is_client_capes_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    
    // Check for cape_invisible texture in various possible locations
    let cape_invisible_patterns = [
        "textures/entity/cape_invisible.png",
        "/textures/entity/cape_invisible.png",
        "textures/entity/cape_invisible",
        "/textures/entity/cape_invisible",
        "entity/cape_invisible.png",
        "/entity/cape_invisible.png",
        "entity/cape_invisible",
        "/entity/cape_invisible",
        "resource_packs/vanilla/textures/entity/cape_invisible.png",
        "assets/resource_packs/vanilla/textures/entity/cape_invisible.png",
        "vanilla/textures/entity/cape_invisible.png",
        "resource_packs/vanilla/textures/entity/cape_invisible",
        "assets/resource_packs/vanilla/textures/entity/cape_invisible",
        "vanilla/textures/entity/cape_invisible",
    ];
    
    cape_invisible_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}
// Enhanced clouds detection with more patterns
fn is_clouds_texture_file(c_path: &Path) -> bool {
    if !is_java_clouds_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    
    let cloud_patterns = [
        "textures/environment/clouds.png",
        "/textures/environment/clouds.png",
        "environment/clouds.png",
        "/environment/clouds.png",
        "clouds.png",
        "textures/clouds.png",
        "/textures/clouds.png",
        "resource_packs/vanilla/textures/environment/clouds.png",
        "assets/resource_packs/vanilla/textures/environment/clouds.png",
        "vanilla/textures/environment/clouds.png",
    ];
    
    cloud_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

fn is_skin_file_path(c_path: &Path, filename: &str) -> bool {
    let path_str = c_path.to_string_lossy();
    
    let possible_paths = [
        format!("vanilla/{}", filename),
        format!("skin_packs/vanilla/{}", filename),
        format!("resource_packs/vanilla/{}", filename),
        format!("assets/skin_packs/vanilla/{}", filename),
    ];
    
    possible_paths.iter().any(|path| {
        path_str.contains(path) || path_str.ends_with(path)
    })
}

fn is_classic_skins_steve_texture_file(c_path: &Path) -> bool {
    if !is_classic_skins_enabled() {
        return false;
    }
    
    is_skin_file_path(c_path, "steve.png")
}

fn is_classic_skins_alex_texture_file(c_path: &Path) -> bool {
    if !is_classic_skins_enabled() {
        return false;
    }
    
    is_skin_file_path(c_path, "alex.png")
}

fn is_classic_skins_json_file(c_path: &Path) -> bool {
    if !is_classic_skins_enabled() {
        return false;
    }
    
    is_skin_file_path(c_path, "skins.json")
}

fn is_client_capes_file(c_path: &Path) -> bool {
    if !is_client_capes_enabled() {
        return false;
    }
    
    is_skin_file_path(c_path, "cape.render_controllers.json")
}

fn is_persona_file_to_block(c_path: &Path) -> bool {
    if !is_classic_skins_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    
    let blocked_personas = [
        "persona/08_Kai_Dcast.json",
        "persona/07_Zuri_Dcast.json", 
        "persona/06_Efe_Dcast.json",
        "persona/05_Makena_Dcast.json",
        "persona/04_Sunny_Dcast.json",
        "persona/03_Ari_Dcast.json",
        "persona/02_ Noor_Dcast.json", 
    ];
    
    blocked_personas.iter().any(|persona_path| {
        path_str.contains(persona_path) || path_str.ends_with(persona_path)
    })
}

fn get_cape_model_data(filename: &str) -> Option<&'static [u8]> {
    if !is_cape_physics_enabled() {
        return None;
    }
    
    match filename {
        "mobs.json" => Some(MOBS_JSON),
        _ => None,
    }
}

fn get_cape_animation_data(filename: &str) -> Option<&'static [u8]> {
    if !is_cape_physics_enabled() {
        return None;
    }
    
    match filename {
        "player.animation.json" => Some(PLAYER_ANIMATION_JSON),
        _ => None,
    }
}

// Function to check if this is a player.entity.json file that needs client capes modification
fn is_player_entity_file(c_path: &Path) -> bool {
    if !is_client_capes_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    let os_filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };
    
    // Check if it's player.entity.json in various possible locations
    let player_entity_patterns = [
        "entity/player.entity.json",
        "/entity/player.entity.json",
        "entities/player.entity.json", 
        "/entities/player.entity.json",
        "resource_packs/vanilla/entity/player.entity.json",
        "assets/resource_packs/vanilla/entity/player.entity.json",
        "vanilla/entity/player.entity.json",
    ];
    
    os_filename == "player.entity.json" && player_entity_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

fn load_custom_cape_texture() -> Option<Vec<u8>> {
    match std::fs::read(CAPE_TEXTURE_PATH) {
        Ok(data) => {
            log::info!("Successfully loaded custom cape texture from: {}", CAPE_TEXTURE_PATH);
            Some(data)
        }
        Err(e) => {
            log::warn!("Failed to load custom cape texture from {}: {}", CAPE_TEXTURE_PATH, e);
            log::info!("Make sure xelo_cape.png exists in the origin_mods folder");
            None
        }
    }
}
// Function to modify player.entity.json to add cape render controller
fn modify_player_entity_json(original_data: &[u8]) -> Option<Vec<u8>> {
    let json_str = match std::str::from_utf8(original_data) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to parse player.entity.json as UTF-8: {}", e);
            return None;
        }
    };
    
    let mut json_value: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse player.entity.json as JSON: {}", e);
            return None;
        }
    };
    
    // Navigate to the render_controllers array
    if let Some(client_entity) = json_value
        .get_mut("minecraft:client_entity")
        .and_then(|ce| ce.as_object_mut())
    {
        if let Some(description) = client_entity
            .get_mut("description")
            .and_then(|desc| desc.as_object_mut())
        {
            if let Some(render_controllers) = description
                .get_mut("render_controllers")
                .and_then(|rc| rc.as_array_mut())
            {
                // Create the cape render controller object
                let cape_controller = serde_json::json!({
                    "controller.render.player.cape": "(query.armor_texture_slot(1) != 5) && (!variable.is_first_person || variable.is_paperdoll) && (!variable.map_face_icon)"
                });
                
                // Check if cape controller already exists
                let cape_exists = render_controllers.iter().any(|controller| {
                    controller.as_object()
                        .map(|obj| obj.contains_key("controller.render.player.cape"))
                        .unwrap_or(false)
                });
                
                if !cape_exists {
                    render_controllers.push(cape_controller);
                    log::info!("Added cape render controller to player.entity.json");
                } else {
                    log::info!("Cape render controller already exists in player.entity.json");
                }
            } else {
                log::error!("render_controllers array not found in player.entity.json");
                return None;
            }
        } else {
            log::error!("description object not found in player.entity.json");
            return None;
        }
    } else {
        log::error!("minecraft:client_entity not found in player.entity.json");
        return None;  
    }
    
    // Convert back to JSON string
    match serde_json::to_string_pretty(&json_value) {
        Ok(modified_json) => Some(modified_json.into_bytes()),
        Err(e) => {
            log::error!("Failed to serialize modified player.entity.json: {}", e);
            None
        }
    }
}

pub(crate) unsafe fn open(
    man: *mut AAssetManager,
    fname: *const libc::c_char,
    mode: libc::c_int,
) -> *mut ndk_sys::AAsset {
    let aasset = unsafe { ndk_sys::AAssetManager_open(man, fname, mode) };
    let c_str = unsafe { CStr::from_ptr(fname) };
    let raw_cstr = c_str.to_bytes();
    let os_str = OsStr::from_bytes(raw_cstr);
    let c_path: &Path = Path::new(os_str);
    
    let Some(os_filename) = c_path.file_name() else {
        log::warn!("Path had no filename: {c_path:?}");
        return aasset;
    };

    // Debug logging for features
    
    if is_particles_disabler_enabled() {
        let path_str = c_path.to_string_lossy();
        if path_str.contains("particle") || path_str.contains("effect") {
            log::info!("Particles disabler enabled - checking file: {}", c_path.display());
        }
    }
    
    if is_cape_invisible_texture_file(c_path) {
        log::info!("Intercepting cape_invisible texture with custom cape: {}", c_path.display());
        
        if let Some(custom_cape_data) = load_custom_cape_texture() {
            let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
            wanted_lock.insert(AAssetPtr(aasset), Cursor::new(custom_cape_data));
            return aasset;
        } else {
            log::warn!("Custom cape texture not found, using original cape_invisible");
            // Continue with original file
        }
    }

    // Block persona files if classic skins enabled
    if is_persona_file_to_block(c_path) {
        log::info!("Blocking persona file due to classic_skins enabled: {}", c_path.display());
        if !aasset.is_null() {
            ndk_sys::AAsset_close(aasset);
        }
        return std::ptr::null_mut();
    }

    // Block entire particles folder if particles disabler enabled
    if is_particles_folder_to_block(c_path) {
        log::info!("Blocking particles file due to particles_disabler enabled: {}", c_path.display());
        if !aasset.is_null() {
            ndk_sys::AAsset_close(aasset);
        }
        return std::ptr::null_mut();
    }
    
    // Client capes - modify player.entity.json
    if is_player_entity_file(c_path) {
        log::info!("Intercepting player.entity.json with client capes modification: {}", c_path.display());
        
        // Read the original file first
        if aasset.is_null() {
            log::error!("Failed to open original player.entity.json");
            return aasset;
        }
        
        let length = ndk_sys::AAsset_getLength(aasset) as usize;
        let mut original_data = vec![0u8; length];
        let bytes_read = ndk_sys::AAsset_read(aasset, original_data.as_mut_ptr() as *mut libc::c_void, length);
        
        if bytes_read != length as i32 {
            log::error!("Failed to read original player.entity.json completely");
            return aasset;
        }
        
        // Reset the asset position for normal operation
        ndk_sys::AAsset_seek(aasset, 0, libc::SEEK_SET);
        
        if let Some(modified_data) = modify_player_entity_json(&original_data) {
            let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
            wanted_lock.insert(AAssetPtr(aasset), Cursor::new(modified_data));
            return aasset;
        } else {
            log::warn!("Failed to modify player.entity.json, using original");
            return aasset;
        }
    }
    
    // Custom splashes
    if os_filename == "splashes.json" {
        log::info!("Intercepting splashes.json with custom content");
        let buffer = CUSTOM_SPLASHES_JSON.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    // Custom loading messages
    if os_filename == "loading_messages.json" {
        log::info!("Intercepting loading_messages.json with custom content");
        let buffer = CUSTOM_LOADING_MESSAGES_JSON.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    // Java clouds texture replacement
    if is_clouds_texture_file(c_path) {
        log::info!("Intercepting clouds texture with Java clouds texture: {}", c_path.display());
        let buffer = JAVA_CLOUDS_TEXTURE.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }

    // Classic skins replacements
    if is_classic_skins_steve_texture_file(c_path) {
        log::info!("Intercepting steve.png with classic Steve texture: {}", c_path.display());
        let buffer = CLASSIC_STEVE_TEXTURE.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_classic_skins_alex_texture_file(c_path) {
        log::info!("Intercepting alex.png with classic Alex texture: {}", c_path.display());
        let buffer = CLASSIC_ALEX_TEXTURE.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_classic_skins_json_file(c_path) {
        log::info!("Intercepting skins.json with classic skins content: {}", c_path.display());
        let buffer = RENDER_JSON.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_client_capes_file(c_path) {
        log::info!("Intercepting cape.render_controllers.json with cape content: {}", c_path.display());
        let buffer = CUSTOM_SKINS_JSON.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    // No hurt cam camera replacements
    if is_no_hurt_cam_enabled() {
        let path_str = c_path.to_string_lossy();
        
        if path_str.contains("cameras/") {
            if os_filename == "first_person.json" {
                log::info!("Intercepting cameras/first_person.json with custom content (nohurtcam enabled)");
                let buffer = CUSTOM_FIRST_PERSON_JSON.as_bytes().to_vec();
                let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
                wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
                return aasset;
            }
            
            if os_filename == "third_person.json" {
                log::info!("Intercepting cameras/third_person.json with custom content (nohurtcam enabled)");
                let buffer = CUSTOM_THIRD_PERSON_JSON.as_bytes().to_vec();
                let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
                wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
                return aasset;
            }
            
            if os_filename == "third_person_front.json" {
                log::info!("Intercepting cameras/third_person_front.json with custom content (nohurtcam enabled)");
                let buffer = CUSTOM_THIRD_PERSON_FRONT_JSON.as_bytes().to_vec();
                let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
                wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
                return aasset;
            }
        }
    }

    // Material replacements
    let filename_str = os_filename.to_string_lossy();
    if let Some(no_fog_data) = get_no_fog_material_data(&filename_str) {
        log::info!("Intercepting {} with no-fog material (no-fog enabled)", filename_str);
        let buffer = no_fog_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if let Some(night_vision_data) = get_nightvision_material_data(&filename_str) {
        log::info!("Intercepting {} with night-vision material (night-vision enabled)", filename_str);
        let buffer = night_vision_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if let Some(cape_physics_animation_data) = get_cape_animation_data(&filename_str) {
        log::info!("Intercepting {} with cape-physics animation (cape-physics enabled)", filename_str);
        let buffer = cape_physics_animation_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if let Some(cape_physics_model_data) = get_cape_model_data(&filename_str) {
        log::info!("Intercepting {} with cape-physics model (cape-physics enabled)", filename_str);
        let buffer = cape_physics_model_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if let Some(java_cubemap_data) = get_java_cubemap_material_data(&filename_str) {
        log::info!("Intercepting {} with java-cubemap material (java-cubemap enabled)", filename_str);
        let buffer = java_cubemap_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if let Some(title_png_data) = get_title_png_data(&filename_str) {
        log::info!("Intercepting {} with xelo title png (xelo-title enabled)", filename_str);
        let buffer = title_png_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }

    // Resource pack loading logic
    let stripped = match c_path.strip_prefix("assets/") {
        Ok(yay) => yay,
        Err(_e) => c_path,
    };
    
    let replacement_list = folder_list! {
        apk: "gui/dist/hbui/" -> pack: "hbui/",
        apk: "skin_packs/persona/" -> pack: "persona/",
        apk: "renderer/" -> pack: "renderer/",
        apk: "resource_packs/vanilla/cameras/" -> pack: "vanilla_cameras/",
    };
    
    for replacement in replacement_list {
        if let Ok(file) = stripped.strip_prefix(replacement.0) {
            cxx::let_cxx_string!(cxx_out = "");
            let loadfn = match crate::RPM_LOAD.get() {
                Some(ptr) => ptr,
                None => {
                    log::warn!("ResourcePackManager fn is not ready yet?");
                    return aasset;
                }
            };
            let mut arraybuf = [0; 128];
            let file_path = opt_path_join(&mut arraybuf, &[Path::new(replacement.1), file]);
            let packm_ptr = crate::PACKM_OBJ.load(std::sync::atomic::Ordering::Acquire);
            let resource_loc = ResourceLocation::from_str(file_path.as_ref());
            log::info!("loading rpck file: {:#?}", &file_path);
            if packm_ptr.is_null() {
                log::error!("ResourcePackManager ptr is null");
                return aasset;
            }
            loadfn(packm_ptr, resource_loc, cxx_out.as_mut());
            if cxx_out.is_empty() {
                log::info!("File was not found");
                return aasset;
            }
            let buffer = if os_filename.as_encoded_bytes().ends_with(b".material.bin") {
                match process_material(man, cxx_out.as_bytes()) {
                    Some(updated) => updated,
                    None => cxx_out.as_bytes().to_vec(),
                }
            } else {
                cxx_out.as_bytes().to_vec()
            };
            let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
            wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
            return aasset;
        }
    }
    return aasset;
}

fn opt_path_join<'a>(bytes: &'a mut [u8; 128], paths: &[&Path]) -> Cow<'a, CStr> {
    let total_len: usize = paths.iter().map(|p| p.as_os_str().len()).sum();
    if total_len + 1 > 128 {
        let mut pathbuf = PathBuf::new();
        for path in paths {
            pathbuf.push(path);
        }
        let cpath = CString::new(pathbuf.into_os_string().as_encoded_bytes()).unwrap();
        return Cow::Owned(cpath);
    }

    let mut writer = bytes.as_mut_slice();
    for path in paths {
        let osstr = path.as_os_str().as_bytes();
        let _ = writer.write(osstr);
    }
    let _ = writer.write(&[0]);
    let guh = CStr::from_bytes_until_nul(bytes).unwrap();
    Cow::Borrowed(guh)
}

fn process_material(man: *mut AAssetManager, data: &[u8]) -> Option<Vec<u8>> {
    let mcver = MC_VERSION.get_or_init(|| {
        let pointer = match std::ptr::NonNull::new(man) {
            Some(yay) => yay,
            None => {
                log::warn!("AssetManager is null?, preposterous, mc detection failed");
                return None;
            }
        };
        let manager = unsafe { ndk::asset::AssetManager::from_ptr(pointer) };
        get_current_mcver(manager)
    });
    let mcver = (*mcver)?;
    for version in materialbin::ALL_VERSIONS {
        let material: CompiledMaterialDefinition = match data.pread_with(0, version) {
            Ok(data) => data,
            Err(e) => {
                log::trace!("[version] Parsing failed: {e}");
                continue;
            }
        };
        if version == mcver {
            return None;
        }
        let mut output = Vec::with_capacity(data.len());
        if let Err(e) = material.write(&mut output, mcver) {
            log::trace!("[version] Write error: {e}");
            return None;
        }
        return Some(output);
    }

    None
}

pub(crate) unsafe fn seek64(aasset: *mut AAsset, off: off64_t, whence: libc::c_int) -> off64_t {
    let mut wanted_assets = WANTED_ASSETS.lock().unwrap();
    let file = match wanted_assets.get_mut(&AAssetPtr(aasset)) {
        Some(file) => file,
        None => return ndk_sys::AAsset_seek64(aasset, off, whence),
    };
    seek_facade(off, whence, file) as off64_t
}

pub(crate) unsafe fn seek(aasset: *mut AAsset, off: off_t, whence: libc::c_int) -> off_t {
    let mut wanted_assets = WANTED_ASSETS.lock().unwrap();
    let file = match wanted_assets.get_mut(&AAssetPtr(aasset)) {
        Some(file) => file,
        None => return ndk_sys::AAsset_seek(aasset, off, whence),
    };
    seek_facade(off.into(), whence, file) as off_t
}

pub(crate) unsafe fn read(
    aasset: *mut AAsset,
    buf: *mut libc::c_void,
    count: libc::size_t,
) -> libc::c_int {
    let mut wanted_assets = WANTED_ASSETS.lock().unwrap();
    let file = match wanted_assets.get_mut(&AAssetPtr(aasset)) {
        Some(file) => file,
        None => return ndk_sys::AAsset_read(aasset, buf, count),
    };
    let rs_buffer = core::slice::from_raw_parts_mut(buf as *mut u8, count);
    let read_total = match file.read(rs_buffer) {
        Ok(n) => n,
        Err(e) => {
            log::warn!("failed fake aaset read: {e}");
            return -1 as libc::c_int;
        }
    };
    read_total as libc::c_int
}

pub(crate) unsafe fn len(aasset: *mut AAsset) -> off_t {
    let wanted_assets = WANTED_ASSETS.lock().unwrap();
    let file = match wanted_assets.get(&AAssetPtr(aasset)) {
        Some(file) => file,
        None => return ndk_sys::AAsset_getLength(aasset),
    };
    file.get_ref().len() as off_t
}

pub(crate) unsafe fn len64(aasset: *mut AAsset) -> off64_t {
    let wanted_assets = WANTED_ASSETS.lock().unwrap();
    let file = match wanted_assets.get(&AAssetPtr(aasset)) {
        Some(file) => file,
        None => return ndk_sys::AAsset_getLength64(aasset),
    };
    file.get_ref().len() as off64_t
}

pub(crate) unsafe fn rem(aasset: *mut AAsset) -> off_t {
    let wanted_assets = WANTED_ASSETS.lock().unwrap();
    let file = match wanted_assets.get(&AAssetPtr(aasset)) {
        Some(file) => file,
        None => return ndk_sys::AAsset_getRemainingLength(aasset),
    };
    (file.get_ref().len() - file.position() as usize) as off_t
}

pub(crate) unsafe fn rem64(aasset: *mut AAsset) -> off64_t {
    let wanted_assets = WANTED_ASSETS.lock().unwrap();
    let file = match wanted_assets.get(&AAssetPtr(aasset)) {
        Some(file) => file,
        None => return ndk_sys::AAsset_getRemainingLength64(aasset),
    };
    (file.get_ref().len() - file.position() as usize) as off64_t
}

pub(crate) unsafe fn close(aasset: *mut AAsset) {
    let mut wanted_assets = WANTED_ASSETS.lock().unwrap();
    if wanted_assets.remove(&AAssetPtr(aasset)).is_none() {
        ndk_sys::AAsset_close(aasset);
    }
}

pub(crate) unsafe fn get_buffer(aasset: *mut AAsset) -> *const libc::c_void {
    let mut wanted_assets = WANTED_ASSETS.lock().unwrap();
    let file = match wanted_assets.get_mut(&AAssetPtr(aasset)) {
        Some(file) => file,
        None => return ndk_sys::AAsset_getBuffer(aasset),
    };
    file.get_mut().as_mut_ptr().cast()
}

pub(crate) unsafe fn fd_dummy(
    aasset: *mut AAsset,
    out_start: *mut off_t,
    out_len: *mut off_t,
) -> libc::c_int {
    let wanted_assets = WANTED_ASSETS.lock().unwrap();
    match wanted_assets.get(&AAssetPtr(aasset)) {
        Some(_) => {
            log::error!("WE GOT BUSTED NOOO");
            -1
        }
        None => ndk_sys::AAsset_openFileDescriptor(aasset, out_start, out_len),
    }
}

pub(crate) unsafe fn fd_dummy64(
    aasset: *mut AAsset,
    out_start: *mut off64_t,
    out_len: *mut off64_t,
) -> libc::c_int {
    let wanted_assets = WANTED_ASSETS.lock().unwrap();
    match wanted_assets.get(&AAssetPtr(aasset)) {
        Some(_) => {
            log::error!("WE GOT BUSTED NOOO");
            -1
        }
        None => ndk_sys::AAsset_openFileDescriptor64(aasset, out_start, out_len),
    }
}

pub(crate) unsafe fn is_alloc(aasset: *mut AAsset) -> libc::c_int {
    let wanted_assets = WANTED_ASSETS.lock().unwrap();
    match wanted_assets.get(&AAssetPtr(aasset)) {
        Some(_) => false as libc::c_int,
        None => ndk_sys::AAsset_isAllocated(aasset),
    }
}

fn seek_facade(offset: i64, whence: libc::c_int, file: &mut Cursor<Vec<u8>>) -> i64 {
    let offset = match whence {
        libc::SEEK_SET => {
            let u64_off = match u64::try_from(offset) {
                Ok(uoff) => uoff,
                Err(e) => {
                    log::error!("signed ({offset}) to unsigned failed: {e}");
                    return -1;
                }
            };
            io::SeekFrom::Start(u64_off)
        }
        libc::SEEK_CUR => io::SeekFrom::Current(offset),
        libc::SEEK_END => io::SeekFrom::End(offset),
        _ => {
            log::error!("Invalid seek whence");
            return -1;
        }
    };
    match file.seek(offset) {
        Ok(new_offset) => match new_offset.try_into() {
            Ok(int) => int,
            Err(err) => {
                log::error!("u64 ({new_offset}) to i64 failed: {err}");
                -1
            }
        },
        Err(err) => {
            log::error!("aasset seek failed: {err}");
            -1
        }
    }
}