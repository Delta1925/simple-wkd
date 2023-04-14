use once_cell::sync::Lazy;
use sequoia_net::wkd::Variant;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    #[serde(with = "VariantDef")]
    pub variant: Variant,
    pub root_folder: String,
    pub max_age: i64,
    pub cleanup_interval: u64,
    pub port: u16,
    pub external_url: String,
    pub mail_settings: MailSettings,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MailSettings {
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_port: u16,
    pub smtp_tls: SMTPEncryption,
    pub mail_from: String,
    pub mail_subject: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "Variant")]
pub enum VariantDef {
    Advanced,
    Direct,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SMTPEncryption {
    Tls,
    Starttls,
}

fn get_settings() -> Settings {
    println!("Reaing settings...");
    let content = match fs::read_to_string("wkd.toml") {
        Ok(content) => content,
        Err(_) => panic!("Unable to access settings file!"),
    };
    match toml::from_str(&content) {
        Ok(settings) => settings,
        Err(_) => panic!("Unable to parse settings from file!"),
    }
}

pub static SETTINGS: Lazy<Settings> = Lazy::new(get_settings);
