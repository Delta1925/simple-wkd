mod confirmation;
mod errors;
mod management;
mod settings;
mod utils;

use crate::errors::Error;
use crate::settings::SETTINGS;
use crate::utils::is_email_allowed;

use self::confirmation::{confirm_action, send_confirmation_email};
use self::management::{clean_stale, store_pending_addition, store_pending_deletion, Action};
use self::utils::{gen_random_token, get_email_from_cert, parse_pem};

use actix_files::Files;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{
    get, post, web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Result,
};
use log::{debug, error, info};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use tokio::{task, time};
use utils::init_logger;

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

fn return_success(message: &str) -> Result<HttpResponse, Error> {
    let path = webpage_path!().join("success").join("index.html");
    let template = match fs::read_to_string(&path) {
        Ok(template) => template,
        Err(_) => {
            debug!("file {} is inaccessible", path.display());
            return Err(Error::Inaccessible);
        }
    };
    let page = template.replace("((%m))", message);
    return Ok(HttpResponseBuilder::new(StatusCode::OK)
        .insert_header(ContentType::html())
        .body(page));
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
    HttpServer::new(|| {
        App::new()
            .service(submit)
            .service(confirm)
            .service(delete)
            .service(
                Files::new(
                    "/.well-known",
                    Path::new(&SETTINGS.root_folder).join(".well-known"),
                )
                .use_hidden_files(),
            )
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind(("127.0.0.1", SETTINGS.port))?
    .run()
    .await
}

async fn index(req: HttpRequest) -> Result<HttpResponse, Error> {
    let path = webpage_path!().join(req.match_info().query("filename"));
    for file in &["", "index.html"] {
        let path = if file.is_empty() {
            path.to_owned()
        } else {
            path.join(file)
        };
        if path.is_file() {
            let template = match fs::read_to_string(&path) {
                Ok(template) => template,
                Err(_) => {
                    debug!("File {} is inaccessible", path.display());
                    return Err(Error::Inaccessible);
                }
            };
            let page = template.replace("((%u))", SETTINGS.external_url.as_ref());
            return Ok(HttpResponseBuilder::new(StatusCode::OK)
                .insert_header(ContentType::html())
                .body(page));
        }
    }
    debug!("File {} does not exist", path.display());
    Err(Error::MissingFile)
}

#[post("/api/submit")]
async fn submit(pem: web::Form<Key>) -> Result<HttpResponse, Error> {
    let cert = parse_pem(&pem.key)?;
    let email = get_email_from_cert(&cert)?;
    is_email_allowed(&email)?;
    let token = gen_random_token();
    store_pending_addition(pem.key.clone(), &email, &token)?;
    send_confirmation_email(&email, &Action::Add, &token)?;
    info!("User {} submitted a key!", &email);
    return_success("You submitted your key successfully!")
}

#[get("/api/confirm")]
async fn confirm(token: web::Query<Token>) -> Result<HttpResponse, Error> {
    let (action, email) = confirm_action(&token.token)?;
    match action {
        Action::Add => {
            info!("Key for user {} was added successfully!", email);
            return_success("Your key was added successfully!")
        }
        Action::Delete => {
            info!("Key for user {} was deleted successfully!", email);
            return_success("Your key was deleted successfully!")
        }
    }
}

#[get("/api/delete")]
async fn delete(email: web::Query<Email>) -> Result<HttpResponse, Error> {
    let token = gen_random_token();
    store_pending_deletion(email.email.clone(), &token)?;
    send_confirmation_email(&email.email, &Action::Delete, &token)?;
    info!("User {} requested the deletion of his key!", email.email);
    return_success("You requested the deletion of your key successfully!")
}
