use crate::errors::CompatErr;
use crate::errors::SpecialErrors;
use crate::settings::ROOT_FOLDER;
use crate::settings::SETTINGS;

use actix_web::ResponseError;
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use anyhow::Result;
use flexi_logger::{
    detailed_format, style, DeferredNow, FileSpec, FlexiLoggerError, Logger, LoggerHandle, Record,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sequoia_net::wkd::Url;
use sequoia_openpgp::{parse::Parse, Cert};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[macro_export]
macro_rules! validate_cert {
    ( $x:expr ) => {
        match $x.with_policy($crate::settings::POLICY, None) {
            Ok(validcert) => Ok(validcert),
            Err(_) => Err($crate::errors::SpecialErrors::InvalidCert),
        }
    };
}


pub fn pending_path() -> PathBuf {
    Path::new(&ROOT_FOLDER).join("pending")
}

pub fn webpage_path() -> PathBuf {
    Path::new("assets").join("webpage")
}

pub fn read_file(path: &PathBuf) -> Result<String> {
    if path.is_file() {
        Ok(fs::read_to_string(path)?)
    } else {
        Err(SpecialErrors::MissingFile)?
    }
}

pub fn is_email_allowed(email: &str) -> Result<()> {
    let allowed = match email.split('@').last() {
        Some(domain) => SETTINGS.allowed_domains.contains(&domain.to_string()),
        None => Err(SpecialErrors::MalformedEmail)?,
    };
    if !allowed {
        Err(SpecialErrors::UnallowedDomain)?;
    }
    Ok(())
}

pub fn parse_pem(pemfile: &str) -> Result<Cert> {
    let cert = match sequoia_openpgp::Cert::from_bytes(pemfile.as_bytes()) {
        Ok(cert) => cert,
        Err(_) => Err(SpecialErrors::MalformedCert)?,
    };
    validate_cert!(cert)?;
    Ok(cert)
}

pub fn gen_random_token() -> String {
    let mut rng = thread_rng();
    (0..10).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn get_email_from_cert(cert: &Cert) -> Result<String> {
    let validcert = validate_cert!(cert)?;
    let userid_opt = validcert.primary_userid()?;
    let email_opt = userid_opt.email()?;
    match email_opt {
        Some(email) => Ok(email),
        None => Err(SpecialErrors::EmailMissing)?,
    }
}

pub fn get_user_file_path(email: &str) -> Result<PathBuf> {
    let wkd_url = Url::from(email)?;
    wkd_url.to_file_path(SETTINGS.variant)
}

pub fn key_exists(email: &str) -> Result<bool> {
    let path = get_user_file_path(email)?;
    if !Path::new(&ROOT_FOLDER).join(path).is_file() {
        Err(SpecialErrors::InexistingUser)?
    }
    Ok(true)
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
        .format_for_files(detailed_format)
        .adaptive_format_for_stdout(flexi_logger::AdaptiveFormat::Custom(
            custom_monochrome_format,
            custom_color_format,
        ))
        .set_palette("b1;3;2;4;6".to_string())
        .start()
}

pub fn return_outcome(data: Result<&str, &CompatErr>) -> Result<HttpResponse> {
    let path = webpage_path().join("status").join("index.html");
    let template = read_file(&path)?;
    let (page, message) = match data {
        Ok(message) => (template.replace("((%s))", "Success!"), message.to_string()),
        Err(error) => (template.replace("((%s))", "Failure!"), error.to_string()),
    };
    let status_code = match data {
        Ok(_) => StatusCode::OK,
        Err(error) => error.status_code(),
    };
    let page = page.replace("((%m))", &message);
    return Ok(HttpResponseBuilder::new(status_code)
        .insert_header(ContentType::html())
        .body(page));
}
