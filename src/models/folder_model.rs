use bson::Bson;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use crate::models::file_model::File;
use mongodb::{options::ClientOptions, Client};


#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FolderType{
    Folder,
    Subfolder,
}

impl From<FolderType> for Bson {
    fn from(folder_type: FolderType) -> Self {
        match folder_type {
            FolderType::Folder => Bson::String(String::from("Folder")),
            FolderType::Subfolder => Bson::String(String::from("Subfolder")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub folder_name: String,

    pub folder_type: Option<FolderType>,

    pub files: Option<Vec<File>>,

    pub folders: Option<Vec<Folder>>,

    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,

    pub parent_id: Option<ObjectId>,

    pub user_id: Option<ObjectId>
}