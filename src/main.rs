extern crate core;

mod models;
mod controllers;
mod services;
mod error;
mod context;
mod middleware;

use std::env;
use std::net::SocketAddr;
use axum::{Router, Server};
use axum::body::HttpBody;
use axum::http::{header, Method};
use axum::routing::{delete, get, post};
use mongodb::{Client, Collection, options::ClientOptions};
use dotenv::dotenv;
use crate::controllers::auth_controller::{logout, sign_up, sing_in, test_middleware};
use crate::models::user_model::{User};
use crate::services::auth_services::UserCollection;
use tower_http::cors;
use tower_http::cors::{Any, CorsLayer};
use tower_cookies::CookieManagerLayer;
use axum::middleware as axum_middleware;
use crate::controllers::file_controllers::{delete_file, get_files, upload_file};
use crate::middleware::auth_middleware::verify_token;
use crate::services::file_services::FileCollection;
use crate::services::trait_service::StorageCollection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let cors = CorsLayer::new().allow_origin(Any);

    let user_collection = UserCollection::init().await?;


    let auth_router = Router::new()
        .route("/auth/singup", post(sign_up))
        .route("/auth/signin", post(sing_in))
        .route("/auth/logout", post(logout))
        .with_state(user_collection);


    let files_collection = FileCollection::init().await?;

    let files_router = Router::new()
        .route("/files", get(get_files))
        .route("/upload", post(upload_file))
        .route("/delete", delete(delete_file))
        .route_layer(axum_middleware::from_fn(verify_token))
        .with_state(files_collection);


    let app = Router::new()
        .nest_service("/", auth_router)
        .nest("/api", files_router)
        .layer(cors)
        .layer(CookieManagerLayer::new());


    let addr = "127.0.0.1:8080".parse().unwrap();
    println!("App is running at {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
