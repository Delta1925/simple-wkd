use crate::errors::CompatErr;
use crate::errors::SpecialErrors;
use crate::log_err;
use crate::settings::Variant;
use crate::settings::ROOT_FOLDER;
use crate::settings::SETTINGS;
use crate::settings::POLICY;

use actix_web::ResponseError;
use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use anyhow::Result;
use flexi_logger::{style, DeferredNow, FileSpec, FlexiLoggerError, Logger, LoggerHandle, Record};
use log::debug;
use log::error;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sequoia_openpgp::cert::ValidCert;
use sequoia_openpgp::cert::amalgamation::ValidateAmalgamation;
use sequoia_openpgp::serialize::Marshal;
use sequoia_openpgp::types::HashAlgorithm;
use sequoia_openpgp::{parse::Parse, Cert};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

pub fn validate_cert(cert: &Cert) -> Result<ValidCert> {
    let validcert = match log_err!(cert.with_policy(&*POLICY, None), debug) {
        Ok(validcert) => validcert,
        Err(e) => {
            debug!("Certificate was rejected: The primary key violates the policy: {}", e.source().unwrap());
            Err(SpecialErrors::InvalidCert)?
       }
    };

    for key in cert.keys().subkeys() {
        match log_err!(key.with_policy(&*POLICY, None), debug) {
            Ok(_) => continue,
            Err(e) => {
                debug!("Certificate was rejected: A sub key violates the policy: {}", e.source().unwrap());
                Err(SpecialErrors::KeyPolicyViolation)?
            }
        }
    }

    if let Some(policy_settings) = &SETTINGS.policy {
        if let Some(max_validity_setting) = policy_settings.key_max_validity {
            let max_validity = Duration::from_secs(max_validity_setting);

            if !max_validity.is_zero() {
                for key in validcert.keys() {
                    let validity = key.key_validity_period();

                    if validity.is_none() {
                        debug!("Certificate was rejected: The primary key or a subkey has validity period of zero");
                        return Err(SpecialErrors::KeyNonExpiring)?
                    } else if validity > Some(max_validity) {
                        debug!("Certificate was rejected: The primary key or a subkey has a validity period greater than {max_validity_setting} seconds");
                        return Err(SpecialErrors::KeyValidityTooLong)?
                    }
                }
            }
        }
    }

    Ok(validcert)
}

pub fn encode_local(local: &str) -> String {
    let mut digest = vec![0; 20];
    let mut algo = HashAlgorithm::SHA1.context().unwrap();
    algo.update(local.as_bytes());
    let _ = algo.digest(&mut digest);

    zbase32::encode_full_bytes(&digest[..])
}

pub fn email_to_file_path(email: &str) -> Result<PathBuf> {
    let address_data: Vec<&str> = email.split('@').collect();
    if address_data.len() != 2 {
        Err(SpecialErrors::MalformedEmail)?;
    }

    let domain = address_data[1];
    let local_encoded = encode_local(address_data[0]);

    let directory = match SETTINGS.variant {
        Variant::Advanced => format!(".well-known/openpgpkey/{}/hu/{}", domain, local_encoded),
        Variant::Direct => format!(".well-known/openpgpkey/hu/{}", local_encoded),
    };

    Ok(PathBuf::from(ROOT_FOLDER).join(directory))
}

pub fn insert_key(cert: &ValidCert) -> Result<()> {
    let path = email_to_file_path(&get_email_from_cert(cert)?)?;

    fs::create_dir_all(path.parent().unwrap())?;
    let mut file = fs::File::create(&path)?;
    cert.export(&mut file)?;

    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path.parent().unwrap().parent().unwrap().join("policy"))?;

    Ok(())
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
    let domain = extract_domain(email)?;
    let allowed = SETTINGS.allowed_domains.contains(&domain);
    if !allowed {
        debug!("User {} was rejected: domain not whitelisted", email);
        Err(SpecialErrors::UnallowedDomain)?;
    }
    Ok(())
}

pub fn parse_pem(pemfile: &str) -> Result<Cert> {
    let cert = match log_err!(sequoia_openpgp::Cert::from_bytes(pemfile.as_bytes()), debug) {
        Ok(cert) => cert,
        Err(_) => Err(SpecialErrors::MalformedCert)?,
    };
    validate_cert(&cert)?;
    Ok(cert)
}

pub fn gen_random_token() -> String {
    let mut rng = thread_rng();
    (0..10).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn get_email_from_cert(cert: &ValidCert) -> Result<String> {
    let userid_opt = log_err!(cert.primary_userid(), debug)?;
    let email_opt = userid_opt.email()?;
    match email_opt {
        Some(email) => Ok(email),
        None => log_err!(Err(SpecialErrors::EmailMissing), debug)?,
    }
}

pub fn extract_domain(email: &str) -> Result<String> {
    let domain = match email.split('@').last() {
        Some(domain) => domain.to_string(),
        None => {
            debug!("Unable to extract domain from {}, email malformed", email);
            Err(SpecialErrors::MalformedEmail)?
        }
    };
    Ok(domain)
}

pub fn key_exists(email: &str) -> Result<bool> {
    let path = email_to_file_path(email)?;
    if !path.is_file() {
        debug!("No key found for user {}", email);
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

pub fn custom_file_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] [{}] {} {}:{}: {}",
        now.format("%Y-%m-%d %H:%M:%S"),
        record.module_path().unwrap_or("<unnamed>"),
        record.level(),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        &record.args()
    )
}

pub fn init_logger() -> Result<LoggerHandle, FlexiLoggerError> {
    Logger::try_with_env_or_str("simple_wkd=debug")?
        .log_to_file(FileSpec::default().directory("logs"))
        .duplicate_to_stdout(flexi_logger::Duplicate::All)
        .format_for_files(custom_file_format)
        .adaptive_format_for_stdout(flexi_logger::AdaptiveFormat::Custom(
            custom_monochrome_format,
            custom_color_format,
        ))
        .set_palette("b1;3;2;4;6".to_string())
        .start()
}

pub fn return_outcome(data: Result<&str, &CompatErr>) -> Result<HttpResponse> {
    let path = webpage_path().join("status").join("index.html");
    let template = log_err!(read_file(&path), error, true)?;
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
