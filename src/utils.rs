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

pub fn parse_pem(data: &str) -> Cert {
    sequoia_openpgp::Cert::from_bytes(data.as_bytes()).unwrap()
}

pub fn gen_random_token() -> String {
    let mut rng = thread_rng();
    (0..10).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn get_email_from_cert(cert: &Cert) -> String {
    cert.userids().next().unwrap().email().unwrap().unwrap()
}

pub fn get_user_file_path(email: &str) -> PathBuf {
    Url::from(email).unwrap().to_file_path(VARIANT).unwrap()
}
