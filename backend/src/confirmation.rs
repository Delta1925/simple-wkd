use chrono::Utc;
use lettre::message::header::ContentType;
use log::{warn, debug};

use crate::errors::SpecialErrors;
use crate::management::{delete_key, Action, Pending};
use crate::settings::{MAILER, ROOT_FOLDER, SETTINGS};
use crate::utils::{extract_domain, get_email_from_cert, parse_pem, read_file};
use crate::{log_err, pending_path};
use anyhow::Result;

use lettre::{Message, Transport};
use std::fs;
use std::path::Path;

pub fn confirm_action(token: &str) -> Result<(Action, String)> {
    let pending_path = pending_path().join(token);
    let content = read_file(&pending_path)?;
    let key = log_err!(toml::from_str::<Pending>(&content), warn)?;
    if Utc::now().timestamp() - key.timestamp() > SETTINGS.max_age {
        log_err!(fs::remove_file(&pending_path), warn)?;
        Err(SpecialErrors::ExpiredRequest)?
    } else {
        let address = match key.action() {
            Action::Add => {
                let cert = parse_pem(key.data())?;
                let email = get_email_from_cert(&cert)?;
                let domain = extract_domain(&email)?;
                log_err!(
                    sequoia_net::wkd::insert(ROOT_FOLDER, domain, SETTINGS.variant, &cert),
                    warn
                )?;
                email
            }
            Action::Delete => {
                delete_key(key.data())?;
                key.data().to_owned()
            }
        };
        fs::remove_file(&pending_path)?;
        Ok((*key.action(), address))
    }
}

pub fn send_confirmation_email(address: &str, action: &Action, token: &str) -> Result<()> {
    let template = read_file(&Path::new("assets").join("mail-template.html"))?;
    let mut url = SETTINGS
        .external_url
        .join("api/")
        .unwrap()
        .join("confirm")
        .unwrap();
    url.set_query(Some(&format!("token={}", token)));
    let email = Message::builder()
        .from(match SETTINGS.mail_settings.mail_from.parse() {
            Ok(mailbox) => mailbox,
            Err(_) => {
                panic!("Unable to parse the email in the settings!")
            }
        })
        .to(match log_err!(address.parse(), debug) {
            Ok(mbox) => mbox,
            Err(_) => Err(SpecialErrors::MalformedEmail)?,
        })
        .subject(
            SETTINGS
                .mail_settings
                .mail_subject
                .replace("%a", &action.to_string().to_lowercase()),
        )
        .header(ContentType::TEXT_HTML)
        .body(
            template
                .replace("{{%u}}", url.as_ref())
                .replace("{{%a}}", &action.to_string().to_lowercase()),
        );

    let email = log_err!(email, warn)?;

    match log_err!(MAILER.send(&email), warn){
        Ok(_) => Ok(()),
        Err(_) => Err(SpecialErrors::MailErr)?
    }
}
