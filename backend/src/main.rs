mod confirmation;
mod errors;
mod management;
mod settings;
mod utils;

use crate::confirmation::{confirm_action, send_confirmation_email};
use crate::errors::CompatErr;
use crate::errors::SpecialErrors;
use crate::management::{clean_stale, store_pending_addition, store_pending_deletion, Action};
use crate::settings::{ROOT_FOLDER, SETTINGS};
use crate::utils::{
    gen_random_token, get_email_from_cert, is_email_allowed, parse_pem, read_file, return_outcome, key_exists,
};

use actix_files::Files;
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{
    get, post, web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Result,
};
use log::{debug, error, info, trace};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tokio::{task, time};
use utils::{init_logger, pending_path, webpage_path};

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
    if init_logger().is_err() {
        panic!("Could not set up logger!")
    };
    log_err!(fs::create_dir_all(pending_path()), error)?;
    task::spawn(async {
        let mut metronome = time::interval(time::Duration::from_secs(SETTINGS.cleanup_interval));
        loop {
            metronome.tick().await;
            debug!("Cleaning up stale data...");
            clean_stale(SETTINGS.max_age);
            debug!("Cleanup completed!")
        }
    });
    debug!("Starting server...");
    let server = HttpServer::new(|| {
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
    .run();
    debug!("Server started successfully!");
    info!(
        "Listening on: {}:{} (External url: {})",
        SETTINGS.bind_host, SETTINGS.port, SETTINGS.external_url
    );
    server.await
}

async fn index(req: HttpRequest) -> Result<HttpResponse, CompatErr> {
    let path = webpage_path().join(req.match_info().query("filename"));
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
    trace!("The requested file {} could not be found", path.display());
    Err(SpecialErrors::MissingFile)?
}

#[post("/api/submit")]
async fn submit(pem: web::Form<Key>) -> Result<HttpResponse, CompatErr> {
    let cert = parse_pem(&pem.key)?;
    let email = get_email_from_cert(&cert)?;
    debug!("Handling user {} request to add a key...", email);
    is_email_allowed(&email)?;
    let token = gen_random_token();
    store_pending_addition(pem.key.clone(), &email, &token)?;
    debug!("Sending email to {} to add a key... (Request token: {})", email, token);
    send_confirmation_email(&email, &Action::Add, &token)?;
    info!("User {} requested to add a key successfully!", email);
    Ok(return_outcome(Ok("You submitted your key successfully!"))?)
}

#[get("/api/confirm")]
async fn confirm(token: web::Query<Token>) -> Result<HttpResponse, CompatErr> {
    debug!("Handling token {}...", token.token);
    let (action, email) = confirm_action(&token.token)?;
    info!("User {} confirmed to {} his key successfully!", email, action.to_string().to_lowercase());
    match action {
        Action::Add => Ok(return_outcome(Ok("Your key was added successfully!"))?),
        Action::Delete => Ok(return_outcome(Ok("Your key was deleted successfully!"))?),
    }
}

#[get("/api/delete")]
async fn delete(email: web::Query<Email>) -> Result<HttpResponse, CompatErr> {
    debug!("Handling user {} request to add a key...", email.email);
    key_exists(&email.email)?;
    let token = gen_random_token();
    store_pending_deletion(email.email.clone(), &token)?;
    debug!("Sending email to {} to add a key... (Request token: {})", email.email, token);
    send_confirmation_email(&email.email, &Action::Delete, &token)?;
    info!("User {} requested to delete his key successfully!", email.email);
    Ok(return_outcome(Ok(
        "You requested the deletion of your key successfully!",
    ))?)
}
