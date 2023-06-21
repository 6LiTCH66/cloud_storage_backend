use std::fs::FileType;
use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use bson::doc;
use mongodb::bson::oid::ObjectId;
use chrono::Utc;
use crate::AppState;
use crate::context::user_context::UserContext;
use crate::models::file_model::File;
use crate::models::folder_model::{Folder, FolderJSON, FolderType};
use crate::services::file_services::FileCollection;
use crate::services::folder_service::FolderCollection;
use futures::future::BoxFuture;
use futures::{FutureExt};



async fn save_files_to_db(state: &State<Arc<AppState>>, files: &mut [File], user_id: &ObjectId, folder_id: Option<ObjectId>) -> mongodb::error::Result<Vec<ObjectId>>{
    let mut files_array: Vec<ObjectId> = vec![];

    for file in files{
        file.folder_id = folder_id;
        let inserted_file = state.file_collection.create_file(Json(file.clone()), user_id.clone()).await.unwrap();

        files_array.push(inserted_file.inserted_id.as_object_id().unwrap());

    }

    Ok(files_array)
}



fn save_folders_to_db<'a>(state: &'a State<Arc<AppState>>, mut folder: &'a mut FolderJSON, user_id: &'a ObjectId) -> BoxFuture<'a, mongodb::error::Result<ObjectId>> {
    async move{

        let mut new_folder = Folder::new();

        let folder_id = Some(ObjectId::new());

        folder.id = folder_id;

        new_folder.id = folder_id.clone();

        new_folder.folder_name = folder.folder_name.clone();
        new_folder.folder_type = folder.folder_type.clone();
        new_folder.user_id = Some(user_id.clone());
        new_folder.parent_id = folder.parent_id;

        if let Some(files) = &mut folder.files{

            let mut files_array = save_files_to_db(&state, files, &user_id, folder_id).await.unwrap_or(vec![]);

            new_folder.files = Some(files_array);

        }else{
            new_folder.files = Some(Vec::new());
        }


        if let Some(subfolders) = &mut folder.folders{

            if subfolders.is_empty() {
                new_folder.folders = Some(Vec::new());
            }

            let mut folders_array: Vec<ObjectId> = vec![];

            for subfolder in subfolders {

                subfolder.parent_id = folder.id.clone();
                subfolder.folder_type = Some(FolderType::Subfolder);


                save_folders_to_db(&state, subfolder, &user_id).await.unwrap();
                folders_array.push(subfolder.id.unwrap());
                new_folder.folders = Some(folders_array.clone());

            }

        }else{
            new_folder.folders = Some(Vec::new());
        }

        let inserted_id = state.folder_collection.create_folder(&mut Json(new_folder.clone()), &user_id).await.unwrap();


        Ok(inserted_id.inserted_id.as_object_id().unwrap())
    }.boxed()

}

pub async fn create_folder(ctx: UserContext, state: State<Arc<AppState>>, params: Query<Vec<(String, ObjectId)>>, mut folder: Json<FolderJSON>) -> Result<Json<Vec<Folder>>, StatusCode>{
    let folder_id = params.0.to_vec().iter().filter(|obj| obj.0 == "folder_id").map(|obj| obj.1).collect::<Vec<_>>();

    if folder_id.is_empty() {

        folder.folder_type = Some(FolderType::Folder);

        save_folders_to_db(&state, &mut folder, &ctx.user_id).await;


    }else{

        folder.folder_type = Some(FolderType::Subfolder);
        folder.parent_id = Some(folder_id.get(0).unwrap().clone());

        let id = save_folders_to_db(&state, &mut folder, &ctx.user_id).await.unwrap();

        let update_filter = doc! {"_id": folder_id.get(0).unwrap(), "user_id": ctx.user_id};

        let update = doc! {
                    "$push": {
                        "folders": id
                    },
            };

        state.folder_collection.update_folder(update_filter, update).await;

    }


    let filter = doc! {"user_id": ctx.user_id, "folder_type": FolderType::Folder};
    let folder_to_display = state.folder_collection.get_folder(filter).await.unwrap_or(vec![]);

    Ok(Json(folder_to_display))


}

pub async fn get_folders(ctx: UserContext, state: State<Arc<AppState>>) -> Result<Json<Vec<Folder>>, StatusCode>{
    let filter = doc! {"user_id": ctx.user_id, "folder_type": FolderType::Folder};

    let folder_to_display = state.folder_collection.get_folder(filter).await.unwrap_or(vec![]);
    Ok(Json(folder_to_display))
}


fn delete_folder_from_db<'a>(state: &'a State<Arc<AppState>>, folders: &'a [Folder], user_id: &'a ObjectId) -> BoxFuture<'a, mongodb::error::Result<()>>{

    async move{

        for folder in folders {

            if let Some(subfolders) = &folder.folders{
                for subfolder in subfolders {
                    let filter = doc! {"_id": subfolder, "user_id": user_id};

                    let mut folders = state.folder_collection.get_folder(filter).await.unwrap();

                    delete_folder_from_db(&state, &folders, &user_id).await;
                }
            }

            if let Some(files) = &folder.files{
                for file in files {
                    let file_filter = doc! {"_id": file, "user_id": user_id};
                    state.file_collection.delete_one_file(file_filter).await;
                }
            }

            let update_filter = doc! {"_id": folder.parent_id, "user_id": user_id};

            let update = doc! {
                    "$pull": {
                        "folders": folder.id
                    },
            };

            let filter = doc! {"_id": folder.id, "user_id": user_id};

            state.folder_collection.update_folder(update_filter, update).await;
            state.folder_collection.delete_one_folder(filter).await;


        }

        Ok(())
    }.boxed()
}


pub async fn delete_folder(ctx: UserContext, state: State<Arc<AppState>>, params: Query<Vec<(String, ObjectId)>>) -> Result<Json<Vec<Folder>>, StatusCode>{

    let folders = state.folder_collection.get_folder_by_id(&params, &ctx.user_id).await;


    let is_folders_deleted = delete_folder_from_db(&state, &folders.clone().unwrap(), &ctx.user_id).await;

    match is_folders_deleted {
        Ok(_) => {
            let filter = doc! {"user_id": ctx.user_id, "folder_type": FolderType::Folder};
            let folders = state.folder_collection.get_folder(filter).await.unwrap_or(vec![]);
            return Ok(Json(folders));
        }
        Err(_) => {
            Err(StatusCode::BAD_REQUEST)
        }
    }
}