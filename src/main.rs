mod confirmation;
mod errors;
mod management;
mod settings;
mod utils;

use crate::settings::SETTINGS;
use crate::utils::is_email_allowed;

use self::confirmation::{confirm_action, send_confirmation_email};
use self::management::{clean_stale, store_pending_addition, store_pending_deletion, Action};
use self::utils::{gen_random_token, get_email_from_cert, parse_pem};

use actix_web::{get, post, web, App, HttpServer, Result};
use log::{error, info};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use tokio::{task, time};
use utils::init_logger;

const PENDING_FOLDER: &str = "pending";

#[derive(Deserialize, Debug)]
struct Key {
    key: String,
}

#[derive(Deserialize, Debug)]
struct Token {
    token: String,
}

#[derive(Deserialize, Debug)]
struct Email {
    email: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Ok(value) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", format!("simple_wkd={}", value));
    }
    if init_logger().is_err() {
        error!("Could not set up logger!");
        panic!("Could not set up logger!")
    };
    fs::create_dir_all(pending_path!())?;
    task::spawn(async {
        let mut metronome = time::interval(time::Duration::from_secs(SETTINGS.cleanup_interval));
        loop {
            metronome.tick().await;
            info!("Running cleanup...");
            clean_stale(SETTINGS.max_age);
            info!("Cleanup completed!");
        }
    });
    info!(
        "Running server on http://127.0.0.1:{} (External URL: {})",
        SETTINGS.port, SETTINGS.external_url
    );
    HttpServer::new(|| App::new().service(submit).service(confirm).service(delete))
        .bind(("127.0.0.1", SETTINGS.port))?
        .run()
        .await
}

#[post("/api/submit")]
async fn submit(pem: web::Form<Key>) -> Result<String> {
    let cert = parse_pem(&pem.key)?;
    let email = get_email_from_cert(&cert)?;
    is_email_allowed(&email)?;
    let token = gen_random_token();
    store_pending_addition(pem.key.clone(), &email, &token)?;
    send_confirmation_email(&email, &Action::Add, &token)?;
    info!("User {} submitted a key!", &email);
    Ok(String::from("(0x00) Key submitted successfully!"))
}

#[get("/api/confirm")]
async fn confirm(token: web::Query<Token>) -> Result<String> {
    let (action, email) = confirm_action(&token.token)?;
    match action {
        Action::Add => info!("Key for user {} was added successfully!", email),
        Action::Delete => info!("Key for user {} was deleted successfully!", email),
    }
    Ok(String::from("(0x00) Confirmation successful!"))
}

#[get("/api/delete")]
async fn delete(email: web::Query<Email>) -> Result<String> {
    let token = gen_random_token();
    store_pending_deletion(email.email.clone(), &token)?;
    send_confirmation_email(&email.email, &Action::Delete, &token)?;
    info!("User {} requested the deletion of his key!", email.email);
    Ok(String::from("(0x00) Deletion request submitted successfully!"))
}
