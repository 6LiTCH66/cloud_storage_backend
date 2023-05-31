use mongodb::Collection;
use crate::models::folder_model::Folder;
use crate::services::trait_service::StorageCollection;
use async_trait::async_trait;
use std::env;
use axum::Json;
use bson::Document;
use bson::oid::ObjectId;
use chrono::Utc;
use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::options::{ClientOptions};

use mongodb::{Client};
use mongodb::results::InsertOneResult;
use crate::services::file_services::FileCollection;

#[derive(Debug, Clone)]
pub struct FolderCollection{
    pub folder_collection: Collection<Folder>,
}

impl FolderCollection {

    pub async fn get_folder(&self, filter: Document) -> Result<Vec<Folder>, mongodb::error::Error>{
        self.folder_collection.find(filter, None).await?.try_collect::<Vec<Folder>>().await
    }

    pub async fn create_folder(&self, mut new_folder: &mut Json<Folder>, user_id: &ObjectId) -> Result<InsertOneResult, mongodb::error::Error>{
        let now = Utc::now();
        new_folder.created_at = Some(now);
        new_folder.updated_at = Some(now);
        new_folder.user_id = Some(user_id.clone());
        self.folder_collection.insert_one(&**new_folder, None).await
    }
}


#[async_trait]
impl StorageCollection for FolderCollection{
    type Error = Box<dyn std::error::Error>;

    async fn init() -> Result<Self, Self::Error> where Self: Sized {
        dotenv().ok();
        let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI not found in env");

        let client_options = ClientOptions::parse(mongo_uri).await?;

        let client = Client::with_options(client_options)?;
        let db = client.database("cloud_storage");
        let folder_col: Collection<Folder> = db.collection("folders");

        Ok(Self{ folder_collection: folder_col })
    }
}