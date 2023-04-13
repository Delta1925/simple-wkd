use crate::errors::Error;
use crate::management::{delete_key, Action, Pending};
use crate::utils::{get_email_from_cert, parse_pem};
use crate::PENDING;
use crate::{pending_path, PATH, VARIANT};

use std::fs;
use std::path::Path;

pub fn confirm_action(token: &str) -> Result<(), Error> {
    let pending_path = pending_path!().join(token);
    let content = if pending_path.is_file() {
        match fs::read_to_string(&pending_path) {
            Ok(content) => content,
            Err(_) => return Err(Error::Inaccessible),
        }
    } else {
        return Err(Error::MissingPath);
    };
    let key = match serde_json::from_str::<Pending>(&content) {
        Ok(key) => key,
        Err(_) => return Err(Error::ParseStored),
    };
    match key.action() {
        Action::Add => {
            let cert = parse_pem(key.data())?;
            let domain = match get_email_from_cert(&cert)?.split('@').last() {
                Some(domain) => domain.to_string(),
                None => return Err(Error::MalformedMail),
            };
            match sequoia_net::wkd::insert(PATH, domain, VARIANT, &cert) {
                Ok(_) => (),
                Err(_) => return Err(Error::AddingKey),
            }
        }
        Action::Delete => delete_key(key.data())?,
    }
    match fs::remove_file(&pending_path) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Inaccessible),
    }
}

pub fn send_confirmation_email(email: &str, action: &Action, token: &str) {
    println!("Email sent to {email}");
    println!("Action: {action:?}");
    println!("Token: {token}");
}
