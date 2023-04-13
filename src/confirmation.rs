use crate::management::{delete_key, Action, Pending};
use crate::utils::{get_email_from_cert, parse_pem};
use crate::PENDING;
use crate::{pending_path, PATH, VARIANT};

use std::fs;
use std::path::Path;

pub fn confirm_action(token: &str) {
    let pending_path = pending_path!().join(token);
    let key: Pending = serde_json::from_str(&fs::read_to_string(&pending_path).unwrap()).unwrap();
    match key.action() {
        Action::Add => {
            let cert = parse_pem(key.data());
            let domain = get_email_from_cert(&cert)
                .split('@')
                .last()
                .unwrap()
                .to_owned();
            sequoia_net::wkd::insert(PATH, domain, VARIANT, &cert).unwrap();
        }
        Action::Delete => delete_key(key.data()),
    }
    fs::remove_file(&pending_path).unwrap();
}

pub fn send_confirmation_email(email: &str, action: Action, token: &str) {
    println!("Email sent to {}", email);
    println!("Action: {:?}", action);
    println!("Token: {}", token);
}
