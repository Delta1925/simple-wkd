use crate::errors::Error;
use crate::VARIANT;

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sequoia_net::wkd::Url;
use sequoia_openpgp::{parse::Parse, Cert};
use std::path::PathBuf;

#[macro_export]
macro_rules! pending_path {
    () => {
        Path::new(PATH).join(PENDING)
    };
}

pub fn parse_pem(data: &str) -> Result<Cert, Error> {
    match sequoia_openpgp::Cert::from_bytes(data.as_bytes()) {
        Ok(data) => Ok(data),
        Err(_) => Err(Error::ParseCert),
    }
}

pub fn gen_random_token() -> String {
    let mut rng = thread_rng();
    (0..10).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn get_email_from_cert(cert: &Cert) -> Result<String, Error> {
    match cert.userids().next() {
        Some(data) => match data.email() {
            Ok(data) => match data {
                Some(data) => Ok(data),
                None => Err(Error::MissingMail),
            },
            Err(_) => Err(Error::ParseMail),
        },
        None => Err(Error::ParseCert),
    }
}

pub fn get_user_file_path(email: &str) -> Result<PathBuf, Error> {
    match Url::from(email) {
        Ok(data) => match data.to_file_path(VARIANT) {
            Ok(data) => Ok(data),
            Err(_) => Err(Error::ParseMail),
        },
        Err(_) => Err(Error::ParseMail),
    }
}
