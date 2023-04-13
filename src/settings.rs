use std::fs;

use once_cell::sync::Lazy;
use sequoia_net::wkd::Variant;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    #[serde(with = "VariantDef")]
    pub variant: Variant,
    pub max_age: i64,
    pub port: u16,
    pub folder_structure: FolderStructure,
    pub smtp_settings: MailSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FolderStructure {
    pub root_folder: String,
    pub pending_folder: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MailSettings {
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_port: u16,
    pub mail_from: String,
    pub mail_subject: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "Variant")]
pub enum VariantDef {
    Advanced,
    Direct,
}

fn get_settings() -> Settings {
    let content = fs::read_to_string("wkd.toml").unwrap();
    toml::from_str(&content).unwrap()
}

pub static SETTINGS: Lazy<Settings> = Lazy::new(get_settings);
