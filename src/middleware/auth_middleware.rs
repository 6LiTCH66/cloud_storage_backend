use std::env;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use dotenv::dotenv;
use jsonwebtoken::{decode, DecodingKey, Validation};
use tower_cookies::Cookies;
use crate::context::user_context::UserContext;
use crate::controllers::auth_controller::handle_response;
use crate::models::token_model::Claims;


pub async fn verify_token<B>(cookies: Cookies, mut req: Request<B>, next: Next<B>,) -> axum::response::Response {
    dotenv().ok();
    let jwt_token = env::var("JWT_TOKEN").expect("JWT_TOKEN not found in env");

    let cookie = cookies.get("accessToken");

    match cookie {
        Some(cookie) => {


            let token = cookie.value();
            let decoding_key = DecodingKey::from_secret(jwt_token.as_ref());
            let validation = Validation::default();
            let token_data = decode::<Claims>(&token, &decoding_key, &validation);


            match token_data {
                Ok(toke) => {
                    let id = toke.claims.id.unwrap();
                    let email = toke.claims.email;

                    let user_context =
                        UserContext::new(id.to_owned(), email.to_string());

                    req.extensions_mut().insert(user_context);


                }
                Err(_) => {
                    return handle_response("Token is not valid!".to_string(), StatusCode::FORBIDDEN).into_response()
                }
            }


        }
        None => {
            req.extensions_mut().remove::<UserContext>();
            return handle_response("A token is required for authentication!".to_string(), StatusCode::UNAUTHORIZED).into_response();
        }
    };


    // let user_ctx = req.extensions().get::<UserContext>().unwrap();
    //
    // println!("{:?}", user_ctx);

    next.run(req).await


}