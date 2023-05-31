use std::{env};
use std::io::Read;
use std::str::FromStr;
use async_trait::async_trait;
use axum::body::Body;
use axum::Json;
use axum::extract::{FromRequestParts, State};
use mongodb::{Client, Collection, bson::doc};
use crate::models::user_model::{User};
use crate::services::auth_services::UserCollection;
use axum::http::{Request, Response, StatusCode};
use axum::http::header::COOKIE;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::IntoResponse;
use mongodb::results::InsertOneResult;
use bcrypt::{DEFAULT_COST, hash, hash_with_salt, verify};
use chrono::Duration;
use dotenv::dotenv;
use jsonwebtoken::{decode, DecodingKey, encode, EncodingKey, Header, Validation};
use mongodb::bson::oid::ObjectId;
use tower_cookies::{Cookies, Cookie};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use tower_cookies::cookie::{CookieBuilder, SameSite};
use chrono::prelude::*;
use serde::de::StdError;
use crate::context::user_context::UserContext;
use crate::models::token_model::Claims;


pub fn handle_response(message: String, status: StatusCode) -> Response<String>{
    let response_body = json!({ "message": message }).to_string();

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(response_body)
        .unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse{
    id: Option<ObjectId>,
    email: String,
}

pub async fn get_user(ctx: Result<UserContext, StatusCode>, user_col: State<UserCollection>) -> Result<Json<UserResponse>, StatusCode>{
    match ctx {
        Ok(context) => {
            let filter = doc! {"_id": context.user_id};
            let user = user_col.user_collection.find_one(filter, None).await.unwrap_or(Option::None);

            match user {
                Some(user) => {

                    let user_response = UserResponse{
                        id: user.id,
                        email: user.email,
                    };

                    Ok(Json(user_response))
                },
                None => {
                    Err(StatusCode::NOT_FOUND)
                }
            }
        },
        Err(err) => {
            Err(err)
        }
    }
}


pub async fn sign_up(user_col: State<UserCollection>, user: Json<User>) -> Response<String>{

    let user_filter = doc!{"email": &user.email};

    let candidate = user_col.user_collection.find_one(user_filter, None).await.unwrap_or(Option::None);

    match candidate {
        Some(_) => {
            let error_message = "User with this email is already exists.".to_string();
            handle_response(error_message, StatusCode::BAD_REQUEST)

        },
        None => {
            let hashed_password = hash(&user.password, DEFAULT_COST).unwrap();
            let user_details = User {
                id: None,
                email: user.email.to_string(),
                password: hashed_password,
            };

            let new_user = user_col.user_collection.insert_one(&user_details, None).await;

            match new_user {
                Ok(_) =>{
                    let success_message = "User has been created.".to_string();
                    handle_response(success_message, StatusCode::OK)

                },

                Err(_) => {

                    let error_message = "Something went wrong while signing up!".to_string();
                    handle_response(error_message, StatusCode::BAD_REQUEST)

                }
            }


        }
    }
}


pub async fn sing_in(cookies: Cookies, user_col: State<UserCollection>, user: Json<User>) -> Result<Json<Claims>, Response<String>>{
    dotenv().ok();
    let jwt_token = env::var("JWT_TOKEN").expect("JWT_TOKEN not found in env");
    let email = &user.email;
    let user_password = &user.password;

    let user_filter = doc!{"email": email};
    let candidate = user_col.user_collection.find_one(user_filter, None).await.unwrap_or(Option::None);

    match candidate {

        Some(mut user) => {
            let validate_password = verify(user_password, &user.password).unwrap_or(false);

            if validate_password {
                let claims = Claims{
                    id: user.id.clone(),
                    email: user.email.clone(),
                    exp: (Utc::now() + Duration::days(1)).timestamp()
                };


                let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_token.as_ref())).unwrap();

                // secure true in production
                let cookie = Cookie::build("accessToken", token)
                    .path("/")
                    .max_age(time::Duration::days(1))
                    .http_only(true)
                    .same_site(SameSite::None)
                    .secure(false)
                    .finish();

                cookies.add(cookie);

                return Ok(Json(claims));

            }else{
                let error_message = "Password is invalid!".to_string();
                return Err(handle_response(error_message, StatusCode::BAD_REQUEST));
            }


        }

        None => {
            let error_message = format!("User with email {email} is not found!");
            return Err(handle_response(error_message, StatusCode::NOT_FOUND));
        }
    }

}



pub async fn logout(cookies: Cookies) -> Response<String>{

    let cookie = Cookie::named("accessToken");

    let token = cookie.value().to_string();
    let delete_cookie = Cookie::build("accessToken", token)
        .path("/")
        .http_only(true)
        .same_site(SameSite::None)
        .secure(true)
        .finish();


    cookies.remove(delete_cookie);


    handle_response("User has been logged out.".to_string(), StatusCode::OK)



}


#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for UserContext{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserContext>()
            .ok_or(StatusCode::BAD_REQUEST)
            .cloned()
    }
}
