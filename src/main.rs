mod confirmation;
mod errors;
mod management;
mod utils;

use self::confirmation::{confirm_action, send_confirmation_email};
use self::management::{clean_stale, store_pending_addition, store_pending_deletion, Action};
use self::utils::{gen_random_token, get_email_from_cert, parse_pem};

use actix_web::{get, post, web, App, HttpServer, Result};
use sequoia_net::wkd::Variant;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tokio::{task, time};

const PATH: &str = "data";
const PENDING: &str = "pending";
const MAX_AGE: i64 = 0;
const VARIANT: Variant = Variant::Direct;

#[derive(Deserialize, Debug)]
struct Pem {
    key: String,
}

#[derive(Deserialize, Debug)]
struct Token {
    data: String,
}

#[derive(Deserialize, Debug)]
struct Email {
    address: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    fs::create_dir_all(pending_path!())?;
    task::spawn(async {
        let mut metronome = time::interval(time::Duration::from_secs(60 * 60 * 3));
        loop {
            metronome.tick().await;
            clean_stale(MAX_AGE).unwrap();
        }
    });
    HttpServer::new(|| App::new().service(submit).service(confirm).service(delete))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[post("/api/submit")]
async fn submit(pem: web::Form<Pem>) -> Result<String> {
    let cert = parse_pem(&pem.key)?;
    let email = get_email_from_cert(&cert)?;
    let token = gen_random_token();
    store_pending_addition(pem.key.clone(), &token)?;
    send_confirmation_email(&email, &Action::Add, &token);
    Ok(String::from("OK!"))
}

#[get("/api/confirm/{data}")]
async fn confirm(token: web::Path<Token>) -> Result<String> {
    confirm_action(&token.data)?;
    Ok(String::from("OK!"))
}

#[get("/api/delete/{address}")]
async fn delete(email: web::Path<Email>) -> Result<String> {
    let token = gen_random_token();
    store_pending_deletion(email.address.clone(), &token)?;
    send_confirmation_email(&email.address, &Action::Delete, &token);
    Ok(String::from("OK!"))
}
