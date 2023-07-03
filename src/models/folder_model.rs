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

// Folder struct to be save in DB
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub folder_name: String,

    pub folder_type: Option<FolderType>,

    pub files: Option<Vec<ObjectId>>,

    pub folders: Option<Vec<ObjectId>>,

    #[serde(rename = "createdAt")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "updatedAt")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,

    pub parent_id: Option<ObjectId>,

    pub user_id: Option<ObjectId>,

    pub path: Option<String>
}




#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderJSON{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,

    pub folder_name: String,

    pub folder_type: Option<FolderType>,

    pub files: Option<Vec<File>>,

    pub folders: Option<Vec<FolderJSON>>,

    pub parent_id: Option<ObjectId>,

    pub path: Option<String>


}

impl Folder {
    pub fn new() -> Self {
        Self {
            id: None,
            folder_name: "".to_string(),
            folder_type: None,
            files: None,
            folders: None,
            created_at: None,
            updated_at: None,
            parent_id: None,
            user_id: None,
            path: None
        }
    }

}

