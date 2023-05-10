use std::env;
use axum::Json;
use dotenv::dotenv;
use mongodb::{Client, Collection};
use mongodb::options::ClientOptions;
use mongodb::results::InsertOneResult;
use crate::models::user_model::{User};
use crate::services::trait_service::StorageCollection;
use async_trait::async_trait;


#[derive(Debug, Clone)]
pub struct UserCollection{
    pub user_collection: Collection<User>
}


#[async_trait]
impl StorageCollection for UserCollection{
    type Error = Box<dyn std::error::Error>;

    async fn init() -> Result<Self, Self::Error> where Self: Sized {
        dotenv().ok();
        let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI not found in env");


        let client_options = ClientOptions::parse(mongo_uri).await?;

        let client = Client::with_options(client_options)?;
        let db = client.database("cloud_storage");
        let col: Collection<User> = db.collection("users");

        Ok(UserCollection{user_collection: col})
    }
}

// type Error = Box<dyn std::error::Error>;
//
// impl UserCollection {
//
//     pub async fn init() -> Result<Self, Error> {
//         dotenv().ok();
//         let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI not found in env");
//
//
//         let client_options = ClientOptions::parse(mongo_uri).await?;
//
//         let client = Client::with_options(client_options)?;
//         let db = client.database("cloud_storage");
//         let col: Collection<User> = db.collection("users");
//
//         Ok(UserCollection{user_collection: col})
//     }
//
//
// }