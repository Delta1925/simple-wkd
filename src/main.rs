mod confirmation;
mod errors;
mod management;
mod settings;
mod utils;

use crate::settings::SETTINGS;
use crate::utils::key_exists;

use self::confirmation::{confirm_action, send_confirmation_email};
use self::management::{clean_stale, store_pending_addition, store_pending_deletion, Action};
use self::utils::{gen_random_token, get_email_from_cert, parse_pem};

use actix_web::{get, post, web, App, HttpServer, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tokio::{task, time};

const PENDING_FOLDER: &str = "pending";

#[derive(Deserialize, Debug)]
struct Pem {
    key: String,
}

#[derive(Deserialize, Debug)]
struct Token {
    value: String,
}

#[derive(Deserialize, Debug)]
struct Email {
    address: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    fs::create_dir_all(pending_path!())?;
    task::spawn(async {
        let mut metronome = time::interval(time::Duration::from_secs(SETTINGS.cleanup_interval));
        loop {
            metronome.tick().await;
            if clean_stale(SETTINGS.max_age).is_err() {
                eprintln!("Error while cleaning stale requests...");
            }
        }
    });
    HttpServer::new(|| App::new().service(submit).service(confirm).service(delete))
        .bind(("127.0.0.1", SETTINGS.port))?
        .run()
        .await
}

#[post("/api/submit")]
async fn submit(pem: web::Form<Pem>) -> Result<String> {
    let cert = parse_pem(&pem.key)?;
    let email = get_email_from_cert(&cert)?;
    let token = gen_random_token();
    store_pending_addition(pem.key.clone(), &token)?;
    send_confirmation_email(&email, &Action::Add, &token)?;
    Ok(String::from("Key submitted successfully!"))
}

#[get("/api/confirm/{value}")]
async fn confirm(token: web::Path<Token>) -> Result<String> {
    confirm_action(&token.value)?;
    Ok(String::from("Confirmation successful!"))
}

#[get("/api/delete/{address}")]
async fn delete(email: web::Path<Email>) -> Result<String> {
    key_exists(&email.address)?;
    let token = gen_random_token();
    store_pending_deletion(email.address.clone(), &token)?;
    send_confirmation_email(&email.address, &Action::Delete, &token)?;
    Ok(String::from("Deletion request submitted successfully!"))
}
