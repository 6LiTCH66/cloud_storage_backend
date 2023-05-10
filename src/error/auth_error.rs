use axum::http::StatusCode;
use axum::response::Response;

pub enum AuthError{
    LoginError(String, StatusCode),
    SignupError,
}


impl AuthError {

    pub fn create_error(&self) -> Response<String>{
        match self {
            Self::LoginError(error_message, status) => {

                let response_body = serde_json::json!({ "message": error_message }).to_string();

                Response::builder()
                    .status(status)
                    .header("content-type", "application/json")
                    .body(response_body)
                    .unwrap()
            },
            _ => {
                let response_body = serde_json::json!({ "message": "No impl" }).to_string();

                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("content-type", "application/json")
                    .body(response_body)
                    .unwrap()
            }
        }
    }
}