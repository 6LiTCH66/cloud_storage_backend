use std::env;
use mongodb::{Client, Collection, Cursor};
use crate::models::file_model::File;
use async_trait::async_trait;
use axum::Json;
use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::bson::Document;
use mongodb::options::ClientOptions;
use crate::services::auth_services::UserCollection;
use crate::services::trait_service::StorageCollection;
use mongodb::error::{Error, Result as MongoResult};
use mongodb::results::InsertOneResult;

#[derive(Debug, Clone)]
pub struct FileCollection{
    pub file_collection: Collection<File>
}


impl FileCollection {

    pub async fn get_files(&self, filter: Document) -> Result<Vec<File>, Error> {
        let files = self.file_collection.find(filter, None).await?.try_collect::<Vec<File>>().await;
        files

    }


    pub async fn create_file(&self, new_file: Json<File>) -> Result<InsertOneResult, Error>{
        let insert_file = self.file_collection.insert_one(&*new_file, None).await;
        insert_file
    }


}


#[async_trait]
impl StorageCollection for FileCollection{
    type Error = Box<dyn std::error::Error>;

    async fn init() -> Result<Self, Self::Error> where Self: Sized {
        dotenv().ok();
        let mongo_uri = env::var("MONGODB_URI").expect("MONGODB_URI not found in env");

        let client_options = ClientOptions::parse(mongo_uri).await?;

        let client = Client::with_options(client_options)?;
        let db = client.database("cloud_storage");
        let col: Collection<File> = db.collection("files");

        Ok(Self{ file_collection: col })
    }

}
