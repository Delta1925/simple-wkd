use chrono::Utc;
use lettre::message::header::ContentType;

use crate::errors::SpecialErrors;
use crate::management::{delete_key, Action, Pending};
use crate::pending_path;
use crate::settings::{MAILER, ROOT_FOLDER, SETTINGS};
use crate::utils::{get_email_from_cert, parse_pem, read_file};
use anyhow::Result;

use lettre::{Message, Transport};
use std::fs;
use std::path::Path;

pub fn confirm_action(token: &str) -> Result<(Action, String)> {
    let pending_path = pending_path!().join(token);
    let content = read_file(&pending_path)?;
    let key = toml::from_str::<Pending>(&content)?;
    if Utc::now().timestamp() - key.timestamp() > SETTINGS.max_age {
        fs::remove_file(&pending_path)?;
        Err(SpecialErrors::ExpiredRequest)?
    } else {
        let address = match key.action() {
            Action::Add => {
                let cert = parse_pem(key.data())?;
                let email = get_email_from_cert(&cert)?;
                let domain = match email.split('@').last() {
                    Some(domain) => domain.to_string(),
                    None => Err(SpecialErrors::MalformedEmail)?,
                };
                sequoia_net::wkd::insert(ROOT_FOLDER, domain, SETTINGS.variant, &cert)?;
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
        .to(match address.parse() {
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
        )?;

    MAILER.send(&email)?;
    Ok(())
}
