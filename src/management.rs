use crate::errors::Error;
use crate::utils::get_user_file_path;
use crate::PENDING;
use crate::{pending_path, PATH};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Add,
    Delete,
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
    match serde_json::to_string(pending) {
        Ok(serialized) => match fs::write(pending_path!().join(token), serialized) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::Inaccessible),
        },
        Err(_) => Err(Error::SerializeData),
    }
}

pub fn store_pending_addition(pem: String, token: &str) -> Result<(), Error> {
    let data = Pending::build_add(pem);
    store_pending(&data, token)?;
    Ok(())
}

pub fn store_pending_deletion(email: String, token: &str) -> Result<(), Error> {
    let data = Pending::build_delete(email);
    store_pending(&data, token)?;
    Ok(())
}

pub fn clean_stale(max_age: i64) -> Result<(), Error> {
    for path in fs::read_dir(pending_path!()).unwrap().flatten() {
        let file_path = path.path();
        if file_path.exists() {
            match fs::read_to_string(&file_path) {
                Ok(data) => match serde_json::from_str::<Pending>(&data) {
                    Ok(key) => {
                        let now = Utc::now().timestamp();
                        if now - key.timestamp() > max_age {
                            if fs::remove_file(&file_path).is_err() {
                                return Err(Error::Inaccessible);
                            }
                            println!(
                                "Deleted {}, since it was stale",
                                &file_path.to_str().unwrap()
                            );
                        }
                    }
                    Err(_) => return Err(Error::DeserializeData),
                },
                Err(_) => return Err(Error::Inaccessible),
            }
        }
    }
    Ok(())
}

pub fn delete_key(email: &str) -> Result<(), Error> {
    let path = Path::new(PATH).join(get_user_file_path(email)?);
    match fs::remove_file(path) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Inaccessible),
    }
}
