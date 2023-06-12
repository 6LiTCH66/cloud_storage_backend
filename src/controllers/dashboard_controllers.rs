use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use bson::doc;
use bson::oid::ObjectId;
use crate::{AppState, Item};
use crate::context::user_context::UserContext;
use crate::models::folder_model::{Folder, FolderType};
use axum::{response::IntoResponse};
use crate::models::file_model::File;


pub async fn get_dashboard(ctx: UserContext, state: State<Arc<AppState>>, params: Query<Vec<(String, ObjectId)>>) -> Result<Json<Vec<Item<File, Folder>>>, StatusCode>{

    let dashboard_result = state.get_dashboard_controller(&ctx.user_id, &params).await;

    return match dashboard_result {
        Ok(dashboard) => {
            Ok(Json(dashboard))
        },
        Err(_) => Err(StatusCode::BAD_REQUEST)
    };

}