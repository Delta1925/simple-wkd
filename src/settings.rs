use lettre::{transport::smtp::authentication::Credentials, SmtpTransport};
use once_cell::sync::Lazy;
use sequoia_net::wkd::Variant;
use serde::{Deserialize, Serialize};
use std::fs;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    #[serde(with = "VariantDef")]
    pub variant: Variant,
    pub root_folder: String,
    pub max_age: i64,
    pub cleanup_interval: u64,
    pub port: u16,
    pub external_url: Url,
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
    let content = match fs::read_to_string("wkd.toml") {
        Ok(content) => content,
        Err(_) => panic!("Unable to access settings file!"),
    };
    let settings = match toml::from_str(&content) {
        Ok(settings) => settings,
        Err(_) => panic!("Unable to parse settings from file!"),
    };
    settings
}

fn get_mailer() -> SmtpTransport {
    let creds = Credentials::new(
        SETTINGS.mail_settings.smtp_username.to_owned(),
        SETTINGS.mail_settings.smtp_password.to_owned(),
    );
    let builder = match &SETTINGS.mail_settings.smtp_tls {
        SMTPEncryption::Tls => SmtpTransport::relay(&SETTINGS.mail_settings.smtp_host),
        SMTPEncryption::Starttls => {
            SmtpTransport::starttls_relay(&SETTINGS.mail_settings.smtp_host)
        }
    };
    let mailer = match builder {
        Ok(builder) => builder,
        Err(_) => panic!("Unable to set up smtp"),
    }
    .credentials(creds)
    .port(SETTINGS.mail_settings.smtp_port)
    .build();
    mailer
}

pub static SETTINGS: Lazy<Settings> = Lazy::new(get_settings);
pub static MAILER: Lazy<SmtpTransport> = Lazy::new(get_mailer);
