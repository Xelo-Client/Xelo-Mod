use crate::ResourceLocation;
use crate::config::{is_no_hurt_cam_enabled, is_no_fog_enabled, is_java_cubemap_enabled, is_particles_disabler_enabled, is_java_clouds_enabled, is_classic_skins_enabled, is_cape_physics_enabled};
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

const CUSTOM_SPLASHES_JSON: &str = r#"{"splashes":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"]}"#;

// Cape geometry data as JSON Value for easy insertion
const CUSTOM_CAPE_GEOMETRY_JSON: &str = r#"{"format_version":"1.12.0","minecraft:geometry":[{"description":{"identifier":"geometry.cape","texture_width":64,"texture_height":32,"visible_bounds_width":2,"visible_bounds_height":3.5,"visible_bounds_offset":[0,1.25,0]},"bones":[{"name":"root","pivot":[0,0,0]},{"name":"waist","parent":"root","pivot":[0,12,0]},{"name":"body","parent":"waist","pivot":[0,24,0]},{"name":"cape","parent":"body","pivot":[0,24,2],"rotation":[0,180,0]},{"name":"part1","parent":"cape","pivot":[0,24,2],"cubes":[{"origin":[-5,23,1],"size":[10,1,1],"uv":{"north":{"uv":[1,1],"uv_size":[10,1]},"east":{"uv":[0,1],"uv_size":[1,1]},"south":{"uv":[12,1],"uv_size":[10,1]},"west":{"uv":[11,1],"uv_size":[1,1]},"up":{"uv":[1,1],"uv_size":[10,-1]}}}]},{"name":"part2","parent":"part1","pivot":[0,23,1],"cubes":[{"origin":[-5,22,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,1.5],"uv_size":[10,1.5]},"east":{"uv":[0,1.5],"uv_size":[1,1.5]},"south":{"uv":[12,1.5],"uv_size":[10,1.5]},"west":{"uv":[11,1.5],"uv_size":[1,1.5]}}}]},{"name":"part3","parent":"part2","pivot":[0,22,1],"cubes":[{"origin":[-5,21,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,2.5],"uv_size":[10,1.5]},"east":{"uv":[0,2.5],"uv_size":[1,1.5]},"south":{"uv":[12,2.5],"uv_size":[10,1.5]},"west":{"uv":[11,2.5],"uv_size":[1,1.5]}}}]},{"name":"part4","parent":"part3","pivot":[0,21,1],"cubes":[{"origin":[-5,20,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,3.5],"uv_size":[10,1.5]},"east":{"uv":[0,3.5],"uv_size":[1,1.5]},"south":{"uv":[12,3.5],"uv_size":[10,1.5]},"west":{"uv":[11,3.5],"uv_size":[1,1.5]}}}]},{"name":"part5","parent":"part4","pivot":[0,20,1],"cubes":[{"origin":[-5,19,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,4.5],"uv_size":[10,1.5]},"east":{"uv":[0,4.5],"uv_size":[1,1.5]},"south":{"uv":[12,4.5],"uv_size":[10,1.5]},"west":{"uv":[11,4.5],"uv_size":[1,1.5]}}}]},{"name":"part6","parent":"part5","pivot":[0,19,1],"cubes":[{"origin":[-5,18,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,5.5],"uv_size":[10,1.5]},"east":{"uv":[0,5.5],"uv_size":[1,1.5]},"south":{"uv":[12,5.5],"uv_size":[10,1.5]},"west":{"uv":[11,5.5],"uv_size":[1,1.5]}}}]},{"name":"part7","parent":"part6","pivot":[0,18,1],"cubes":[{"origin":[-5,17,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,6.5],"uv_size":[10,1.5]},"east":{"uv":[0,6.5],"uv_size":[1,1.5]},"south":{"uv":[12,6.5],"uv_size":[10,1.5]},"west":{"uv":[11,6.5],"uv_size":[1,1.5]}}}]},{"name":"part8","parent":"part7","pivot":[0,17,1],"cubes":[{"origin":[-5,16,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,7.5],"uv_size":[10,1.5]},"east":{"uv":[0,7.5],"uv_size":[1,1.5]},"south":{"uv":[12,7.5],"uv_size":[10,1.5]},"west":{"uv":[11,7.5],"uv_size":[1,1.5]}}}]},{"name":"part9","parent":"part8","pivot":[0,16,1],"cubes":[{"origin":[-5,15,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,8.5],"uv_size":[10,1.5]},"east":{"uv":[0,8.5],"uv_size":[1,1.5]},"south":{"uv":[12,8.5],"uv_size":[10,1.5]},"west":{"uv":[11,8.5],"uv_size":[1,1.5]}}}]},{"name":"part10","parent":"part9","pivot":[0,15,1],"cubes":[{"origin":[-5,14,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,9.5],"uv_size":[10,1.5]},"east":{"uv":[0,9.5],"uv_size":[1,1.5]},"south":{"uv":[12,9.5],"uv_size":[10,1.5]},"west":{"uv":[11,9.5],"uv_size":[1,1.5]}}}]},{"name":"part11","parent":"part10","pivot":[0,14,1],"cubes":[{"origin":[-5,13,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,10.5],"uv_size":[10,1.5]},"east":{"uv":[0,10.5],"uv_size":[1,1.5]},"south":{"uv":[12,10.5],"uv_size":[10,1.5]},"west":{"uv":[11,10.5],"uv_size":[1,1.5]}}}]},{"name":"part12","parent":"part11","pivot":[0,13,1],"cubes":[{"origin":[-5,12,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,11.5],"uv_size":[10,1.5]},"east":{"uv":[0,11.5],"uv_size":[1,1.5]},"south":{"uv":[12,11.5],"uv_size":[10,1.5]},"west":{"uv":[11,11.5],"uv_size":[1,1.5]}}}]},{"name":"part13","parent":"part12","pivot":[0,12,1],"cubes":[{"origin":[-5,11,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,12.5],"uv_size":[10,1.5]},"east":{"uv":[0,12.5],"uv_size":[1,1.5]},"south":{"uv":[12,12.5],"uv_size":[10,1.5]},"west":{"uv":[11,12.5],"uv_size":[1,1.5]}}}]},{"name":"part14","parent":"part13","pivot":[0,11,1],"cubes":[{"origin":[-5,10,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,13.5],"uv_size":[10,1.5]},"east":{"uv":[0,13.5],"uv_size":[1,1.5]},"south":{"uv":[12,13.5],"uv_size":[10,1.5]},"west":{"uv":[11,13.5],"uv_size":[1,1.5]}}}]},{"name":"part15","parent":"part14","pivot":[0,10,1],"cubes":[{"origin":[-5,9,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,14.5],"uv_size":[10,1.5]},"east":{"uv":[0,14.5],"uv_size":[1,1.5]},"south":{"uv":[12,14.5],"uv_size":[10,1.5]},"west":{"uv":[11,14.5],"uv_size":[1,1.5]}}}]},{"name":"part16","parent":"part15","pivot":[0,9,1],"cubes":[{"origin":[-5,8,1],"size":[10,1.5,1],"uv":{"north":{"uv":[1,15.5],"uv_size":[10,1.5]},"east":{"uv":[0,15.5],"uv_size":[1,1.5]},"south":{"uv":[12,15.5],"uv_size":[10,1.5]},"west":{"uv":[11,15.5],"uv_size":[1,1.5]},"down":{"uv":[11,1],"uv_size":[10,-1]}}}]}]}]}"#;

const CUSTOM_CAPE_ANIMATION_JSON: &str = r#"{"format_version":"1.8.0","animations":{"animation.player.cape":{"loop":true,"bones":{"cape":{"rotation":["math.clamp(math.lerp(0, -110, query.cape_flap_amount) - (13 * query.modified_move_speed), -70, 0)","query.modified_move_speed * math.pow(math.sin(query.body_y_rotation - query.head_y_rotation(0)), 3) * 55",0],"position":[0,0,"query.get_root_locator_offset('armor_offset.default_neck', 1)"]},"part1":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * (math.cos(query.modified_distance_moved * 18) * 16)",0,"0"]},"part2":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(22 - query.modified_distance_moved * 18) * 13",0,0],"scale":1},"part3":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(50 - query.modified_distance_moved * 18) * 13",0,0]},"part4":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(76 - query.modified_distance_moved * 18) * 13",0,0]},"part5":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(100 - query.modified_distance_moved * 18) * 13",0,0]},"part6":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(122 - query.modified_distance_moved * 18) * 13",0,0]},"part7":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(142 - query.modified_distance_moved * 18) * 13",0,0]},"part8":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(160 - query.modified_distance_moved * 18) * 13",0,0]},"part9":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(176 - query.modified_distance_moved * 18) * 13",0,0]},"part10":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(190 - query.modified_distance_moved * 18) * 13",0,0]},"part11":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(202 - query.modified_distance_moved * 18) * 13",0,0]},"part12":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(212 - query.modified_distance_moved * 18) * 13",0,0]},"part13":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(220 - query.modified_distance_moved * 18) * 13",0,0]},"part14":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(226 - query.modified_distance_moved * 18) * 13",0,0]},"part15":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(230 - query.modified_distance_moved * 18) * 13",0,0]},"part16":{"rotation":["math.clamp(query.cape_flap_amount, 0, 0.5) * math.cos(232 - query.modified_distance_moved * 18) * 13",0,0]},"shoulders":{"rotation":[0,"query.modified_move_speed * math.pow(math.sin(query.body_y_rotation - query.head_y_rotation(0)), 3) * 60",0]}}}}}"#;

const CUSTOM_FIRST_PERSON_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:first_person"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_first_person":{},"minecraft:camera_render_first_person_objects":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,0,0]},"minecraft:camera_direct_look":{"pitch_min":-89.9,"pitch_max":89.9},"minecraft:camera_perspective_option":{"view_mode":"first_person"},"minecraft:update_player_from_camera":{"look_mode":"along_camera"},"minecraft:extend_player_rendering":{},"minecraft:camera_player_sleep_vignette":{},"minecraft:vr_comfort_move":{},"minecraft:default_input_camera":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{}}}}"#;
const CUSTOM_THIRD_PERSON_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:third_person"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_third_person":{},"minecraft:camera_render_player_model":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,2,5]},"minecraft:camera_look_at_player":{},"minecraft:camera_orbit":{"azimuth_smoothing_spring":0,"polar_angle_smoothing_spring":0,"distance_smoothing_spring":0,"polar_angle_min":0.1,"polar_angle_max":179.9,"radius":4},"minecraft:camera_avoidance":{"relax_distance_smoothing_spring":0,"distance_constraint_min":0.25},"minecraft:camera_perspective_option":{"view_mode":"third_person"},"minecraft:update_player_from_camera":{"look_mode":"along_camera"},"minecraft:camera_player_sleep_vignette":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{},"minecraft:extend_player_rendering":{}}}}"#;
const CUSTOM_THIRD_PERSON_FRONT_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:third_person_front"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_third_person":{},"minecraft:camera_render_player_model":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,2,5]},"minecraft:camera_look_at_player":{},"minecraft:camera_orbit":{"azimuth_smoothing_spring":0,"polar_angle_smoothing_spring":0,"distance_smoothing_spring":0,"polar_angle_min":0.1,"polar_angle_max":179.9,"radius":4,"invert_x_input":true},"minecraft:camera_avoidance":{"relax_distance_smoothing_spring":0,"distance_constraint_min":0.25},"minecraft:camera_perspective_option":{"view_mode":"third_person_front"},"minecraft:update_player_from_camera":{"look_mode":"at_camera"},"minecraft:camera_player_sleep_vignette":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{},"minecraft:extend_player_rendering":{}}}}"#;

const CUSTOM_LOADING_MESSAGES_JSON: &str = r#"{"beginner_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"mid_game_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"late_game_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"creative_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"editor_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"realms_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"addons_loading_messages":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"],"store_progress_tooltips":["Origin Client","Origin > any other client","The Best Client!!","BlueCat","Origin is so much better","Origin Optimizes like no other client","Make Sure to star our repository:https://github.com/Origin-Client/Origin","Contributions open!","Made by the community, for the community","Yami is goated!!"]}"#;

const CUSTOM_SKINS_JSON: &str = r#"{"skins":[{"localization_name":"Steve","geometry":"geometry.humanoid.custom","texture":"steve.png","type":"free"},{"localization_name":"Alex","geometry":"geometry.humanoid.customSlim","texture":"alex.png","type":"free"}],"serialize_name":"Standard","localization_name":"Standard"}"#;

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

fn get_java_cubemap_material_data(filename: &str) -> Option<&'static [u8]> {
    if !is_java_cubemap_enabled() {
        return None;
    }
    
    match filename {
        "LegacyCubemap.material.bin" => Some(LEGACY_CUBEMAP_MATERIAL_BIN),
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

// Cape physics helper functions - Modified to patch existing files
fn is_mobs_json_file(c_path: &Path) -> bool {
    if !is_cape_physics_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    
    // Check for mobs.json in models folder
    let mobs_patterns = [
        "models/mobs.json",
        "/models/mobs.json",
        "vanilla/models/mobs.json",
        "resource_packs/vanilla/models/mobs.json",
        "assets/resource_packs/vanilla/models/mobs.json",
    ];
    
    mobs_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

fn is_player_animation_file(c_path: &Path) -> bool {
    if !is_cape_physics_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    
    // Check for player.animation.json in animations folder
    let animation_patterns = [
        "animations/player.animation.json",
        "/animations/player.animation.json",
        "vanilla/animations/player.animation.json",
        "resource_packs/vanilla/animations/player.animation.json",
        "assets/resource_packs/vanilla/animations/player.animation.json",
    ];
    
    animation_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

fn patch_mobs_json(original_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Parse the original mobs.json
    let mut json_value: Value = serde_json::from_slice(original_data)?;
    
    // Parse our custom cape geometry
    let cape_geometry: Value = serde_json::from_str(CUSTOM_CAPE_GEOMETRY_JSON)?;
    
    // Extract the cape geometry object from our custom data
    if let Some(geometries) = cape_geometry["minecraft:geometry"].as_array() {
        if let Some(cape_geo) = geometries.iter().find(|geo| {
            geo["description"]["identifier"].as_str() == Some("geometry.cape")
        }) {
            // Find and replace geometry.cape in the original mobs.json
            if let Some(original_geometries) = json_value["minecraft:geometry"].as_array_mut() {
                // Remove existing cape geometry if it exists
                original_geometries.retain(|geo| {
                    geo["description"]["identifier"].as_str() != Some("geometry.cape")
                });
                
                // Add our custom cape geometry
                original_geometries.push(cape_geo.clone());
                
                log::info!("Successfully patched geometry.cape in mobs.json");
            } else {
                // If minecraft:geometry doesn't exist, create it
                json_value["minecraft:geometry"] = serde_json::json!([cape_geo.clone()]);
                log::info!("Created minecraft:geometry array and added geometry.cape in mobs.json");
            }
        }
    }
    
    // Serialize back to JSON
    let patched_data = serde_json::to_vec_pretty(&json_value)?;
    Ok(patched_data)
}

fn patch_player_animation_json(original_data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Parse the original player.animation.json
    let mut json_value: Value = serde_json::from_slice(original_data)?;
    
    // Parse our custom cape animation
    let cape_animation: Value = serde_json::from_str(CUSTOM_CAPE_ANIMATION_JSON)?;
    
    // Extract the cape animation object from our custom data
    if let Some(cape_anim) = cape_animation["animations"]["animation.player.cape"].as_object() {
        // Ensure animations object exists
        if json_value["animations"].is_null() {
            json_value["animations"] = serde_json::json!({});
        }
        
        // Replace or add the animation.player.cape
        if let Some(animations) = json_value["animations"].as_object_mut() {
            animations.insert("animation.player.cape".to_string(), Value::Object(cape_anim.clone()));
            log::info!("Successfully patched animation.player.cape in player.animation.json");
        }
    }
    
    // Serialize back to JSON
    let patched_data = serde_json::to_vec_pretty(&json_value)?;
    Ok(patched_data)
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
    if is_cape_physics_enabled() {
        let path_str = c_path.to_string_lossy();
        if path_str.contains("cape") || path_str.contains("animation") || path_str.contains("entity") || path_str.contains("mobs.json") || path_str.contains("player.animation.json") {
            log::info!("Cape physics enabled - checking file: {}", c_path.display());
        }
    }
    
    if is_particles_disabler_enabled() {
        let path_str = c_path.to_string_lossy();
        if path_str.contains("particle") || path_str.contains("effect") {
            log::info!("Particles disabler enabled - checking file: {}", c_path.display());
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
    
    // Cape physics patching - Patch existing files instead of creating new ones
    if is_mobs_json_file(c_path) && !aasset.is_null() {
        log::info!("Cape physics: Patching mobs.json with custom cape geometry: {}", c_path.display());
        
        // Read the original file
        let original_length = ndk_sys::AAsset_getLength(aasset) as usize;
        let mut original_data = vec![0u8; original_length];
        let read_result = ndk_sys::AAsset_read(aasset, original_data.as_mut_ptr() as *mut libc::c_void, original_length);
        
        if read_result == original_length as i32 {
            match patch_mobs_json(&original_data) {
                Ok(patched_data) => {
                    let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
                    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(patched_data));
                    return aasset;
                }
                Err(e) => {
                    log::error!("Failed to patch mobs.json: {}", e);
                    // Return original asset on error
                }
            }
        } else {
            log::error!("Failed to read original mobs.json data");
        }
        
        // Reset asset position on error
        ndk_sys::AAsset_seek(aasset, 0, libc::SEEK_SET);
        return aasset;
    }
    
    if is_player_animation_file(c_path) && !aasset.is_null() {
        log::info!("Cape physics: Patching player.animation.json with custom cape animation: {}", c_path.display());
        
        // Read the original file
        let original_length = ndk_sys::AAsset_getLength(aasset) as usize;
        let mut original_data = vec![0u8; original_length];
        let read_result = ndk_sys::AAsset_read(aasset, original_data.as_mut_ptr() as *mut libc::c_void, original_length);
        
        if read_result == original_length as i32 {
            match patch_player_animation_json(&original_data) {
                Ok(patched_data) => {
                    let mut wanted_lock = WANTED_ASSETS.lock().unwrap();
                    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(patched_data));
                    return aasset;
                }
                Err(e) => {
                    log::error!("Failed to patch player.animation.json: {}", e);
                    // Return original asset on error
                }
            }
        } else {
            log::error!("Failed to read original player.animation.json data");
        }
        
        // Reset asset position on error
        ndk_sys::AAsset_seek(aasset, 0, libc::SEEK_SET);
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
    
    if let Some(java_cubemap_data) = get_java_cubemap_material_data(&filename_str) {
        log::info!("Intercepting {} with java-cubemap material (java-cubemap enabled)", filename_str);
        let buffer = java_cubemap_data.to_vec();
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