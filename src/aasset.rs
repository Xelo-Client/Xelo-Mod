//Explanation: Aasset is NOT thread-safe anyways so we will not try adding thread safety either
#![allow(static_mut_refs)]
use crate::{
    loader::{Buffer, FileLoader},
    LockResultExt,
};
use crate::config::{is_no_hurt_cam_enabled, is_particles_disabler_enabled, is_java_clouds_enabled, is_classic_skins_enabled, is_no_shadows_enabled, is_xelo_title_enabled, is_client_capes_enabled, is_block_whiteoutline_enabled, is_no_flipbook_animations_enabled, is_no_spyglass_overlay_enabled, is_no_pumpkin_overlay_enabled, is_double_tppview_enabled,
    is_custom_cross_hair_enabled, is_no_eating_animation, is_portal_optimizer, is_no_bow_animation, is_psm_enabled, is_no_weather_enabled, is_no_sunmoon_enabled, is_no_stars_enabled
};
use libc::{c_char, c_int, c_void, off64_t, off_t, size_t};
use ndk_sys::{AAsset, AAssetManager, acamera_metadata_enum_acamera_depth_available_depth_stream_configurations};
use once_cell::sync::Lazy;
use std::{
    cell::UnsafeCell, collections::HashMap, ffi::{CStr, CString, OsStr}, fs, io::{self, Cursor, Read, Seek, Write}, os::unix::ffi::OsStrExt, path::{Path, PathBuf}, sync::{Arc, LazyLock, Mutex, OnceLock}
};
use serde_json::{Value, Map};

static MC_FILELOADER: LazyLock<Mutex<FileLoader>> = LazyLock::new(|| Mutex::new(FileLoader::new()));
// This makes me feel wrong... but all we will do is compare the pointer
// and the struct will be used in a mutex so this is safe??
#[derive(PartialEq, Eq, Hash)]
struct AAssetPtr(*const ndk_sys::AAsset);
unsafe impl Send for AAssetPtr {}

// The assets we have registered to replace data about
static mut WANTED_ASSETS: LazyLock<UnsafeCell<HashMap<AAssetPtr, Buffer>>> =
    LazyLock::new(|| UnsafeCell::new(HashMap::new()));

static WANTED_ASSETS_MUTEX: Lazy<Mutex<HashMap<AAssetPtr, Cursor<Vec<u8>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
    
// Xelo constants start

pub const CUSTOM_CROSS_HAIR_PNG: &[u8] = include_bytes!("utils/custom_cross_hair/cross_hair.png");

pub const CUSTOM_CROSS_HAIR_PATH: &str = "/storage/emulated/0/games/xelo_client/custom_cross_hair/cross_hair.png";

const PACK_ICN_PNG: &[u8] = include_bytes!("assets/resources/pack_icon.png");

const CAPE_TEXTURE_PATH: &str = "/storage/emulated/0/Android/data/com.origin.launcher/files/origin_mods/xelo_cape.png";

const TITLE_PNG: &[u8] = include_bytes!("minecraft_title_5.png");

const CLEAR_PNG: &[u8] = include_bytes!("utils/clear/c.png");

const SHADOWS_MATERIAL: &[u8] = include_bytes!("optimizers/noshadows/shadows.material");

const CUSTOM_SPLASHES_JSON: &str = r#"{"splashes":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"]}"#;

const CUSTOM_FIRST_PERSON_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:first_person"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_first_person":{},"minecraft:camera_render_first_person_objects":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,0,0]},"minecraft:camera_direct_look":{"pitch_min":-89.9,"pitch_max":89.9},"minecraft:camera_perspective_option":{"view_mode":"first_person"},"minecraft:update_player_from_camera":{"look_mode":"along_camera"},"minecraft:extend_player_rendering":{},"minecraft:camera_player_sleep_vignette":{},"minecraft:vr_comfort_move":{},"minecraft:default_input_camera":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{}}}}"#;
const CUSTOM_THIRD_PERSON_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:third_person"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_third_person":{},"minecraft:camera_render_player_model":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,2,5]},"minecraft:camera_look_at_player":{},"minecraft:camera_orbit":{"azimuth_smoothing_spring":0,"polar_angle_smoothing_spring":0,"distance_smoothing_spring":0,"polar_angle_min":0.1,"polar_angle_max":179.9,"radius":8},"minecraft:camera_avoidance":{"relax_distance_smoothing_spring":0,"distance_constraint_min":0.25},"minecraft:camera_perspective_option":{"view_mode":"third_person"},"minecraft:update_player_from_camera":{"look_mode":"along_camera"},"minecraft:camera_player_sleep_vignette":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{},"minecraft:extend_player_rendering":{}}}}"#;
const CUSTOM_THIRD_PERSON_FRONT_JSON: &str = r#"{"format_version":"1.18.10","minecraft:camera_entity":{"description":{"identifier":"minecraft:third_person_front"},"components":{"minecraft:camera":{"field_of_view":66,"near_clipping_plane":0.025,"far_clipping_plane":2500},"minecraft:camera_third_person":{},"minecraft:camera_render_player_model":{},"minecraft:camera_attach_to_player":{},"minecraft:camera_offset":{"view":[0,0],"entity":[0,2,5]},"minecraft:camera_look_at_player":{},"minecraft:camera_orbit":{"azimuth_smoothing_spring":0,"polar_angle_smoothing_spring":0,"distance_smoothing_spring":0,"polar_angle_min":0.1,"polar_angle_max":179.9,"radius":4,"invert_x_input":true},"minecraft:camera_avoidance":{"relax_distance_smoothing_spring":0,"distance_constraint_min":0.25},"minecraft:camera_perspective_option":{"view_mode":"third_person_front"},"minecraft:update_player_from_camera":{"look_mode":"at_camera"},"minecraft:camera_player_sleep_vignette":{},"minecraft:gameplay_affects_fov":{},"minecraft:allow_inside_block":{},"minecraft:extend_player_rendering":{}}}}"#;

const CUSTOM_LOADING_MESSAGES_JSON: &str = r#"{"beginner_loading_messages":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"],"mid_game_loading_messages":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"],"late_game_loading_messages":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"],"creative_loading_messages":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"],"editor_loading_messages":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"],"realms_loading_messages":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"],"addons_loading_messages":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"],"store_progress_tooltips":["Xelo Client","Xelo > any other client","The Best Client!!","BlueCat","I am Viable","VCX","Xelo is so much better","Xelo Optimizes like no other client","Make Sure to star our repository: https://github.com/Xelo-Client/Xelo","Contributions open!","Made by the community, for the community","Yami is goated!!"]}"#;

const CUSTOM_SKINS_JSON: &str = r#"{"skins":[{"localization_name":"Steve","geometry":"geometry.humanoid.custom","texture":"steve.png","type":"free"},{"localization_name":"Alex","geometry":"geometry.humanoid.customSlim","texture":"alex.png","type":"free"}],"serialize_name":"Standard","localization_name":"Standard"}"#;

const CUSTOM_BLOCKOUTLINE: &str = r#"{"materials":{"block_overlay":{"+states":["Blending","DisableDepthWrite","DisableAlphaWrite","StencilWrite","EnableStencilTest"],"backFace":{"stencilDepthFailOp":"Keep","stencilFailOp":"Keep","stencilFunc":"NotEqual","stencilPassOp":"Replace"},"depthBias":100.0,"depthBiasOGL":100.0,"depthFunc":"LessEqual","fragmentShader":"shaders/texture_cutout.fragment","frontFace":{"stencilDepthFailOp":"Keep","stencilFailOp":"Keep","stencilFunc":"NotEqual","stencilPassOp":"Replace"},"msaaSupport":"Both","slopeScaledDepthBias":15.0,"slopeScaledDepthBiasOGL":20.0,"stencilReadMask":2,"stencilRef":2,"stencilWriteMask":2,"variants":[{"skinning":{"+defines":["USE_SKINNING"],"vertexFields":[{"field":"Position"},{"field":"BoneId0"},{"field":"UV1"},{"field":"UV0"}]}}],"vertexFields":[{"field":"Position"},{"field":"UV1"},{"field":"UV0"}],"vertexShader":"shaders/uv.vertex","vrGeometryShader":"shaders/uv.geometry"},"cracks_overlay:block_overlay":{"+samplerStates":[{"samplerIndex":0,"textureFilter":"Point"}],"blendDst":"Zero","blendSrc":"DestColor","depthFunc":"LessEqual","fragmentShader":"shaders/texture.fragment"},"cracks_overlay_alpha_test:cracks_overlay":{"+defines":["ALPHA_TEST"],"+states":["DisableCulling"]},"cracks_overlay_tile_entity:cracks_overlay":{"+samplerStates":[{"samplerIndex":0,"textureWrap":"Repeat"}],"variants":[{"skinning":{"+defines":["USE_SKINNING"],"vertexFields":[{"field":"Position"},{"field":"BoneId0"},{"field":"Normal"},{"field":"UV0"}]}}],"vertexFields":[{"field":"Position"},{"field":"Normal"},{"field":"UV0"}],"vertexShader":"shaders/uv_scale.vertex","vrGeometryShader":"shaders/uv.geometry"},"debug":{"depthFunc":"LessEqual","fragmentShader":"shaders/color.fragment","msaaSupport":"Both","vertexFields":[{"field":"Position"},{"field":"Color"}],"vertexShader":"shaders/color.vertex","vrGeometryShader":"shaders/color.geometry"},"fullscreen_cube_overlay":{"+samplerStates":[{"samplerIndex":0,"textureFilter":"Point"}],"depthFunc":"Always","fragmentShader":"shaders/texture_ccolor.fragment","msaaSupport":"Both","vertexFields":[{"field":"Position"},{"field":"UV0"}],"vertexShader":"shaders/uv.vertex","vrGeometryShader":"shaders/uv.geometry"},"fullscreen_cube_overlay_blend:fullscreen_cube_overlay":{"+states":["Blending"]},"fullscreen_cube_overlay_opaque:fullscreen_cube_overlay":{"+states":["DisableCulling"]},"lightning":{"+states":["DisableCulling","Blending"],"blendDst":"One","blendSrc":"SourceAlpha","depthFunc":"LessEqual","fragmentShader":"shaders/lightning.fragment","msaaSupport":"Both","vertexFields":[{"field":"Position"},{"field":"Color"}],"vertexShader":"shaders/color.vertex","vrGeometryShader":"shaders/color.geometry"},"name_tag":{"+states":["Blending","DisableDepthWrite"],"depthFunc":"Always","vertexShader":"shaders/position.vertex","vrGeometryShader":"shaders/position.geometry","fragmentShader":"shaders/current_color.fragment","vertexFields":[{ "field":"Position"}],"+samplerStates":[{"samplerIndex":0,"textureFilter":"Point"}],"msaaSupport":"Both"},"name_tag_depth_tested:name_tag":{"depthFunc":"LessEqual"},"name_tag_alpha_tested:name_tag":{"+defines":[ "ALPHA_TEST" ],"depthFunc":"LessEqual","-states":["Blending","DisableDepthWrite"]},    "name_tag_text":{"+states":["Blending"],"depthFunc":"Always","msaaSupport":"Both"},"name_text_depth_tested:sign_text":{},"overlay_quad":{"+samplerStates":[{"samplerIndex":0,"textureFilter":"Bilinear"}],"+states":["DisableCulling","DisableDepthWrite","Blending"],"blendDst":"OneMinusSrcAlpha","blendSrc":"SourceAlpha","depthFunc":"Always","fragmentShader":"shaders/texture_raw_alphatest.fragment","vertexFields":[{"field":"Position"},{"field":"UV0"}],"vertexShader":"shaders/uv.vertex","vrGeometryShader":"shaders/uv.geometry"},"overlay_quad_clear":{"depthFunc":"Always","fragmentShader":"shaders/color.fragment","msaaSupport":"Both","vertexFields":[{"field":"Position"}],"vertexShader":"shaders/simple.vertex","vrGeometryShader":"shaders/color.geometry"},"plankton:precipitation":{"+defines":["COMFORT_MODE","FLIP_OCCLUSION","NO_VARIETY"]},"precipitation":{"+defines":["COMFORT_MODE"],"+samplerStates":[{"samplerIndex":0,"textureFilter":"Point"},{"samplerIndex":1,"textureFilter":"Point"},{"samplerIndex":2,"textureFilter":"Bilinear"}],"+states":["DisableCulling","DisableDepthWrite","Blending"],"blendDst":"OneMinusSrcAlpha","blendSrc":"SourceAlpha","depthFunc":"LessEqual","fragmentShader":"shaders/rain_snow.fragment","msaaSupport":"Both","vertexFields":[{"field":"Position"},{"field":"Color"},{"field":"UV0"}],"vertexShader":"shaders/rain_snow.vertex","vrGeometryShader":"shaders/rain_snow.geometry"},"rain:precipitation":{},"selection_box":{"+defines":["LINE_STRIP"],"depthFunc":"LessEqual","fragmentShader":"shaders/selection_box.fragment","msaaSupport":"Both","primitiveMode":"Line","vertexFields":[{"field":"Position"}],"vertexShader":"shaders/selection_box.vertex","vrGeometryShader":"shaders/position.geometry"},"selection_overlay:block_overlay":{"blendDst":"SourceColor","blendSrc":"DestColor","vertexShader":"shaders/uv_selection_overlay.vertex"},"selection_overlay_alpha:selection_overlay_level":{"vertexFields":[{"field":"Position"},{"field":"UV1"},{"field":"UV0"}]},"selection_overlay_block_entity:selection_overlay":{"variants":[{"skinning":{"+defines":["USE_SKINNING"],"vertexFields":[{"field":"Position"},{"field":"BoneId0"},{"field":"Normal"},{"field":"UV0"}]},"skinning_color":{"+defines":["USE_SKINNING"],"vertexFields":[{"field":"Position"},{"field":"BoneId0"},{"field":"Color"},{"field":"Normal"},{"field":"UV0"}]}}],"vertexFields":[{"field":"Position"},{"field":"Normal"},{"field":"UV0"}]},"selection_overlay_double_sided:selection_overlay":{"+states":["DisableCulling"]},"selection_overlay_item:selection_overlay":{},"selection_overlay_level:selection_overlay":{"msaaSupport":"Both","vertexFields":[{"field":"Position"},{"field":"Normal"},{"field":"UV0"}]},"selection_overlay_opaque:selection_overlay":{"fragmentShader":"shaders/current_color.fragment","msaaSupport":"Both","vertexShader":"shaders/selection_box.vertex","vrGeometryShader":"shaders/position.geometry"},"sign_text":{"+defines":["ALPHA_TEST","USE_LIGHTING"],"+samplerStates":[{"samplerIndex":0,"textureFilter":"Point"}],"+states":["Blending"],"depthBias":10.0,"depthBiasOGL":10.0,"depthFunc":"LessEqual","fragmentShader":"shaders/text.fragment","msaaSupport":"Both","slopeScaledDepthBias":2.0,"slopeScaledDepthBiasOGL":10.0,"vertexFields":[{"field":"Position"},{"field":"Color"},{"field":"UV0"}],"vertexShader":"shaders/color_uv.vertex","vrGeometryShader":"shaders/color_uv.geometry"},"snow:precipitation":{"+defines":["SNOW"]},"version":"1.0.0"}}"#;

const CUSTOM_BOW_RENDER_CONTROLLER: &str = r#"{"format_version":"1.10","render_controllers":{"controller.render.bow":{"arrays":{"textures":{"array.bow_texture_frames":["texture.default"]},"geometries":{"array.bow_geo_frames":["geometry.default"]}},"geometry":"array.bow_geo_frames[query.get_animation_frame]","materials":[{"*":"variable.is_enchanted ? material.enchanted : material.default"}],"textures":["array.bow_texture_frames[query.get_animation_frame]","texture.enchanted"]}}}"#;

// Fixed render controller JSON with proper format and indentation
const RENDER_JSON: &str = r#"{
    "format_version": "1.8.0",
    "render_controllers": {
        "controller.render.player.cape": {
            "rebuild_animation_matrices": true,
            "geometry": "Geometry.cape",
            "materials": [
                {
                    "*": "Material.cape"
                }
            ],
            "textures": [
                "Texture.cape"
            ]
        }
    }
}"#;

const PSM_PARTICLES_MATERIAL: &str = r#"{
  "materials": {
    "version": "1.0.0",

    "particles_base": {
      "vertexShader": "shaders/color_uv.vertex",
      "vrGeometryShader": "shaders/color_uv.geometry",
      "fragmentShader": "shaders/color_texture.fragment",

      "vertexFields": [
        { "field": "Position" },
        { "field": "Color" },
        { "field": "UV0" }
      ],

      "+samplerStates": [
        {
          "samplerIndex": 0,
          "textureFilter": "Point"
        }
      ],

      "msaaSupport": "Both"
    },

    "particles_opaque:particles_base": {
      "+states": [ "DisableAlphaWrite" ]
    },

    "particles_alpha:particles_base": {
      "+defines": [ "ALPHA_TEST" ],
      "+states": [ "DisableAlphaWrite" ]
    },

    "particles_blend:particles_base": {
      "+states": [
        "Blending",
        "DisableDepthWrite"
      ]
    },

    "particles_effects:particles_alpha": {
      "+defines": [ "EFFECTS_OFFSET" ],
      "msaaSupport": "Both"
    },

    "particles_add:particles_blend": {
      "blendSrc": "SourceAlpha",
      "blendDst": "One"
    },

    "particles_random_test": {
      "vertexShader": "shaders/particle_random_test.vertex",
      "vrGeometryShader": "shaders/color_uv.geometry",
      "fragmentShader": "shaders/color_texture.fragment",

      "vertexFields": [
        { "field": "Position" },
        { "field": "Color" },
        { "field": "Normal" },
        { "field": "UV0" }
      ],

      "+samplerStates": [
        {
          "samplerIndex": 0,
          "textureFilter": "Point"
        }
      ],

      "+defines": [ "ALPHA_TEST" ],
      "+states": [ "DisableAlphaWrite" ],

      "msaaSupport": "Both"
    }
  }
}"#;

const CLASSIC_STEVE_TEXTURE: &[u8] = include_bytes!("s.png");
const CLASSIC_ALEX_TEXTURE: &[u8] = include_bytes!("a.png");

const JAVA_CLOUDS_TEXTURE: &[u8] = include_bytes!("Diskksks.png");

// Xelo constants end

// Xelo fn start

fn get_cross_hair_png_data(filename: &str) -> Option<Arc<Vec<u8>>> {
    if !is_custom_cross_hair_enabled() || filename != "cross_hair.png" {
        return None;
    }
    
    if std::path::Path::new(CUSTOM_CROSS_HAIR_PATH).exists() {
        if let Ok(user_data) = std::fs::read(CUSTOM_CROSS_HAIR_PATH) {
            log::info!("Loaded custom crosshair");
            return Some(Arc::new(user_data));
        }
    }
    
    log::info!("Using bundled crosshair");
    Some(Arc::new(CUSTOM_CROSS_HAIR_PNG.to_vec()))
}

fn pack_icn_file(c_path: &Path) -> bool {
    
    let path_str = c_path.to_string_lossy();
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };
    
    if filename != "pack_icon.png" {
        return false;
    }
    
    let pack_icn_patterns = [
        "resource_packs/vanilla/pack_icon.png",
        "assets/resource_packs/vanilla/pack_icon.png",
        "vanilla/pack_icon.png",
        "assets/vanilla/pack_icon.png",
    ];
    
    pack_icn_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

fn get_shadows_material_data(filename: &str) -> Option<&'static [u8]> {
    if !is_no_shadows_enabled() {
        return None;
    }
    
    match filename {
        "shadows.material" => Some(SHADOWS_MATERIAL),
        _ => None,
    }
}

fn is_no_flipbook_animations_file(c_path: &Path) -> bool {
    if !is_no_flipbook_animations_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };
    
    if filename != "Flipbook.material.bin" {
        return false;
    }
    
    let flipbook_textures_patterns = [
        "materials/Flipbook.material.bin",
        "/materials/Flipbook.material.bin",
        "resource_packs/vanilla/materials/Flipbook.material.bin",
        "assets/resource_packs/vanilla/materials/Flipbook.material.bin",
        "vanilla/materials/Flipbook.material.bin",
        "assets/materials/Flipbook.material.bin",
    ];
    
    flipbook_textures_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

fn is_particles_disabler_file(c_path: &Path) -> bool {
    if !is_particles_disabler_enabled() {
        return false;
    }

    let path_str = c_path.to_string_lossy();
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };

    let fname = filename.as_ref();
    if fname != "Particle.material.bin"
        && fname != "ParticleForwardPBR.material.bin"
        && fname != "ParticlePrepass.material.bin"
    {
        return false;
    }

    // Combined patterns for all three particle files
    let patterns = [
        // Particle.material.bin patterns
        "materials/Particle.material.bin",
        "/materials/Particle.material.bin",
        "resource_packs/vanilla/materials/Particle.material.bin",
        "assets/resource_packs/vanilla/materials/Particle.material.bin",
        "vanilla/materials/Particle.material.bin",
        "assets/materials/Particle.material.bin",

        // ParticleForwardPBR.material.bin patterns
        "materials/ParticleForwardPBR.material.bin",
        "/materials/ParticleForwardPBR.material.bin",
        "resource_packs/vanilla/materials/ParticleForwardPBR.material.bin",
        "assets/resource_packs/vanilla/materials/ParticleForwardPBR.material.bin",
        "vanilla/materials/ParticleForwardPBR.material.bin",
        "assets/materials/ParticleForwardPBR.material.bin",

        // ParticlePrepass.material.bin patterns
        "materials/ParticlePrepass.material.bin",
        "/materials/ParticlePrepass.material.bin",
        "resource_packs/vanilla/materials/ParticlePrepass.material.bin",
        "assets/resource_packs/vanilla/materials/ParticlePrepass.material.bin",
        "vanilla/materials/ParticlePrepass.material.bin",
        "assets/materials/ParticlePrepass.material.bin",
    ];

    patterns.iter().any(|pattern| path_str.contains(pattern) || path_str.ends_with(pattern))
}

fn is_no_weather_file(c_path: &Path) -> bool {
    if !is_no_weather_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };

    let fname = filename.as_ref();
    if fname != "Weather.material.bin"
    && fname != "WeatherForwardPBR.material.bin"
    {
        return false;
    }

    let patterns = [
        // Weather.material.bin patterns
        "materials/Weather.material.bin",
        "/materials/Weather.material.bin",
        "resource_packs/vanilla/materials/Weather.material.bin",
        "assets/resource_packs/vanilla/materials/Weather.material.bin",
        "vanilla/materials/Weather.material.bin",
        "assets/materials/Weather.material.bin",

        // WeatherForwardPBR.material.bin
        "materials/WeatherForwardPBR.material.bin",
        "/materials/WeatherForwardPBR.material.bin",
        "resource_packs/vanilla/materials/WeatherForwardPBR.material.bin",
        "assets/resource_packs/vanilla/materials/WeatherForwardPBR.material.bin",
        "vanilla/materials/WeatherForwardPBR.material.bin",
        "assets/materials/WeatherForwardPBR.material.bin",
    ];

    patterns.iter().any(|pattern| path_str.contains(pattern) || path_str.ends_with(pattern))
}

fn is_no_sunmoon_file(c_path: &Path) -> bool {
    if !is_no_sunmoon_enabled() {
        return false;
    }

    let path_str = c_path.to_string_lossy();
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };

    let fname = filename.as_ref();
    if fname != "SunMoon.material.bin"
    && fname != "SunMoonForwardPBR.material.bin"
    {
        return false;
    }

    let patterns = [
        "materials/SunMoon.material.bin",
        "/materials/SunMoon.material.bin",
        "resource_packs/vanilla/materials/SunMoon.material.bin",
        "assets/resource_packs/vanilla/materials/SunMoon.material.bin",
        "vanilla/materials/SunMoon.material.bin",
        "assets/materials/SunMoon.material.bin",

        "materials/SunMoonForwardPBR.material.bin",
        "/materials/SunMoonForwardPBR.material.bin",
        "resource_packs/vanilla/materials/SunMoonForwardPBR.material.bin",
        "assets/resource_packs/vanilla/materials/SunMoonForwardPBR.material.bin",
        "vanilla/materials/SunMoonForwardPBR.material.bin",
        "assets/materials/SunMoonForwardPBR.material.bin",
    ];

    patterns.iter().any(|pattern| path_str.contains(pattern) || path_str.ends_with(pattern))
}

fn is_no_stars_file(c_path: &Path) -> bool {
    if !is_no_stars_enabled() {
        return false;
    }

    let path_str = c_path.to_string_lossy();
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };

    let fname = filename.as_ref();
    if fname != "Stars.material.bin"
    && fname != "StarsForwardPBR.material.bin"
    {
        return false;
    }

    let patterns = [
        "materials/Stars.material.bin",
        "/materials/Stars.material.bin",
        "resource_packs/vanilla/materials/Stars.material.bin",
        "assets/resource_packs/vanilla/materials/Stars.material.bin",
        "vanilla/materials/Stars.material.bin",
        "assets/materials/Stars.material.bin",

        "materials/StarsForwardPBR.material.bin",
        "/materials/StarsForwardPBR.material.bin",
        "resource_packs/vanilla/materials/StarsForwardPBR.material.bin",
        "assets/resource_packs/vanilla/materials/StarsForwardPBR.material.bin",
        "vanilla/materials/StarsForwardPBR.material.bin",
        "assets/materials/StarsForwardPBR.material.bin",
    ];

    patterns.iter().any(|pattern| path_str.contains(pattern) || path_str.ends_with(pattern))
}

fn is_third_person_camera_file(c_path: &Path) -> bool {
    if !is_double_tppview_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };
    
    // Must be exactly third_person.json
    if filename != "third_person.json" {
        return false;
    }
    
    // Check if it's in cameras directory
    let third_person_patterns = [
        "cameras/third_person.json",
        "/cameras/third_person.json",
        "resource_packs/vanilla/cameras/third_person.json",
        "assets/resource_packs/vanilla/cameras/third_person.json",
        "vanilla/cameras/third_person.json",
        "assets/cameras/third_person.json",
    ];
    
    third_person_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

fn modify_third_person_radius(original_data: &[u8]) -> Option<Vec<u8>> {
    let json_str = match std::str::from_utf8(original_data) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to parse third_person.json as UTF-8: {}", e);
            return None;
        }
    };
    
    let mut json_value: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse third_person.json as JSON: {}", e);
            return None;
        }
    };
    
    // Navigate to the camera_orbit radius
    if let Some(camera_entity) = json_value
        .get_mut("minecraft:camera_entity")
        .and_then(|ce| ce.as_object_mut())
    {
        if let Some(components) = camera_entity
            .get_mut("components")
            .and_then(|comp| comp.as_object_mut())
        {
            if let Some(camera_orbit) = components
                .get_mut("minecraft:camera_orbit")
                .and_then(|orbit| orbit.as_object_mut())
            {
                // Check if radius exists and modify it
                if let Some(radius) = camera_orbit.get("radius") {
                    let current_radius = radius.as_f64().unwrap_or(4.0);
                    log::info!("Found radius: {}, changing to: {}", current_radius, current_radius * 2.0);
                    camera_orbit.insert("radius".to_string(), Value::from(current_radius * 2.0));
                } else {
                    // If radius doesn't exist, add it with doubled value
                    log::info!("No radius found, adding radius: 8.0");
                    camera_orbit.insert("radius".to_string(), Value::from(8.0));
                }
            } else {
                log::warn!("minecraft:camera_orbit not found in third_person.json");
                return None;
            }
        } else {
            log::error!("components not found in third_person.json");
            return None;
        }
    } else {
        log::error!("minecraft:camera_entity not found in third_person.json");
        return None;
    }
    
    // Convert back to JSON string with proper formatting
    match serde_json::to_string_pretty(&json_value) {
        Ok(modified_json) => {
            log::info!("Successfully modified third_person.json radius");
            Some(modified_json.into_bytes())
        },
        Err(e) => {
            log::error!("Failed to serialize modified third_person.json: {}", e);
            None
        }
    }
}

fn remove_eat_animation(original_data: &[u8]) -> Option<Vec<u8>> {
    let json_str = match std::str::from_utf8(original_data) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to parse item JSON as UTF-8: {}", e);
            return None;
        }
    };

    let mut root: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse item JSON: {}", e);
            return None;
        }
    };

    let mut changed = false;

    remove_eat_recursive(&mut root, &mut changed);

    if !changed {
        return None; 
    }

    match serde_json::to_string_pretty(&root) {
        Ok(s) => {
            log::info!("Removed minecraft:use_animation eat from item JSON");
            Some(s.into_bytes())
        }
        Err(e) => {
            log::error!("Failed to re-serialise item JSON: {}", e);
            None
        }
    }
}

fn remove_eat_recursive(value: &mut Value, changed: &mut bool) {
    match value {
        Value::Object(map) => {
            if map.get("minecraft:use_animation")
                .and_then(|v| v.as_str())
                == Some("eat")
            {
                map.remove("minecraft:use_animation");
                *changed = true;
            }
            for child in map.values_mut() {
                remove_eat_recursive(child, changed);
            }
        }
        Value::Array(arr) => {
            for child in arr.iter_mut() {
                remove_eat_recursive(child, changed);
            }
        }
        _ => {}
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

fn get_pumpkin_png_data(filename: &str) -> Option<&'static [u8]> {
    if !is_no_pumpkin_overlay_enabled() {
        return None;
    }
    
    match filename {
        "pumpkinblur.png" => Some(CLEAR_PNG),
        _ => None,
    }
}

fn get_spyglass_png_data(filename: &str) -> Option<&'static [u8]> {
    if !is_no_spyglass_overlay_enabled() {
        return None;
    }
    
    match filename {
        "spyglass_scope.png" => Some(CLEAR_PNG),
        _ => None,
    }
}

// Enhanced cape_invisible texture detection with more patterns
fn is_cape_invisible_texture_file(c_path: &Path) -> bool {
    if !is_client_capes_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    let filename = c_path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
    
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
    
    // Also check if filename itself is cape_invisible.png
    if filename == "cape_invisible.png" || filename == "cape_invisible" {
        return true;
    }
    
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

fn is_psm_particles_material_file(c_path: &Path) -> bool {
    if !is_particles_disabler_enabled() {
        return false;
    }

    let path_str = c_path.to_string_lossy();

    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };

    if filename != "particles.material" {
        return false;
    }

    let particles_material_patterns = [
        "materials/particles.material",
        "/materials/particles.material",
        "resource_packs/vanilla/materials/particles.material",
        "assets/resource_packs/vanilla/materials/particles.material",
        "vanilla/materials/particles.material",
        "assets/materials/particles.material",
    ];

    particles_material_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
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

// Enhanced cape render controllers detection
fn is_client_capes_file(c_path: &Path) -> bool {
    if !is_client_capes_enabled() {
        return false;
    }
    
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };
    
    // Check for cape render controller files
    let cape_render_files = [
        "cape.render_controllers.json"
    ];
    
    cape_render_files.contains(&filename.as_ref())
}

fn is_outline_material_file(c_path: &Path) -> bool {
    if !is_block_whiteoutline_enabled() {
        return false;
    }
    
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };
    
    let outline_material_files = [
        "ui3D.material"
    ];
    
    outline_material_files.contains(&filename.as_ref())
}

fn is_bow_render_file(c_path: &Path) -> bool {
    if !is_no_bow_animation() {
        return false;
    }
    
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };
    
    let bow_render_file = [
        "bow.render_controllers.json"
    ];
    
    bow_render_file.contains(&filename.as_ref())
}

fn is_fancy_json_file(c_path: &Path) -> bool {
    if !is_portal_optimizer() {
        return false;
    }

    let path_str = c_path.to_string_lossy();

    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };

    if filename != "fancy.json" {
        return false;
    }

    let fancy_patterns = [
        "materials/fancy.json",
        "/materials/fancy.json",
        "resource_packs/vanilla/materials/fancy.json",
        "assets/resource_packs/vanilla/materials/fancy.json",
        "vanilla/materials/fancy.json",
        "assets/materials/fancy.json",
    ];

    fancy_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

fn is_items_json_file(c_path: &Path) -> bool {
    if !is_no_eating_animation() {
        return false;
    }

    let path_str = c_path.to_string_lossy();

    let extension_ok = c_path
        .extension()
        .map(|e| e == "json")
        .unwrap_or(false);

    if !extension_ok {
        return false;
    }

    path_str.contains("/items/") || path_str.contains("\\items\\")
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

fn remove_portal_fancy_define(original_data: &[u8]) -> Option<Vec<u8>> {
    let json_str = match std::str::from_utf8(original_data) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to parse fancy.json as UTF-8: {}", e);
            return None;
        }
    };

    let mut root: Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to parse fancy.json as JSON: {}", e);
            return None;
        }
    };

    let mut changed = false;

    if let Some(arr) = root.as_array_mut() {
        for entry in arr.iter_mut() {
            if let Some(obj) = entry.as_object_mut() {
                let is_portal = obj
                    .get("path")
                    .and_then(|v| v.as_str())
                    == Some("materials/portal.material");

                if is_portal && obj.contains_key("+defines") {
                    obj.remove("+defines");
                    changed = true;
                    log::info!("Removed +defines from portal.material entry in fancy.json");
                }
            }
        }
    }

    if !changed {
        log::info!("portal.material +defines entry not found in fancy.json, passing through unchanged");
        return None;
    }

    match serde_json::to_string_pretty(&root) {
        Ok(s) => Some(s.into_bytes()),
        Err(e) => {
            log::error!("Failed to re-serialise fancy.json: {}", e);
            None
        }
    }
}


// Enhanced player.entity.json detection
fn is_player_entity_file(c_path: &Path) -> bool {
    if !is_client_capes_enabled() {
        return false;
    }
    
    let path_str = c_path.to_string_lossy();
    let filename = match c_path.file_name() {
        Some(name) => name.to_string_lossy(),
        None => return false,
    };
    
    // Must be exactly player.entity.json
    if filename != "player.entity.json" {
        return false;
    }
    
    // Check if it's in a valid entity location
    let player_entity_patterns = [
        "entity/player.entity.json",
        "/entity/player.entity.json",
        "entities/player.entity.json", 
        "/entities/player.entity.json",
        "resource_packs/vanilla/entity/player.entity.json",
        "assets/resource_packs/vanilla/entity/player.entity.json",
        "vanilla/entity/player.entity.json",
        "assets/entity/player.entity.json",
        "assets/entities/player.entity.json",
    ];
    
    player_entity_patterns.iter().any(|pattern| {
        path_str.contains(pattern) || path_str.ends_with(pattern)
    })
}

// Improved custom cape texture loading with better error handling
fn load_custom_cape_texture() -> Option<Vec<u8>> {
    match std::fs::read(CAPE_TEXTURE_PATH) {
        Ok(data) => {
            if data.is_empty() {
                log::warn!("Custom cape texture file is empty: {}", CAPE_TEXTURE_PATH);
                return None;
            }
            log::info!("Successfully loaded custom cape texture from: {} ({} bytes)", CAPE_TEXTURE_PATH, data.len());
            Some(data)
        }
        Err(e) => {
            log::warn!("Failed to load custom cape texture from {}: {}", CAPE_TEXTURE_PATH, e);
            log::info!("Make sure xelo_cape.png exists in the origin_mods folder and is a valid PNG file");
            None
        }
    }
}

// Improved player.entity.json modification with better error handling
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
            // Get the existing render_controllers array
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
                    if let Some(obj) = controller.as_object() {
                        obj.contains_key("controller.render.player.cape")
                    } else {
                        false
                    }
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
            
            // Verify textures section has cape texture (should already exist in the default file)
            if let Some(textures) = description.get("textures").and_then(|t| t.as_object()) {
                if textures.contains_key("cape") {
                    log::info!("Cape texture reference already exists in player.entity.json");
                } else {
                    log::warn!("Cape texture reference missing from player.entity.json");
                }
            } else {
                log::error!("Textures section not found in player.entity.json");
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
    
    // Convert back to JSON string with proper formatting
    match serde_json::to_string_pretty(&json_value) {
        Ok(modified_json) => Some(modified_json.into_bytes()),
        Err(e) => {
            log::error!("Failed to serialize modified player.entity.json: {}", e);
            None
        }
    }
}

// Xelo fn end

pub unsafe extern "C" fn open(
    man: *mut AAssetManager,
    fname: *const c_char,
    mode: c_int,
) -> *mut AAsset {
    // This is where UB can happen, but we are merely a hook.
    let aasset = unsafe { ndk_sys::AAssetManager_open(man, fname, mode) };
    let pointer = match std::ptr::NonNull::new(man) {
        Some(yay) => yay,
        None => {
            log::warn!("AssetManager is null?, preposterous, mc detection failed");
            return aasset;
        }
    };
    let manager = unsafe { ndk::asset::AssetManager::from_ptr(pointer) };
    let c_str = unsafe { CStr::from_ptr(fname) };
    let raw_cstr = c_str.to_bytes();
    let os_str = OsStr::from_bytes(raw_cstr);
    let c_path: &Path = Path::new(os_str);
    let Some(os_filename) = c_path.file_name() else {
        log::warn!("Path had no filename: {c_path:?}");
        return aasset;
    };
    
// Xelo Start

    // Debug logging for client capes
    if is_client_capes_enabled() {
        let path_str = c_path.to_string_lossy();
        if path_str.contains("cape") || path_str.contains("player.entity") {
            log::info!("Client capes enabled - checking file: {}", c_path.display());
        }
    }
    
    
    // Handle cape_invisible texture replacement
    if is_cape_invisible_texture_file(c_path) {
        log::info!("Intercepting cape_invisible texture with custom cape: {}", c_path.display());
        
        if let Some(custom_cape_data) = load_custom_cape_texture() {
            let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
            wanted_lock.insert(AAssetPtr(aasset), Cursor::new(custom_cape_data));
            return aasset;
        } else {
            log::warn!("Custom cape texture not found, blocking cape_invisible texture");
            // Block the original cape_invisible texture if custom one isn't available
            if !aasset.is_null() {
                ndk_sys::AAsset_close(aasset);
            }
            return std::ptr::null_mut();
        }
    }
    
    if is_psm_particles_material_file(c_path) {
        log::info!("PSM: Intercepting particles.material with single-mapping replacement: {}", c_path.display());
        let buffer = PSM_PARTICLES_MATERIAL.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }

    // Block persona files if classic skins enabled
    if is_persona_file_to_block(c_path) {
        log::info!("Blocking persona file due to classic_skins enabled: {}", c_path.display());
        if !aasset.is_null() {
            ndk_sys::AAsset_close(aasset);
        }
        return std::ptr::null_mut();
    }
    
    // Handle player.entity.json modification
    if is_player_entity_file(c_path) {
        log::info!("Intercepting player.entity.json with client capes modification: {}", c_path.display());
        
        // Read the original file first
        if aasset.is_null() {
            log::error!("Failed to open original player.entity.json");
            return aasset;
        }
        
        let length = ndk_sys::AAsset_getLength(aasset) as usize;
        if length == 0 {
            log::error!("player.entity.json has zero length");
            return aasset;
        }
        
        let mut original_data = vec![0u8; length];
        let bytes_read = ndk_sys::AAsset_read(aasset, original_data.as_mut_ptr() as *mut libc::c_void, length);
        
        if bytes_read != length as i32 {
            log::error!("Failed to read original player.entity.json completely (read {}, expected {})", bytes_read, length);
            return aasset;
        }
        
        // Reset the asset position for normal operation
        ndk_sys::AAsset_seek(aasset, 0, libc::SEEK_SET);
        
        if let Some(modified_data) = modify_player_entity_json(&original_data) {
            let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
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
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    // Custom loading messages
    if os_filename == "loading_messages.json" {
        log::info!("Intercepting loading_messages.json with custom content");
        let buffer = CUSTOM_LOADING_MESSAGES_JSON.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    // Java clouds texture replacement
    if is_clouds_texture_file(c_path) {
        log::info!("Intercepting clouds texture with Java clouds texture: {}", c_path.display());
        let buffer = JAVA_CLOUDS_TEXTURE.to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_third_person_camera_file(c_path) {
        log::info!("Intercepting third_person.json with double TPP view modification: {}", c_path.display());
        
        // Read the original file first
        if aasset.is_null() {
            log::error!("Failed to open original third_person.json");
            return aasset;
        }
        
        let length = ndk_sys::AAsset_getLength(aasset) as usize;
        if length == 0 {
            log::error!("third_person.json has zero length");
            return aasset;
        }
        
        let mut original_data = vec![0u8; length];
        let bytes_read = ndk_sys::AAsset_read(aasset, original_data.as_mut_ptr() as *mut libc::c_void, length);
        
        if bytes_read != length as i32 {
            log::error!("Failed to read original third_person.json completely (read {}, expected {})", bytes_read, length);
            return aasset;
        }
        
        // Reset the asset position for normal operation
        ndk_sys::AAsset_seek(aasset, 0, libc::SEEK_SET);
        
        if let Some(modified_data) = modify_third_person_radius(&original_data) {
            let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
            wanted_lock.insert(AAssetPtr(aasset), Cursor::new(modified_data));
            return aasset;
        } else {
            log::warn!("Failed to modify third_person.json radius, using original");
            return aasset;
        }
    }

    // Classic skins replacements
    if is_classic_skins_steve_texture_file(c_path) {
        log::info!("Intercepting steve.png with classic Steve texture: {}", c_path.display());
        let buffer = CLASSIC_STEVE_TEXTURE.to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_classic_skins_alex_texture_file(c_path) {
        log::info!("Intercepting alex.png with classic Alex texture: {}", c_path.display());
        let buffer = CLASSIC_ALEX_TEXTURE.to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_classic_skins_json_file(c_path) {
        log::info!("Intercepting skins.json with classic skins content: {}", c_path.display());
        let buffer = CUSTOM_SKINS_JSON.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    // Handle cape render controllers
    if is_client_capes_file(c_path) {
        log::info!("Intercepting cape render controller file with cape content: {}", c_path.display());
        let buffer = RENDER_JSON.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_outline_material_file(c_path) {
        log::info!("Intercepting  ui3dmaterial file with new content: {}", c_path.display());
        let buffer = CUSTOM_BLOCKOUTLINE.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_bow_render_file(c_path) {
        log::info!("Intercepting  bow render controller file with new content: {}", c_path.display());
        let buffer = CUSTOM_BOW_RENDER_CONTROLLER.as_bytes().to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
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
                let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
                wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
                return aasset;
            }
            
            if os_filename == "third_person.json" {
                log::info!("Intercepting cameras/third_person.json with custom content (nohurtcam enabled)");
                let buffer = CUSTOM_THIRD_PERSON_JSON.as_bytes().to_vec();
                let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
                wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
                return aasset;
            }
            
            if os_filename == "third_person_front.json" {
                log::info!("Intercepting cameras/third_person_front.json with custom content (nohurtcam enabled)");
                let buffer = CUSTOM_THIRD_PERSON_FRONT_JSON.as_bytes().to_vec();
                let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
                wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
                return aasset;
            }
        }
    }
        
    // Material replacements
    let filename_str = os_filename.to_string_lossy();
    if let Some(shadows_material_data) = get_shadows_material_data(&filename_str) {
        log::info!("Intercepting {} with shadow material (noshadows enabled)", filename_str);
        let buffer = shadows_material_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if is_no_flipbook_animations_file(c_path) {
    log::info!("Intercepting shield animation with side shield animation: {}", c_path.display());
    let buffer = Vec::new();
    let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
    return aasset;
}


if is_particles_disabler_file(c_path) {
    log::info!("Intercepting particle material file with combined replacements: {}", c_path.display());
    
    // Combine all particle material buffers
    let buffer = Vec::new();
    
    let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
    return aasset;
}

if is_no_weather_file(c_path) {
    log::info!("Intercepting Weather material file with combined replacement: {}", c_path.display());

    // Combine bla bla buffers
    let buffer = Vec::new();

    let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
    return aasset;
}

if is_no_sunmoon_file(c_path) {
    log::info!("gooning Sun and Moon file and bla bla bla replacement: {}", c_path.display());

    // no need comment lol 
    let buffer = Vec::new();

    let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
    return aasset;
}

if is_no_stars_file(c_path) {
    log::info!("shit here we go again with stars material: {}", c_path.display());

    // do i need to Repeat that sentence
    let buffer = Vec::new();

    let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
    return aasset;
}
    
    if let Some(title_png_data) = get_title_png_data(&filename_str) {
        log::info!("Intercepting {} with xelo title png (xelo-title enabled)", filename_str);
        let buffer = title_png_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if let Some(spyglass_png_data) = get_spyglass_png_data(&filename_str) {
        log::info!("Intercepting {} with no spyglass png (no-spyglass-overlay-enabled enabled)", filename_str);
        let buffer = spyglass_png_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    if let Some(pumpkin_png_data) = get_pumpkin_png_data(&filename_str) {
        log::info!("Intercepting {} with no pumpkin overlay png (no-pumpkin-overlay-enabled enabled)", filename_str);
        let buffer = pumpkin_png_data.to_vec();
        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
        return aasset;
    }
    
    //custom cross hair
    
    if let Some(cross_hair_png_data) = get_cross_hair_png_data(&filename_str) {
    log::info!("Intercepting {} with crosshair png (custom-cross-hair-enabled enabled)", filename_str);
    let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(cross_hair_png_data.clone().to_vec()));
    return aasset;
}

if is_items_json_file(c_path) {
        log::info!("Checking item file for eat animation: {}", c_path.display());

        if !aasset.is_null() {
            let length = ndk_sys::AAsset_getLength(aasset) as usize;
            if length > 0 {
                let mut original_data = vec![0u8; length];
                let bytes_read = ndk_sys::AAsset_read(
                    aasset,
                    original_data.as_mut_ptr() as *mut libc::c_void,
                    length,
                );

                if bytes_read == length as i32 {
                    ndk_sys::AAsset_seek(aasset, 0, libc::SEEK_SET);

                    if let Some(modified_data) = remove_eat_animation(&original_data) {
                        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
                        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(modified_data));
                    }
                }
            }
        }

        return aasset;
    }
    
    if is_fancy_json_file(c_path) {
        log::info!("Checking fancy.json for portal optimizer: {}", c_path.display());

        if !aasset.is_null() {
            let length = ndk_sys::AAsset_getLength(aasset) as usize;
            if length > 0 {
                let mut original_data = vec![0u8; length];
                let bytes_read = ndk_sys::AAsset_read(
                    aasset,
                    original_data.as_mut_ptr() as *mut libc::c_void,
                    length,
                );

                if bytes_read == length as i32 {
                    ndk_sys::AAsset_seek(aasset, 0, libc::SEEK_SET);

                    if let Some(modified_data) = remove_portal_fancy_define(&original_data) {
                        let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
                        wanted_lock.insert(AAssetPtr(aasset), Cursor::new(modified_data));
                    }
                }
            }
        }

        return aasset;
    }

if pack_icn_file(c_path) {
    log::info!("Intercepting with pack icon: {}", c_path.display());
    let buffer = PACK_ICN_PNG.to_vec();
    let mut wanted_lock = WANTED_ASSETS_MUTEX.lock().unwrap();
    wanted_lock.insert(AAssetPtr(aasset), Cursor::new(buffer));
    return aasset;
}
    
    
    let mut sus = MC_FILELOADER.lock().ignore_poison();
    if let Some(yay) = sus.get_file(c_path, manager) {
        unsafe { WANTED_ASSETS.get_mut() }.insert(AAssetPtr(aasset), yay);
    }
    aasset
}
macro_rules! handle_result {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                log::error!("{e}");
                return -1;
            }
        }
    };
}

pub unsafe extern "C" fn seek64(aasset: *mut AAsset, off: off64_t, whence: c_int) -> off64_t {
    let ptr = AAssetPtr(aasset);
    
    if let Some(file) = WANTED_ASSETS.get_mut().get_mut(&ptr) {
        handle_result!(seek_facade(off, whence, file).try_into())
    } else if let Ok(mut cursor_map) = WANTED_ASSETS_MUTEX.lock() {
        if let Some(cursor) = cursor_map.get_mut(&ptr) {
            let offset = match whence {
                libc::SEEK_SET => io::SeekFrom::Start(off.max(0) as u64),
                libc::SEEK_CUR => io::SeekFrom::Current(off),
                libc::SEEK_END => io::SeekFrom::End(off),
                _ => return ndk_sys::AAsset_seek64(aasset, off, whence),
            };
            match cursor.seek(offset) {
                Ok(pos) => pos as off64_t,
                Err(_) => ndk_sys::AAsset_seek64(aasset, off, whence),
            }
        } else {
            ndk_sys::AAsset_seek64(aasset, off, whence)
        }
    } else {
        ndk_sys::AAsset_seek64(aasset, off, whence)
    }
}

pub unsafe extern "C" fn seek(aasset: *mut AAsset, off: off_t, whence: c_int) -> off_t {
    let ptr = AAssetPtr(aasset);
    
    if let Some(file) = WANTED_ASSETS.get_mut().get_mut(&ptr) {
        handle_result!(seek_facade(off.into(), whence, file).try_into())
    } else if let Ok(mut cursor_map) = WANTED_ASSETS_MUTEX.lock() {
        if let Some(cursor) = cursor_map.get_mut(&ptr) {
            let offset = match whence {
                libc::SEEK_SET => io::SeekFrom::Start(off.max(0) as u64),
                libc::SEEK_CUR => io::SeekFrom::Current(off.into()),
                libc::SEEK_END => io::SeekFrom::End(off.into()),
                _ => return ndk_sys::AAsset_seek(aasset, off, whence),
            };
            match cursor.seek(offset) {
                Ok(pos) => pos as off_t,
                Err(_) => ndk_sys::AAsset_seek(aasset, off, whence),
            }
        } else {
            ndk_sys::AAsset_seek(aasset, off, whence)
        }
    } else {
        ndk_sys::AAsset_seek(aasset, off, whence)
    }
}

pub unsafe extern "C" fn read(aasset: *mut AAsset, buf: *mut c_void, count: size_t) -> c_int {
    let ptr = AAssetPtr(aasset);
    let rs_buffer = core::slice::from_raw_parts_mut(buf as *mut u8, count);
    
    if let Some(file) = WANTED_ASSETS.get_mut().get_mut(&ptr) {
        let read_total = handle_result!((*file).read(rs_buffer));
        handle_result!(read_total.try_into())
    } else if let Ok(mut cursor_map) = WANTED_ASSETS_MUTEX.lock() {
        if let Some(cursor) = cursor_map.get_mut(&ptr) {
            let read_total = handle_result!(cursor.read(rs_buffer));
            handle_result!(read_total.try_into())
        } else {
            ndk_sys::AAsset_read(aasset, buf, count)
        }
    } else {
        ndk_sys::AAsset_read(aasset, buf, count)
    }
}

pub unsafe extern "C" fn len(aasset: *mut AAsset) -> off_t {
    let ptr = AAssetPtr(aasset);
    
    if let Some(file) = unsafe { WANTED_ASSETS.get_mut() }.get(&ptr) {
        handle_result!(file.get_ref().len().try_into())
    } else if let Ok(cursor_map) = WANTED_ASSETS_MUTEX.lock() {
        if let Some(cursor) = cursor_map.get(&ptr) {
            handle_result!(cursor.get_ref().len().try_into())
        } else {
            ndk_sys::AAsset_getLength(aasset)
        }
    } else {
        ndk_sys::AAsset_getLength(aasset)
    }
}

pub unsafe extern "C" fn len64(aasset: *mut AAsset) -> off64_t {
    let ptr = AAssetPtr(aasset);
    
    if let Some(file) = unsafe { WANTED_ASSETS.get_mut() }.get(&ptr) {
        handle_result!(file.get_ref().len().try_into())
    } else if let Ok(cursor_map) = WANTED_ASSETS_MUTEX.lock() {
        if let Some(cursor) = cursor_map.get(&ptr) {
            handle_result!(cursor.get_ref().len().try_into())
        } else {
            ndk_sys::AAsset_getLength64(aasset)
        }
    } else {
        ndk_sys::AAsset_getLength64(aasset)
    }
}

pub unsafe extern "C" fn rem(aasset: *mut AAsset) -> off_t {
    let ptr = AAssetPtr(aasset);
    
    if let Some(file) = unsafe { WANTED_ASSETS.get_mut() }.get(&ptr) {
        handle_result!((file.get_ref().len() - file.position() as usize).try_into())
    } else if let Ok(cursor_map) = WANTED_ASSETS_MUTEX.lock() {
        if let Some(cursor) = cursor_map.get(&ptr) {
            handle_result!((cursor.get_ref().len() - cursor.position() as usize).try_into())
        } else {
            ndk_sys::AAsset_getRemainingLength(aasset)
        }
    } else {
        ndk_sys::AAsset_getRemainingLength(aasset)
    }
}

pub unsafe extern "C" fn rem64(aasset: *mut AAsset) -> off64_t {
    let ptr = AAssetPtr(aasset);
    
    if let Some(file) = unsafe { WANTED_ASSETS.get_mut() }.get(&ptr) {
        handle_result!((file.get_ref().len() - file.position() as usize).try_into())
    } else if let Ok(cursor_map) = WANTED_ASSETS_MUTEX.lock() {
        if let Some(cursor) = cursor_map.get(&ptr) {
            handle_result!((cursor.get_ref().len() - cursor.position() as usize).try_into())
        } else {
            ndk_sys::AAsset_getRemainingLength64(aasset)
        }
    } else {
        ndk_sys::AAsset_getRemainingLength64(aasset)
    }
}

pub unsafe extern "C" fn close(aasset: *mut AAsset) {
    let ptr = AAssetPtr(aasset);
    
    if let Some(buffer) = unsafe { WANTED_ASSETS.get_mut() }.remove(&ptr) {
        MC_FILELOADER.lock().ignore_poison().last_buffer = Some(buffer);
    }
    WANTED_ASSETS_MUTEX.lock().unwrap().remove(&ptr);
    
    ndk_sys::AAsset_close(aasset);
}

pub unsafe extern "C" fn get_buffer(aasset: *mut AAsset) -> *const c_void {
    let ptr = AAssetPtr(aasset);
    
    if let Some(file) = unsafe { WANTED_ASSETS.get_mut() }.get(&ptr) {
        file.get_ref().as_ptr().cast()
    } else if let Ok(cursor_map) = WANTED_ASSETS_MUTEX.lock() {
        if let Some(cursor) = cursor_map.get(&ptr) {
            cursor.get_ref().as_ptr().cast()
        } else {
            ndk_sys::AAsset_getBuffer(aasset)
        }
    } else {
        ndk_sys::AAsset_getBuffer(aasset)
    }
}

pub unsafe extern "C" fn fd_dummy(
    aasset: *mut AAsset,
    out_start: *mut off_t,
    out_len: *mut off_t,
) -> c_int {
    let ptr = AAssetPtr(aasset);
    
    if unsafe { WANTED_ASSETS.get_mut() }.contains_key(&ptr) {
        log::error!("WE GOT BUSTED NOOO");
        -1
    } else if WANTED_ASSETS_MUTEX.lock().is_ok_and(|map| map.contains_key(&ptr)) {
        log::error!("WE GOT BUSTED NOOO");
        -1
    } else {
        ndk_sys::AAsset_openFileDescriptor(aasset, out_start, out_len)
    }
}

pub unsafe extern "C" fn fd_dummy64(
    aasset: *mut AAsset,
    out_start: *mut off64_t,
    out_len: *mut off64_t,
) -> c_int {
    let ptr = AAssetPtr(aasset);
    
    if unsafe { WANTED_ASSETS.get_mut() }.contains_key(&ptr) {
        log::error!("WE GOT BUSTED NOOO");
        -1
    } else if WANTED_ASSETS_MUTEX.lock().is_ok_and(|map| map.contains_key(&ptr)) {
        log::error!("WE GOT BUSTED NOOO");
        -1
    } else {
        ndk_sys::AAsset_openFileDescriptor64(aasset, out_start, out_len)
    }
}

pub unsafe extern "C" fn is_alloc(aasset: *mut AAsset) -> c_int {
    let ptr = AAssetPtr(aasset);
    
    if unsafe { WANTED_ASSETS.get_mut() }.contains_key(&ptr) {
        false as c_int
    } else if WANTED_ASSETS_MUTEX.lock().is_ok_and(|map| map.contains_key(&ptr)) {
        false as c_int
    } else {
        ndk_sys::AAsset_isAllocated(aasset)
    }
}

fn seek_facade(offset: i64, whence: c_int, file: &mut Buffer) -> i64 {
    let offset = match whence {
        libc::SEEK_SET => {
            //Let's check this so we don't mess up
            let u64_off = handle_result!(u64::try_from(offset));
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
        Ok(new_offset) => handle_result!(new_offset.try_into()),
        Err(err) => {
            log::error!("seek Error: {err}");
            return -1;
        }
    }
}
