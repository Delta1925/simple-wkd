use crate::errors::Error;
use crate::settings::SETTINGS;

use flexi_logger::{style, DeferredNow, FlexiLoggerError, Logger, LoggerHandle, Record};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sequoia_net::wkd::Url;
use sequoia_openpgp::{parse::Parse, Cert};
use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! pending_path {
    () => {
        Path::new(&SETTINGS.root_folder).join(PENDING_FOLDER)
    };
}

pub fn parse_pem(pemfile: &str) -> Result<Cert, Error> {
    match sequoia_openpgp::Cert::from_bytes(pemfile.as_bytes()) {
        Ok(cert) => Ok(cert),
        Err(_) => Err(Error::ParseCert),
    }
}

pub fn gen_random_token() -> String {
    let mut rng = thread_rng();
    (0..10).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn get_email_from_cert(cert: &Cert) -> Result<String, Error> {
    let userid_opt = match cert.userids().next() {
        Some(userid_opt) => userid_opt,
        None => return Err(Error::ParseCert),
    };
    let email_opt = match userid_opt.email() {
        Ok(email_opt) => email_opt,
        Err(_) => return Err(Error::ParseCert),
    };
    match email_opt {
        Some(email) => Ok(email),
        None => Err(Error::MissingMail),
    }
}

pub fn get_user_file_path(email: &str) -> Result<PathBuf, Error> {
    let wkd_url = match Url::from(email) {
        Ok(wkd_url) => wkd_url,
        Err(_) => return Err(Error::PathGeneration),
    };
    match wkd_url.to_file_path(SETTINGS.variant) {
        Ok(path) => Ok(path),
        Err(_) => Err(Error::PathGeneration),
    }
}

pub fn key_exists(email: &str) -> Result<bool, Error> {
    let path = get_user_file_path(email)?;
    if !Path::new(&SETTINGS.root_folder).join(path).is_file() {
        return Err(Error::MissingKey);
    }
    Ok(true)
}

pub fn get_filename(path: &Path) -> Option<&str> {
    path.file_name()?.to_str()
}

pub fn custom_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{}] [{}] {}: {}",
        style(level).paint(now.format("%Y-%m-%d %H:%M:%S").to_string()),
        style(level).paint(record.module_path().unwrap_or("<unnamed>")),
        style(level).paint(record.level().to_string()),
        style(level).paint(&record.args().to_string())
    )
}

pub fn init_logger() -> Result<LoggerHandle, FlexiLoggerError> {
    Logger::try_with_env_or_str("simple_wkd=trace")?
        .format(custom_format)
        .set_palette("b1;3;2;4;6".to_string())
        .start()
}
