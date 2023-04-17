mod confirmation;
mod errors;
mod management;
mod settings;
mod utils;

use crate::confirmation::{confirm_action, send_confirmation_email};
use crate::errors::CompatErr;
use crate::management::{clean_stale, store_pending_addition, store_pending_deletion, Action};
use crate::settings::{ROOT_FOLDER, SETTINGS};
use crate::utils::{
    gen_random_token, get_email_from_cert, is_email_allowed, parse_pem, return_outcome, read_file,
};

use actix_files::Files;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{
    get, post, web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Result,
};
use anyhow::anyhow;
use errors::SpecialErrors;
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Ok(value) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", format!("simple_wkd={}", value));
    }
    if init_logger().is_err() {
        panic!("Could not set up logger!")
    };
    fs::create_dir_all(pending_path!())?;
    task::spawn(async {
        let mut metronome = time::interval(time::Duration::from_secs(SETTINGS.cleanup_interval));
        loop {
            metronome.tick().await;
            clean_stale(SETTINGS.max_age);
        }
    });
    HttpServer::new(|| {
        App::new()
            .service(submit)
            .service(confirm)
            .service(delete)
            .service(
                Files::new("/.well-known", Path::new(&ROOT_FOLDER).join(".well-known"))
                    .use_hidden_files(),
            )
            .route("/{filename:.*}", web::get().to(index))
    })
    .bind((SETTINGS.bind_host.to_string(), SETTINGS.port))?
    .run()
    .await
}

async fn index(req: HttpRequest) -> Result<HttpResponse, CompatErr> {
    let path = webpage_path!().join(req.match_info().query("filename"));
    for file in &["", "index.html"] {
        let path = if file.is_empty() {
            path.to_owned()
        } else {
            path.join(file)
        };
        if path.is_file() {
            let template = read_file(&path)?;
            let page = template.replace("((%u))", SETTINGS.external_url.as_ref());
            return Ok(HttpResponseBuilder::new(StatusCode::OK)
                .insert_header(ContentType::html())
                .body(page));
        }
    }
    Err(SpecialErrors::MissingFile)?
}

#[post("/api/submit")]
async fn submit(pem: web::Form<Key>) -> Result<HttpResponse, CompatErr> {
    let cert = parse_pem(&pem.key)?;
    let email = get_email_from_cert(&cert)?;
    is_email_allowed(&email)?;
    let token = gen_random_token();
    store_pending_addition(pem.key.clone(), &email, &token)?;
    send_confirmation_email(&email, &Action::Add, &token)?;
    Ok(return_outcome(Ok("You submitted your key successfully!"))?)
}

#[get("/api/confirm")]
async fn confirm(token: web::Query<Token>) -> Result<HttpResponse, CompatErr> {
    let (action, _email) = confirm_action(&token.token)?;
    match action {
        Action::Add => {
            Ok(return_outcome(Ok("Your key was added successfully!"))?)
        }
        Action::Delete => {
            Ok(return_outcome(Ok("Your key was deleted successfully!"))?)
        }
    }
}

#[get("/api/delete")]
async fn delete(email: web::Query<Email>) -> Result<HttpResponse, CompatErr> {
    let token = gen_random_token();
    store_pending_deletion(email.email.clone(), &token)?;
    send_confirmation_email(&email.email, &Action::Delete, &token)?;
    Ok(return_outcome(Ok("You requested the deletion of your key successfully!"))?)
}
