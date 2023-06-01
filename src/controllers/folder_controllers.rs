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
use crate::models::folder_model::{Folder, FolderType};
use crate::services::file_services::FileCollection;
use crate::services::folder_service::FolderCollection;
use futures::future::BoxFuture;
use futures::{FutureExt};

async fn save_files_to_db(state: &State<Arc<AppState>>, files: &mut [File], user_id: &ObjectId) -> mongodb::error::Result<Vec<File>>{
    let mut files_array: Vec<File> = vec![];

    for file in files{
        let inserted_file = state.file_collection.create_file(Json(file.clone()), user_id.clone()).await.unwrap();

        let filter = doc! { "_id": inserted_file.inserted_id };
        let result = state.file_collection.get_files(filter).await.unwrap_or(vec![]);
        files_array.extend(result);

    }

    Ok(files_array)
}



fn save_folders_to_db<'a>(state: &'a State<Arc<AppState>>, mut folder: &'a mut Folder, user_id: &'a ObjectId) -> BoxFuture<'a, mongodb::error::Result<()>> {
    async move{

        let folder_id = Some(ObjectId::new());
        folder.id = folder_id;

        if let Some(files) = &mut folder.files{
            let mut files_array = save_files_to_db(&state, files, &user_id).await.unwrap_or(vec![]);


            folder.files = None;
            folder.files = Some(files_array);
        }

        if let Some(subfolders) = &mut folder.folders{
            let now = Utc::now();
            for subfolder in subfolders {

                subfolder.parent_id = folder.id.clone();

                subfolder.folder_type = Some(FolderType::Subfolder);
                subfolder.created_at = Some(now);
                subfolder.updated_at = Some(now);
                subfolder.user_id = Some(user_id.clone());


                save_folders_to_db(&state, subfolder, &user_id).await.unwrap();

                state.folder_collection.create_folder(&mut Json(subfolder.clone()), &user_id).await.unwrap();

            }
        }

        Ok(())
    }.boxed()

}

pub async fn create_folder(ctx: UserContext, state: State<Arc<AppState>>, mut folder: Json<Folder>) -> Result<Json<Vec<Folder>>, StatusCode>{


    folder.folder_type = Some(FolderType::Folder);
    save_folders_to_db(&state, &mut folder, &ctx.user_id).await;

    let filter = doc! {"user_id": ctx.user_id, "folder_type": FolderType::Folder};

    state.folder_collection.create_folder(&mut folder, &ctx.user_id).await.unwrap();
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
                delete_folder_from_db(&state, subfolders, &user_id).await;
            }

            if let Some(files) = &folder.files{
                for file in files {
                    let file_filter = doc! {"_id": file.id};
                    state.file_collection.delete_one_file(file_filter).await;
                }
            }
            let update_filter = doc! {"_id": folder.parent_id};

            let update = doc! {
                    "$pull": {
                        "folders": {
                            "_id": folder.id,
                        },
                    },
                };

            let filter = doc! {"_id": folder.id};
            state.folder_collection.update_folder(update_filter, update).await;
            state.folder_collection.delete_one_folder(filter).await;


        }

        Ok(())
    }.boxed()
}


pub async fn delete_folder(ctx: UserContext, state: State<Arc<AppState>>, params: Query<Vec<(String, ObjectId)>>) -> Result<Json<Vec<Folder>>, StatusCode>{

    let folders = state.folder_collection.get_folder_by_id(&params, &ctx.user_id).await; // getting single folder with nested folders

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