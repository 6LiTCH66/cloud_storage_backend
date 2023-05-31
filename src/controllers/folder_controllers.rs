use std::sync::Arc;
use axum::extract::State;
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

        // file.id = inserted_file.inserted_id.as_object_id();

        let filter = doc! { "_id": inserted_file.inserted_id };
        let result = state.file_collection.get_files(filter).await.unwrap_or(vec![]);
        files_array.extend(result);

    }

    Ok(files_array)
}



fn save_folders_to_db<'a>(state: &'a State<Arc<AppState>>, mut folder: &'a mut Folder, user_id: &'a ObjectId) -> BoxFuture<'a, mongodb::error::Result<()>> {
    async move{
        if let Some(files) = &mut folder.files{
            let mut files_array = save_files_to_db(&state, files, &user_id).await.unwrap_or(vec![]);
            // save_files_to_db(&state, files, &user_id).await.unwrap_or(vec![]);
            //
            //
            folder.files = None;
            folder.files = Some(files_array);
        }

        if let Some(subfolders) = &mut folder.folders{
            let now = Utc::now();
            for subfolder in subfolders {

                subfolder.parent_id = folder.id;
                subfolder.folder_type = Some(FolderType::Subfolder);
                subfolder.created_at = Some(now);
                subfolder.updated_at = Some(now);
                subfolder.user_id = Some(user_id.clone());

                save_folders_to_db(&state, subfolder, &user_id).await.unwrap();

                let new_subfolder = state.folder_collection.create_folder(&mut Json(subfolder.clone()), &user_id).await.unwrap();
                let new_subfolder_id = new_subfolder.inserted_id.as_object_id();
                subfolder.id = new_subfolder_id;
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