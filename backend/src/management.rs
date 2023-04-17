use crate::pending_path;
use crate::settings::ROOT_FOLDER;
use crate::utils::{get_user_file_path, key_exists, read_file};

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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

fn store_pending(pending: &Pending, token: &str) -> Result<()> {
    let serialized = toml::to_string(pending)?;
    fs::write(pending_path!().join(token), serialized)?;
    Ok(())
}

pub fn store_pending_addition(pem: String, _email: &str, token: &str) -> Result<()> {
    let pending = Pending::build_add(pem);
    store_pending(&pending, token)?;
    Ok(())
}

pub fn store_pending_deletion(email: String, token: &str) -> Result<()> {
    key_exists(&email)?;
    let pending = Pending::build_delete(email);
    store_pending(&pending, token)?;
    Ok(())
}

pub fn clean_stale(max_age: i64) {
    for path in fs::read_dir(pending_path!()).unwrap().flatten() {
        let file_path = path.path();
        let content = match read_file(&file_path) {
            Ok(content) => content,
            Err(_) => {
                continue;
            }
        };
        let key = match toml::from_str::<Pending>(&content) {
            Ok(key) => key,
            Err(_) => {
                continue;
            }
        };
        let now = Utc::now().timestamp();
        if now - key.timestamp() > max_age {
            let _ = fs::remove_file(&file_path);
        }
    }
}

pub fn delete_key(email: &str) -> Result<()> {
    let path = Path::new(&ROOT_FOLDER).join(get_user_file_path(email)?);
    fs::remove_file(path)?;
    Ok(())
}
