use crate::settings::SETTINGS;
use crate::{errors::Error, settings::ROOT_FOLDER};

use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use flexi_logger::{style, DeferredNow, FileSpec, FlexiLoggerError, Logger, LoggerHandle, Record};
use log::debug;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sequoia_net::wkd::Url;
use sequoia_openpgp::{parse::Parse, policy::StandardPolicy, Cert};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[macro_export]
macro_rules! pending_path {
    () => {
        Path::new(&ROOT_FOLDER).join("pending")
    };
}

#[macro_export]
macro_rules! webpage_path {
    () => {
        Path::new("assets").join("webpage")
    };
}

pub fn is_email_allowed(email: &str) -> Result<(), Error> {
    let allowed = match email.split('@').last() {
        Some(domain) => SETTINGS.allowed_domains.contains(&domain.to_string()),
        None => return Err(Error::ParseEmail),
    };
    if !allowed {
        return Err(Error::WrongDomain);
    }
    Ok(())
}

pub fn parse_pem(pemfile: &str) -> Result<Cert, Error> {
    let cert = match sequoia_openpgp::Cert::from_bytes(pemfile.as_bytes()) {
        Ok(cert) => cert,
        Err(_) => return Err(Error::ParseCert),
    };
    let policy = StandardPolicy::new();
    if cert.with_policy(&policy, None).is_err() {
        return Err(Error::InvalidCert);
    };
    Ok(cert)
}

pub fn gen_random_token() -> String {
    let mut rng = thread_rng();
    (0..10).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn get_email_from_cert(cert: &Cert) -> Result<String, Error> {
    let policy = StandardPolicy::new();
    let validcert = match cert.with_policy(&policy, None) {
        Ok(validcert) => validcert,
        Err(_) => return Err(Error::InvalidCert),
    };
    let userid_opt = match validcert.primary_userid() {
        Ok(userid_opt) => userid_opt,
        Err(_) => return Err(Error::ParseCert),
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
    if !Path::new(&ROOT_FOLDER).join(path).is_file() {
        return Err(Error::MissingKey);
    }
    Ok(true)
}

pub fn get_filename(path: &Path) -> Option<&str> {
    path.file_name()?.to_str()
}

pub fn custom_color_format(
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

pub fn custom_monochrome_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] [{}] {}: {}",
        now.format("%Y-%m-%d %H:%M:%S"),
        record.module_path().unwrap_or("<unnamed>"),
        record.level(),
        record.args()
    )
}

pub fn init_logger() -> Result<LoggerHandle, FlexiLoggerError> {
    Logger::try_with_env_or_str("simple_wkd=debug")?
        .log_to_file(FileSpec::default().directory("logs"))
        .duplicate_to_stdout(flexi_logger::Duplicate::All)
        .format_for_files(custom_monochrome_format)
        .adaptive_format_for_stdout(flexi_logger::AdaptiveFormat::Custom(
            custom_monochrome_format,
            custom_color_format,
        ))
        .set_palette("b1;3;2;4;6".to_string())
        .start()
}

pub fn return_outcome(data: Result<&str, &str>) -> Result<HttpResponse, Error> {
    let path = webpage_path!().join("status").join("index.html");
    let template = match fs::read_to_string(&path) {
        Ok(template) => template,
        Err(_) => {
            debug!("file {} is inaccessible", path.display());
            return Err(Error::Inaccessible);
        }
    };
    let (page, message) = match data {
        Ok(message) => (template.replace("((%s))", "Success!"), message),
        Err(message) => (template.replace("((%s))", "Failure!"), message),
    };
    let page = page.replace("((%m))", message);
    return Ok(HttpResponseBuilder::new(StatusCode::OK)
        .insert_header(ContentType::html())
        .body(page));
}
