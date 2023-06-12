use std::error::Error;
use std::fmt::Debug;
use axum::extract::Query;
use axum::Json;
use bson::doc;
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{AppState, Item};
// use crate::Item::Folder;
use crate::models::file_model::File;
use crate::models::folder_model::{Folder, FolderType};


fn sort_items(items: &mut Vec<Item<File, Folder>>) {
    items.sort_by(|a, b| match (a, b) {
        (Item::File(file1), Item::File(file2)) => file1.created_at.cmp(&file2.created_at),
        (Item::Folder(folder1), Item::Folder(folder2)) => folder1.created_at.cmp(&folder2.created_at),
        (Item::File(file), Item::Folder(folder)) => file.created_at.cmp(&folder.created_at),
        (Item::Folder(folder), Item::File(file)) => folder.created_at.cmp(&file.created_at),
    });
}

impl AppState {
    pub async fn get_dashboard_controller(&self, user_id: &ObjectId, params: &Query<Vec<(String, ObjectId)>>) -> Result<Vec<Item<File, Folder>>, Box<dyn Error>>{

        let details_folder = self.folder_collection.get_folder_by_id(&params, &user_id).await; // if there is an id of subfolder

        let filters = match details_folder {
            Ok(subfolders) => {

                let files = if let Some(folders) = subfolders[0].clone().files {
                    folders
                } else {
                    Vec::new()
                };

                let folders = if let Some(folders) = subfolders[0].clone().folders {
                    folders
                } else {
                    Vec::new()
                };

                let file_filter = doc! {
                    "_id": {
                        "$in": files
                    }
                };


                let folder_filter = doc! {
                    "_id": {
                        "$in": folders
                    }
                };

                (file_filter, folder_filter)

            },
            Err(_) => {
                let file_filter = doc! {"user_id": user_id, "folder_id": None::<ObjectId>};
                let folder_filter = doc! {"user_id": user_id, "folder_type": FolderType::Folder};
                (file_filter, folder_filter)
            }
        };


        let files = match self.file_collection.get_file(filters.0).await{
            Ok(files) => files,
            Err(e) => return Err(e.into())
        };

        let folders = match self.folder_collection.get_folder(filters.1).await{
            Ok(folders) => folders,
            Err(e) => return Err(e.into())
        };

        let mut items: Vec<Item<File, Folder>> = Vec::new();

        for file in files.iter() {
            items.push(Item::File(file.clone()));
        }

        for folder in folders.iter() {
            items.push(Item::Folder(folder.clone()));
        }
        sort_items(&mut items);

        Ok(items)

    }
}