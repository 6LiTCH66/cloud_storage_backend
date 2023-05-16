use std::collections::HashMap;
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


#[derive(Deserialize, Debug)]
pub struct MyQueryParams {
    pub file_type: Option<String>,
}

pub async fn get_files(ctx: Result<UserContext, StatusCode>, file_col: State<FileCollection>, Query(query_params): Query<MyQueryParams>) -> Result<Json<Vec<File>>, StatusCode>{
    match ctx {
        Ok(user_context) => {

            let mut filter = doc! {"user_id": user_context.user_id};

            match query_params.file_type {
                Some(file_type) => {
                    filter = doc! {"user_id": user_context.user_id, "file_type": file_type}
                },
                None => ()
            }

            let files = file_col.get_files(filter).await;

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

pub async fn upload_file(ctx: Result<UserContext, StatusCode>, file_col: State<FileCollection>, mut file: Json<File>) -> Result<Json<Vec<File>>, StatusCode>{

    match ctx {
        Ok(user_context) => {

            file.user_id = Some(user_context.user_id);

            file.original_file_name = Some(file.file_name.to_string());

            let filter = doc! {"original_file_name": &file.file_name};

            let mut count_duplicates = file_col.get_files(filter).await.unwrap_or(vec![]);

            if count_duplicates.len() > 0 {
                let mut split_file_name = file.file_name.split(".").collect::<Vec<_>>();
                let file_name = split_file_name[0];
                let file_duplicate = count_duplicates.len();
                let file_format = split_file_name.pop().unwrap();

                file.file_name = format!("{} ({}).{}", file_name, file_duplicate, file_format)
            }
            // Ok(Json(count_duplicates))

            let new_file = file_col.create_file(file).await;

            match new_file {

                Ok(_) => {
                    let filter = doc! {"user_id": user_context.user_id};
                    let mut files = file_col.get_files(filter).await;

                    match files {
                        Ok(files) => {
                            return Ok(Json(files));
                        },
                        Err(_) => {
                            return Err(StatusCode::BAD_REQUEST);
                        }
                    }
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

#[derive(Debug, Deserialize)]
pub struct DeleteParams{
    ids: ObjectId
}

pub async fn delete_file(ctx: Result<UserContext, StatusCode>, file_col: State<FileCollection>, params: Query<Vec<(String, ObjectId)>>) -> Result<Json<Vec<File>>, StatusCode>{

    match ctx {
        Ok(user_context) => {
            let ids = params.0.to_vec().iter().map(|obj| doc! {"_id": obj.1, "user_id": user_context.user_id}).collect::<Vec<_>>();

            let delete = file_col.file_collection.delete_many(doc! {"$or": ids, "user_id": user_context.user_id}, None).await;

            match delete {
                Ok(_) => {
                    let filter = doc! {"user_id": user_context.user_id};
                    let files = file_col.get_files(filter).await.unwrap_or(vec![]);
                    return Ok(Json(files));
                }
                Err(_) => {
                    Err(StatusCode::BAD_REQUEST)
                }
            }
        },
        Err(err) => {
            Err(err)
        }
    }

}

