use chrono::Utc;
use lettre::message::header::ContentType;
use log::{debug, error, warn};

use crate::errors::SpecialErrors;
use crate::management::{delete_key, Action, Pending};
use crate::settings::{MAILER, SETTINGS};
use crate::utils::{get_email_from_cert, insert_key, parse_pem, read_file, validate_cert};
use crate::{log_err, pending_path};
use anyhow::Result;

use lettre::{AsyncTransport, Message};
use std::fs;
use std::path::Path;

pub fn confirm_action(token: &str) -> Result<(Action, String)> {
    let pending_path = pending_path().join(token);
    let content = log_err!(read_file(&pending_path), debug)?;
    let key = log_err!(toml::from_str::<Pending>(&content), warn)?;
    if Utc::now().timestamp() - key.timestamp() > SETTINGS.max_age {
        log_err!(fs::remove_file(&pending_path), warn)?;
        debug!("Token {} was stale", token);
        Err(SpecialErrors::ExpiredRequest)?
    } else {
        let address = match key.action() {
            Action::Add => {
                let cert = parse_pem(key.data())?;
                let validcert = validate_cert(&cert)?;
                let email = get_email_from_cert(&validcert)?;
                log_err!(insert_key(&validcert), warn)?;
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

pub async fn send_confirmation_email(address: &str, action: &Action, token: &str) -> Result<()> {
    let template = log_err!(
        read_file(&Path::new("assets").join("mail-template.html")),
        error,
        true
    )?;
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

    match log_err!(MAILER.send(email).await, warn) {
        Ok(_) => Ok(()),
        Err(_) => Err(SpecialErrors::MailErr)?,
    }
}
