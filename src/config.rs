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
    
    #[serde(rename = "cape_physics")]
    pub cape_physics: bool,
    
    #[serde(rename = "night_vision")]
    pub night_vision: bool,
    
    #[serde(rename = "xelo_title")]
    pub xelo_title: bool,
    
    #[serde(rename = "client_capes")]
    pub client_capes: bool,
    // You can add more fields as needed
    // #[serde(rename = "CustomField")]
    // pub custom_field: bool,
}

impl Default for ModConfig {
    fn default() -> Self {
        Self {
            no_hurt_cam: true,
            no_fog: false,
            particles_disabler: false,
            java_clouds: false,
            java_cubemap: false,
            classic_skins: false,
            cape_physics: false,
            night_vision: false,
            xelo_title: true,
            client_capes: false, 
        }
    }
}

// Global config instance
static CONFIG: OnceLock<ModConfig> = OnceLock::new();

// Config file paths - try multiple locations
const CONFIG_DIRS: &[&str] = &[
    "/storage/emulated/0/Android/data/com.origin.launcher/files/origin_mods",
    "/storage/emulated/0/Android/data/com.mojang.minecraftpe/files/origin_mods",
    "/sdcard/Android/data/com.origin.launcher/files/origin_mods",
    "/sdcard/origin_mods",
];

const CONFIG_FILES: &[&str] = &[
    "/storage/emulated/0/Android/data/com.origin.launcher/files/origin_mods/config.json",
    "/storage/emulated/0/Android/data/com.mojang.minecraftpe/files/origin_mods/config.json",
    "/sdcard/Android/data/com.origin.launcher/files/origin_mods/config.json",
    "/sdcard/origin_mods/config.json",
];

pub fn init_config() {
    let config = load_or_create_config();
    log::info!("Config initialized - client_capes: {}", config.client_capes);
    CONFIG.set(config).expect("Failed to set config");
}

pub fn get_config() -> &'static ModConfig {
    CONFIG.get().expect("Config not initialized")
}

fn load_or_create_config() -> ModConfig {
    // Try to create directories
    for dir in CONFIG_DIRS {
        if let Err(e) = fs::create_dir_all(dir) {
            log::debug!("Failed to create config directory {}: {}", dir, e);
        } else {
            log::debug!("Created/verified config directory: {}", dir);
        }
    }

    // Try to load existing config from any location
    for config_file in CONFIG_FILES {
        if Path::new(config_file).exists() {
            match load_config(config_file) {
                Ok(config) => {
                    log::info!("Loaded config from {}", config_file);
                    log::info!("Client capes enabled: {}", config.client_capes);
                    return config;
                }
                Err(e) => {
                    log::warn!("Failed to load config from {}: {}", config_file, e);
                }
            }
        }
    }

    // Create default config file in the first available directory
    let default_config = ModConfig::default();
    for config_file in CONFIG_FILES {
        if let Err(e) = save_config(&default_config, config_file) {
            log::debug!("Failed to save default config to {}: {}", config_file, e);
        } else {
            log::info!("Created default config at {}", config_file);
            break;
        }
    }

    log::info!("Using default config - client_capes: {}", default_config.client_capes);
    default_config
}

fn load_config(path: &str) -> Result<ModConfig, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let config: ModConfig = serde_json::from_str(&contents)?;
    Ok(config)
}

fn save_config(config: &ModConfig, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    let json = serde_json::to_string_pretty(config)?;
    let mut file = File::create(path)?;
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

pub fn is_cape_physics_enabled() -> bool {
    get_config().cape_physics
}

pub fn is_night_vision_enabled() -> bool {
    get_config().night_vision
}

pub fn is_xelo_title_enabled() -> bool {
    get_config().xelo_title
}

pub fn is_client_capes_enabled() -> bool {
    let enabled = get_config().client_capes;
    log::debug!("is_client_capes_enabled() called: {}", enabled);
    enabled
}

// You can add more helper functions for other config values
// pub fn is_custom_field_enabled() -> bool {
//     get_config().custom_field
// }