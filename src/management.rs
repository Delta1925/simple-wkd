use crate::errors::Error;
use crate::pending_path;
use crate::settings::SETTINGS;
use crate::utils::get_user_file_path;
use crate::PENDING_FOLDER;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Add,
    Delete,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pending {
    action: Action,
    data: String,
    timestamp: i64,
}
impl Pending {
    pub fn build_add(pem: String) -> Self {
        let timestamp = Utc::now().timestamp();
        Self {
            action: Action::Add,
            data: pem,
            timestamp,
        }
    }
    pub fn build_delete(email: String) -> Self {
        let timestamp = Utc::now().timestamp();
        Self {
            action: Action::Delete,
            data: email,
            timestamp,
        }
    }
    pub const fn action(&self) -> &Action {
        &self.action
    }
    pub fn data(&self) -> &str {
        &self.data
    }
    pub const fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

fn store_pending(pending: &Pending, token: &str) -> Result<(), Error> {
    let serialized = match serde_json::to_string(pending) {
        Ok(serialized) => serialized,
        Err(_) => return Err(Error::SerializeData),
    };
    match fs::write(pending_path!().join(token), serialized) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Inaccessible),
    }
}

pub fn store_pending_addition(pem: String, token: &str) -> Result<(), Error> {
    let pending = Pending::build_add(pem);
    store_pending(&pending, token)?;
    Ok(())
}

pub fn store_pending_deletion(email: String, token: &str) -> Result<(), Error> {
    let pending = Pending::build_delete(email);
    store_pending(&pending, token)?;
    Ok(())
}

pub fn clean_stale(max_age: i64) -> Result<(), Error> {
    for path in fs::read_dir(pending_path!()).unwrap().flatten() {
        let file_path = path.path();
        if file_path.is_file() {
            let content = match fs::read_to_string(&file_path) {
                Ok(content) => content,
                Err(_) => return Err(Error::Inaccessible),
            };
            let key = match serde_json::from_str::<Pending>(&content) {
                Ok(key) => key,
                Err(_) => return Err(Error::DeserializeData),
            };
            let now = Utc::now().timestamp();
            if now - key.timestamp() > max_age {
                let err = fs::remove_file(&file_path).is_err();
                if err {
                    return Err(Error::Inaccessible);
                }
            }
        }
    }
    Ok(())
}

pub fn delete_key(email: &str) -> Result<(), Error> {
    let path = Path::new(&SETTINGS.root_folder).join(get_user_file_path(email)?);
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Inaccessible),
    }
}
