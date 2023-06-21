use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::Response;
use crate::context::user_context::UserContext;
use crate::models::file_model::File;
use crate::services::auth_services::UserCollection;
use crate::services::file_services::FileCollection;
use chrono::{TimeZone, Utc};
use futures::FutureExt;
use mongodb::bson::doc;
use futures::stream::{StreamExt, TryStreamExt};
use mongodb::{options::FindOptions};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use crate::controllers::auth_controller::handle_response;
use serde::{Serializer, Deserializer};
use serde_qs::from_str;
use crate::AppState;


#[derive(Deserialize, Debug)]
pub struct MyQueryParams {
    pub file_type: Option<String>,
}

pub async fn get_files(ctx: Result<UserContext, StatusCode>, state: State<Arc<AppState>>, Query(query_params): Query<MyQueryParams>) -> Result<Json<Vec<File>>, StatusCode>{
    match ctx {
        Ok(user_context) => {

            let mut filter = doc! {"user_id": user_context.user_id};

            match query_params.file_type {
                Some(file_type) => {
                    filter = doc! {"user_id": user_context.user_id, "file_type": file_type}
                },
                None => ()
            }

            let files = state.file_collection.get_file(filter).await;

            match files {
                Ok(files) => {
                    return Ok(Json(files));
                },
                Err(_) => {
                    return Err(StatusCode::BAD_REQUEST);
                }
            }


        },
        Err(err) => {
            return Err(err);
        }
    }


}

pub async fn upload_file(ctx: UserContext, state: State<Arc<AppState>>, params: Query<Vec<(String, ObjectId)>>, mut file: Json<File>) -> Result<Json<Vec<File>>, StatusCode>{

    let new_file = state.file_collection.create_file(file, ctx.user_id).await;


    return match new_file {
        Ok(file) => {
            let folder_id = params.0.to_vec().iter().filter(|obj| obj.0 == "folder_id").map(|obj| obj.1).collect::<Vec<_>>();


            if !folder_id.is_empty() {
                let update_file_filter = doc! {"_id": file.inserted_id.as_object_id()};

                let update_file = doc! {
                    "$set":{
                        "folder_id": folder_id.get(0).unwrap()
                        }
                };

                state.file_collection.update_folder(update_file_filter, update_file).await;


                let update_folder_filter = doc! {
                    "_id": folder_id.get(0).unwrap(),
                };

                let update_folder = doc! {
                    "$push": {
                        "files": file.inserted_id.as_object_id(),
                    },
                };

                state.folder_collection.update_folder(update_folder_filter, update_folder).await;
            }



            let filter = doc! {"user_id": ctx.user_id, "folder_id": None::<ObjectId>};
            let mut files = state.file_collection.get_file(filter).await.unwrap_or(vec![]);

            Ok(Json(files))
        },
        Err(_) => {
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteParams{
    ids: ObjectId
}

pub async fn delete_file(ctx: UserContext, state: State<Arc<AppState>>, params: Query<Vec<(String, ObjectId)>>) -> Result<Json<Vec<File>>, StatusCode>{


    let file_ids = params.0.to_vec().iter().filter(|obj| obj.0 == "ids").map(|obj| obj.1).collect::<Vec<_>>();
    let folder_id = params.0.to_vec().iter().filter(|obj| obj.0 == "folder_id").map(|obj| obj.1).collect::<Vec<_>>();

    let update_filter = doc! {"_id": folder_id.get(0), "user_id": ctx.user_id};

    let update = doc! {
        "$pull": {
            "files": {
                "$in": file_ids
            }
        },
    };

    state.folder_collection.update_folder(update_filter, update).await;
    let delete = state.file_collection.delete_files(&params, &ctx.user_id).await;


    match delete {
        Ok(_) => {
            let filter = doc! {"user_id": ctx.user_id};
            let files = state.file_collection.get_file(filter).await.unwrap_or(vec![]);
            return Ok(Json(files));
        }
        Err(_) => {
            Err(StatusCode::BAD_REQUEST)
        }
    }


}

