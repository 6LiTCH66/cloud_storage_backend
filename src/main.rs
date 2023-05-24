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
use crate::controllers::auth_controller::{get_user, logout, sign_up, sing_in, test_middleware};
use crate::models::user_model::{User};
use crate::services::auth_services::UserCollection;
use tower_http::cors;
use tower_http::cors::{Any, CorsLayer, AllowOrigin, AllowMethods};
use tower_cookies::CookieManagerLayer;
use axum::middleware as axum_middleware;
use crate::controllers::file_controllers::{delete_file, get_files, upload_file};
use crate::middleware::auth_middleware::verify_token;
use crate::services::file_services::FileCollection;
use crate::services::trait_service::StorageCollection;
use axum::{http::HeaderValue};

async fn root() -> &'static str {
    "Hello, World!"
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact("http://localhost:3000".parse().unwrap()))
        .allow_headers(vec![
        header::ACCEPT,
        header::ACCEPT_LANGUAGE,
        header::AUTHORIZATION,
        header::CONTENT_LANGUAGE,
        header::CONTENT_TYPE,
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::ACCESS_CONTROL_ALLOW_HEADERS
    ]).allow_methods(AllowMethods::exact(Method::DELETE))
        .allow_credentials(true)
        .expose_headers(vec![header::LOCATION]);



    let user_collection = UserCollection::init().await?;


    let auth_router = Router::new()
        .route("/signup", post(sign_up))
        .route("/signin", post(sing_in))
        .route("/logout", post(logout))
        .with_state(user_collection.clone());



    let files_collection = FileCollection::init().await?;

    let files_router = Router::new()
        .route("/files", get(get_files))
        .route("/upload", post(upload_file))
        .route("/delete", delete(delete_file))

        .route_layer(axum_middleware::from_fn(verify_token))
        .with_state(files_collection);


    let user_router = Router::new()
        .route("/get-user", get(get_user))
        .route_layer(axum_middleware::from_fn(verify_token))
        .with_state(user_collection.clone());


    let app = Router::new()
        .route("/", get(root))
        .nest_service("/auth", auth_router)
        .nest("/api", files_router)
        .nest("/user", user_router)
        .layer(cors)
        .layer(CookieManagerLayer::new());


    let addr = "0.0.0.0:3000".parse().unwrap();
    println!("App is running at {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
