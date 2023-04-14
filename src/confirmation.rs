use chrono::Utc;

use crate::errors::Error;
use crate::management::{delete_key, Action, Pending};
use crate::pending_path;
use crate::settings::{SMTPEncryption, SETTINGS};
use crate::utils::{get_email_from_cert, parse_pem};
use crate::PENDING_FOLDER;

use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
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
        return Err(Error::MissingPending);
    };
    let key = match serde_json::from_str::<Pending>(&content) {
        Ok(key) => key,
        Err(_) => return Err(Error::DeserializeData),
    };
    if Utc::now().timestamp() - key.timestamp() > SETTINGS.max_age {
        match fs::remove_file(pending_path) {
            Ok(_) => Err(Error::MissingPending),
            Err(_) => Err(Error::Inaccessible),
        }
    } else {
        match key.action() {
            Action::Add => {
                let cert = parse_pem(key.data())?;
                let domain = match get_email_from_cert(&cert)?.split('@').last() {
                    Some(domain) => domain.to_string(),
                    None => return Err(Error::ParseEmail),
                };
                match sequoia_net::wkd::insert(
                    &SETTINGS.root_folder,
                    domain,
                    SETTINGS.variant,
                    &cert,
                ) {
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
}

pub fn send_confirmation_email(email: &str, action: &Action, token: &str) -> Result<(), Error> {
    println!("Sending mail, token: {}", &token);
    let email = Message::builder()
        .from(match SETTINGS.mail_settings.mail_from.parse() {
            Ok(mailbox) => mailbox,
            Err(_) => panic!("Unable to parse the email in the settings!"),
        })
        .to(match email.parse() {
            Ok(mailbox) => mailbox,
            Err(_) => return Err(Error::ParseEmail),
        })
        .subject(&SETTINGS.mail_settings.mail_subject)
        .body(format!("{action} - {token}"));
    let message = match email {
        Ok(message) => message,
        Err(_) => return Err(Error::MailGeneration),
    };
    let creds = Credentials::new(
        SETTINGS.mail_settings.smtp_username.to_owned(),
        SETTINGS.mail_settings.smtp_password.to_owned(),
    );
    let builder = match &SETTINGS.mail_settings.smtp_tls {
        SMTPEncryption::Tls => SmtpTransport::relay(&SETTINGS.mail_settings.smtp_host),
        SMTPEncryption::Starttls => {
            SmtpTransport::starttls_relay(&SETTINGS.mail_settings.smtp_host)
        }
    };
    let mailer = match builder {
        Ok(builder) => builder,
        Err(_) => return Err(Error::SmtpBuilder),
    }
    .credentials(creds)
    .port(SETTINGS.mail_settings.smtp_port)
    .build();
    match mailer.send(&message) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::SendMail),
    }
}
