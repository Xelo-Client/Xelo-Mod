use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::OnceLock,
};
use serde::{Deserialize, Serialize};

// Config structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModConfig {
    #[serde(rename = "Nohurtcam")]
    pub no_hurt_cam: bool,
    
    #[serde(rename = "Nofog")]
    pub no_fog: bool,
    
    #[serde(rename = "particles_disabler")]
    pub particles_disabler: bool,
    
    #[serde(rename = "java_clouds")]
    pub java_clouds: bool,
    
    #[serde(rename = "java_cubemap")]
    pub java_cubemap: bool,
    
    #[serde(rename = "classic_skins")]
    pub classic_skins: bool,
    
    #[serde(rename = "night_vision")]
    pub night_vision: bool,
    
    #[serde(rename = "xelo_title")]
    pub xelo_title: bool,
    
    #[serde(rename = "no_shadows")]
    pub no_shadows: bool,
    
    #[serde(rename = "client_capes")]
    pub client_capes: bool,
    
    #[serde(rename = "white_block_outline")]
    pub white_block_outline: bool,
    
    #[serde(rename = "no_flipbook_animations")]
    pub no_flipbook_animations: bool,
    
    #[serde(rename = "no_pumpkin_overlay")]
    pub no_pumpkin_overlay: bool,
    
    #[serde(rename = "no_spyglass_overlay")]
    pub no_spyglass_overlay: bool,
    
    #[serde(rename = "double_tppview")]
    pub double_tppview: bool,
    
    // You can add more fields as needed
    // #[serde(rename = "CustomField")]
    // pub custom_field: bool,
}

impl Default for ModConfig {
    fn default() -> Self {
        Self {
            no_hurt_cam: false,
            no_fog: false,
            particles_disabler: false,
            java_clouds: false,
            java_cubemap: false,
            classic_skins: false,
            night_vision: false,
            xelo_title: true,
            client_capes: false,
            no_shadows: false,
            no_flipbook_animations: false,
            no_spyglass_overlay: false,
            no_pumpkin_overlay: false,
            white_block_outline: false,
            double_tppview: false,
        }
    }
}

// Global config instance
static CONFIG: OnceLock<ModConfig> = OnceLock::new();

// Config file path
const CONFIG_DIR: &str = "/storage/emulated/0/Android/data/com.origin.launcher/files/origin_mods";
const CONFIG_FILE: &str = "/storage/emulated/0/Android/data/com.origin.launcher/files/origin_mods/config.json";

pub fn init_config() {
    let config = load_or_create_config();
    CONFIG.set(config).expect("Failed to set config");
}

pub fn get_config() -> &'static ModConfig {
    CONFIG.get().expect("Config not initialized")
}

fn load_or_create_config() -> ModConfig {
    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(CONFIG_DIR) {
        log::warn!("Failed to create config directory: {}", e);
        return ModConfig::default();
    }

    // Try to load existing config
    if Path::new(CONFIG_FILE).exists() {
        match load_config() {
            Ok(config) => {
                log::info!("Loaded config from {}", CONFIG_FILE);
                return config;
            }
            Err(e) => {
                log::warn!("Failed to load config, using default: {}", e);
            }
        }
    }

    // Create default config file
    let default_config = ModConfig::default();
    if let Err(e) = save_config(&default_config) {
        log::warn!("Failed to save default config: {}", e);
    } else {
        log::info!("Created default config at {}", CONFIG_FILE);
    }

    default_config
}

fn load_config() -> Result<ModConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(CONFIG_FILE)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let config: ModConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

fn save_config(config: &ModConfig) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    let mut file = File::create(CONFIG_FILE)?;
    file.write_all(json.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

// Helper functions to check individual settings
pub fn is_no_hurt_cam_enabled() -> bool {
    get_config().no_hurt_cam
}

pub fn is_no_fog_enabled() -> bool {
    get_config().no_fog
}

pub fn is_particles_disabler_enabled() -> bool {
    get_config().particles_disabler
}

pub fn is_java_clouds_enabled() -> bool {
    get_config().java_clouds
}

pub fn is_java_cubemap_enabled() -> bool {
    get_config().java_cubemap
}

pub fn is_classic_skins_enabled() -> bool {
    get_config().classic_skins
}

pub fn is_night_vision_enabled() -> bool {
    get_config().night_vision
}

pub fn is_xelo_title_enabled() -> bool {
    get_config().xelo_title
}

pub fn is_client_capes_enabled() -> bool {
    get_config().client_capes
}

pub fn is_no_shadows_enabled() -> bool {
    get_config().no_shadows
}

pub fn is_block_whiteoutline_enabled() -> bool {
    get_config().white_block_outline
}

pub fn is_no_flipbook_animations_enabled() -> bool {
    get_config().no_flipbook_animations
}

pub fn is_no_pumpkin_overlay_enabled() -> bool {
    get_config().no_pumpkin_overlay
}

pub fn is_no_spyglass_overlay_enabled() -> bool {
    get_config().no_spyglass_overlay
}

pub fn is_double_tppview_enabled() -> bool {
    get_config().double_tppview
}

// You can add more helper functions for other config values
// pub fn is_custom_field_enabled() -> bool {
//     get_config().custom_field
// }