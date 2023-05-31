use std::env;
use mongodb::{Client, Collection, Cursor};
use crate::models::file_model::File;
use async_trait::async_trait;
use axum::Json;
use bson::oid::ObjectId;
use chrono::Utc;
use dotenv::dotenv;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::{ClientOptions, FindOptions};
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



    pub async fn create_file(&self, mut new_file: Json<File>, user_id: ObjectId) -> Result<InsertOneResult, Error>{

        let now = Utc::now();
        new_file.user_id = Some(user_id.clone());
        new_file.original_file_name = Some(new_file.file_name.to_string());
        new_file.created_at = Some(now);
        new_file.updated_at = Some(now);


        let filter = doc! {"original_file_name": &new_file.file_name, "user_id": user_id.clone()};

        let mut count_duplicates = self.get_files(filter).await.unwrap_or(vec![]);

        if count_duplicates.len() > 0 {
            let mut split_file_name = new_file.file_name.split(".").collect::<Vec<_>>();
            let file_name = split_file_name[0];
            let file_duplicate = count_duplicates.len();
            let file_format = split_file_name.pop().unwrap();

            new_file.file_name = format!("{} ({}).{}", file_name, file_duplicate, file_format)
        }


        return self.file_collection.insert_one(&*new_file, None).await;
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
