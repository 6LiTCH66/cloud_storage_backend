use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use bson::doc;
use bson::oid::ObjectId;
use crate::{AppState, Item};
use crate::context::user_context::UserContext;
use crate::models::folder_model::{Folder, FolderType};
use axum::{response::IntoResponse};


pub async fn get_dashboard(ctx: UserContext, state: State<Arc<AppState>>) -> Result<Json<Vec<Item>>, StatusCode>{

    let dashboard_result = state.get_dashboard_controller(&ctx.user_id).await;
    return match dashboard_result {
        Ok(dashboard) => {
            Ok(Json(dashboard))
        },
        Err(_) => Err(StatusCode::BAD_REQUEST)
    };

}