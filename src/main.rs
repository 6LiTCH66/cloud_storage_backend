extern crate core;

mod models;
mod controllers;
mod services;
mod error;
mod context;
mod middleware;

use std::env;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use axum::{Router, Server};
use axum::body::HttpBody;
use axum::http::{header, Method};
use axum::routing::{delete, get, post};
use mongodb::{Client, Collection, options::ClientOptions};
use dotenv::dotenv;
use crate::controllers::auth_controller::{get_user, logout, sign_up, sing_in};
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
use axum::handler::Handler;
use axum::middleware::AddExtension;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::controllers::dashboard_controllers::get_dashboard;
use crate::controllers::folder_controllers::{create_folder, delete_folder, get_folder_details, get_folders};
use crate::models::file_model::File;
use crate::models::folder_model::Folder;
use crate::services::folder_service::FolderCollection;

async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Clone)]
pub struct AppState {
    pub file_collection: FileCollection,
    pub folder_collection: FolderCollection,
}

#[derive(Serialize, Clone, Debug)]

enum Item<A, B> {
    File(A),
    Folder(B),
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // https://cloud-storage-frontend-murex.vercel.app

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact("https://cloud-storage-frontend-murex.vercel.app".parse().unwrap()))
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


    let folder_collection = FolderCollection::init().await?;
    let files_collection = FileCollection::init().await?;

    let state = Arc::new(AppState {
        file_collection: files_collection.clone(),
        folder_collection: folder_collection.clone(),
    });


    let files_router = Router::new()
        .route("/files", get(get_files))
        .route("/upload", post(upload_file))
        .route("/delete", delete(delete_file))

        .route_layer(axum_middleware::from_fn(verify_token))
        .with_state(state.clone());


    let dashboard_router = Router::new()
        .route("/", get(get_dashboard))
        .route_layer(axum_middleware::from_fn(verify_token))
        .with_state(state.clone());



    let folder_router = Router::new()
        .route("/create", post(create_folder))
        .route("/folders", get(get_folders))
        .route("/details", get(get_folder_details))
        .route("/delete", delete(delete_folder))
        .route_layer(axum_middleware::from_fn(verify_token))

        .with_state(state);


    let user_router = Router::new()
        .route("/get-user", get(get_user))
        .route_layer(axum_middleware::from_fn(verify_token))
        .with_state(user_collection.clone());


    let app = Router::new()
        .route("/", get(root))
        .nest_service("/auth", auth_router)
        .nest("/api", files_router)
        .nest("/user", user_router)
        .nest("/folder", folder_router)
        .nest("/dashboard", dashboard_router)
        .layer(cors)
        .layer(CookieManagerLayer::new());


    let addr = "0.0.0.0:3000".parse().unwrap();
    // let addr = "127.0.0.1:8080".parse().unwrap();
    println!("App is running at {}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
