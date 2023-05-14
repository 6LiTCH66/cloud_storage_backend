use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct File{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_file_name: Option<String>,
    pub file_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
    pub aws_file_name: String,
    pub file_location: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<ObjectId>
}