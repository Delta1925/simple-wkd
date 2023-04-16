use chrono::Utc;
use lettre::message::header::ContentType;
use log::{debug, error, trace, warn};

use crate::errors::Error;
use crate::management::{delete_key, Action, Pending};
use crate::pending_path;
use crate::settings::{MAILER, ROOT_FOLDER, SETTINGS};
use crate::utils::{get_email_from_cert, get_filename, parse_pem};

use lettre::{Message, Transport};
use std::fs;
use std::path::Path;

pub fn confirm_action(token: &str) -> Result<(Action, String), Error> {
    trace!("Handling token {}", token);
    let pending_path = pending_path!().join(token);
    let content = if pending_path.is_file() {
        match fs::read_to_string(&pending_path) {
            Ok(content) => content,
            Err(_) => {
                warn!(
                    "Token {} was requested, but can't be read to string!",
                    token
                );
                return Err(Error::Inaccessible);
            }
        }
    } else {
        trace!("Requested token {} isn't a file", token);
        return Err(Error::MissingPending);
    };
    let key = match toml::from_str::<Pending>(&content) {
        Ok(key) => key,
        Err(_) => {
            warn!("Error while deserializing token {}!", token);
            return Err(Error::DeserializeData);
        }
    };
    if Utc::now().timestamp() - key.timestamp() > SETTINGS.max_age {
        match fs::remove_file(&pending_path) {
            Ok(_) => {
                debug!(
                    "Deleted stale token {}",
                    get_filename(&pending_path).unwrap()
                );
                Err(Error::MissingPending)
            }
            Err(_) => {
                warn!("Stale token {} can't be deleted!", token);
                Err(Error::Inaccessible)
            }
        }
    } else {
        let address = match key.action() {
            Action::Add => {
                let cert = parse_pem(key.data())?;
                let email = get_email_from_cert(&cert)?;
                let domain = match email.split('@').last() {
                    Some(domain) => domain.to_string(),
                    None => {
                        warn!("Error while parsing email's domain in token {}", token);
                        return Err(Error::ParseEmail);
                    }
                };
                match sequoia_net::wkd::insert(ROOT_FOLDER, domain, SETTINGS.variant, &cert) {
                    Ok(_) => email,
                    Err(_) => {
                        warn!("Unable to create a wkd entry for token {}", token);
                        return Err(Error::AddingKey);
                    }
                }
            }
            Action::Delete => match delete_key(key.data()) {
                Ok(_) => key.data().to_owned(),
                Err(error) => {
                    warn!("Unable to delete key for user {}", key.data());
                    return Err(error);
                }
            },
        };
        debug!("Token {} was confirmed", token);
        match fs::remove_file(&pending_path) {
            Ok(_) => {
                trace!(
                    "Deleted confirmed token {}",
                    pending_path.file_name().unwrap().to_str().unwrap()
                );
                Ok((*key.action(), address))
            }
            Err(_) => {
                warn!("Unable to delete confirmed token {}", token);
                Err(Error::Inaccessible)
            }
        }
    }
}

pub fn send_confirmation_email(address: &str, action: &Action, token: &str) -> Result<(), Error> {
    debug!("Sending email to {}", address);
    let template = fs::read_to_string(Path::new("assets").join("mail-template.html")).unwrap();
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
                error!("Unable to parse the email in the settings!");
                panic!("Unable to parse the email in the settings!")
            }
        })
        .to(match address.parse() {
            Ok(mailbox) => mailbox,
            Err(_) => {
                warn!("Error while parsing destination email for token {}", token);
                return Err(Error::ParseEmail);
            }
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

    let message = match email {
        Ok(message) => message,
        Err(_) => {
            warn!("Unable to build email for token {}", token);
            return Err(Error::MailGeneration);
        }
    };

    match MAILER.send(&message) {
        Ok(_) => {
            debug!("successfully sent email to {}", address);
            Ok(())
        }
        Err(_) => {
            warn!("Unable to send email to {}", address);
            Err(Error::SendMail)
        }
    }
}
