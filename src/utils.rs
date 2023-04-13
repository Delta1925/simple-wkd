use crate::errors::Error;
use crate::{PATH, PENDING, VARIANT};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sequoia_net::wkd::Url;
use sequoia_openpgp::{parse::Parse, Cert};
use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! pending_path {
    () => {
        Path::new(PATH).join(PENDING)
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
        Err(_) => return Err(Error::ParseMail),
    };
    match email_opt {
        Some(email) => Ok(email),
        None => Err(Error::MissingMail),
    }
}

pub fn get_user_file_path(email: &str) -> Result<PathBuf, Error> {
    let wkd_url = match Url::from(email) {
        Ok(wkd_url) => wkd_url,
        Err(_) => return Err(Error::ParseMail),
    };
    match wkd_url.to_file_path(VARIANT) {
        Ok(path) => Ok(path),
        Err(_) => Err(Error::ParseMail),
    }
}

pub fn key_exists(email: &str) -> Result<bool, Error> {
    let path = get_user_file_path(email)?;
    if !pending_path!().join(path).is_file() {
        return Err(Error::MissingKey);
    }
    Ok(true)
}
