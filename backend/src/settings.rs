use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use log::{debug, error};
use once_cell::sync::Lazy;
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_policy_config::ConfiguredStandardPolicy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

use crate::{log_err, utils::read_file};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub variant: Variant,
    pub max_age: i64,
    pub cleanup_interval: u64,
    pub allowed_domains: Vec<String>,
    pub port: u16,
    pub bind_host: String,
    pub external_url: Url,
    pub mail_settings: MailSettings,
    pub policy: Option<Policy>,
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
pub struct Policy {
    pub key_max_validity: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Variant {
    Advanced,
    Direct,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SMTPEncryption {
    Tls,
    Starttls,
}

fn get_settings() -> Settings {
    debug!("Parsing settings...");
    let content = match read_file(&PathBuf::from("config.toml")) {
        Ok(content) => content,
        Err(_) => {
            error!("Unable to access settings file!");
            panic!("Unable to access settings file!")
        }
    };
    match log_err!(toml::from_str(&content), error) {
        Ok(settings) => settings,
        Err(_) => {
            error!("Unable to parse settings from file!");
            panic!("Unable to parse settings from file!")
        }
    }
}

fn get_mailer() -> AsyncSmtpTransport<Tokio1Executor> {
    debug!("Setting up smtp...");
    let creds = Credentials::new(
        SETTINGS.mail_settings.smtp_username.to_owned(),
        SETTINGS.mail_settings.smtp_password.to_owned(),
    );
    let builder = match &SETTINGS.mail_settings.smtp_tls {
        SMTPEncryption::Tls => {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&SETTINGS.mail_settings.smtp_host)
        }
        SMTPEncryption::Starttls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&SETTINGS.mail_settings.smtp_host)
        }
    };
    match builder {
        Ok(builder) => builder,
        Err(_) => {
            error!("Unable to set up smtp");
            panic!("Unable to set up smtp")
        }
    }
    .credentials(creds)
    .port(SETTINGS.mail_settings.smtp_port)
    .build()
}

fn get_policy<'a>() -> StandardPolicy<'a>  {
        let mut p = ConfiguredStandardPolicy::new();

        match p.parse_default_config() {
                Ok(_) => {},
                Err(e) => error!("{e}"),
        }

        p.build()
}

pub const ERROR_TEXT: &str = "An error occoured:";
pub static POLICY: Lazy<StandardPolicy> = Lazy::new(get_policy);
pub const ROOT_FOLDER: &str = "data";
pub static SETTINGS: Lazy<Settings> = Lazy::new(get_settings);
pub static MAILER: Lazy<AsyncSmtpTransport<Tokio1Executor>> = Lazy::new(get_mailer);
