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
    pub fn build_add(pem: String) -> Pending {
        let timestamp = Utc::now().timestamp();
        Pending {
            action: Action::Add,
            data: pem,
            timestamp,
        }
    }
    pub fn build_delete(email: String) -> Pending {
        let timestamp = Utc::now().timestamp();
        Pending {
            action: Action::Delete,
            data: email,
            timestamp,
        }
    }
    pub fn action(&self) -> &Action {
        &self.action
    }
    pub fn data(&self) -> &str {
        &self.data
    }
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

fn store_pending(pending: Pending, token: &str) {
    let serialized = serde_json::to_string(&pending).unwrap();
    fs::create_dir_all(pending_path!()).unwrap();
    fs::write(pending_path!().join(token), serialized).unwrap();
}

pub fn store_pending_addition(pem: String, token: &str) {
    let data = Pending::build_add(pem);
    store_pending(data, token);
}

pub fn store_pending_deletion(email: String, token: &str) {
    let data = Pending::build_delete(email);
    store_pending(data, token);
}

pub fn clean_stale(max_age: i64) {
    for path in fs::read_dir(pending_path!()).unwrap() {
        let file_path = path.unwrap().path();
        let key: Pending = serde_json::from_str(&fs::read_to_string(&file_path).unwrap()).unwrap();
        let now = Utc::now().timestamp();
        if now - key.timestamp() > max_age {
            fs::remove_file(&file_path).unwrap();
            println!(
                "Deleted {}, since it was stale",
                &file_path.to_str().unwrap()
            );
        }
    }
}

pub fn delete_key(email: &str) {
    let path = Path::new(PATH).join(get_user_file_path(email));
    fs::remove_file(path).unwrap();
}
