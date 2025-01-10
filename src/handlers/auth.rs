use axum::{
    extract::{Extension, Request},
    http::StatusCode,
    middleware::{Next},
    response::Response,
};

use crate::structures::user::CurrentUser;

const AUTH_HEADER_NAME: &str = "x-oz-user-id";
const AUTH_HEADER_DEV_ID: &str = "x-oz-dev-id";

pub async fn auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTH_HEADER_NAME)
        .or_else(|| req.headers().get(AUTH_HEADER_DEV_ID))
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(current_user) = authorize_current_user(auth_header).await {
        // insert the current user into a request extension so the handler can
        // extract it
        req.extensions_mut().insert(current_user);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn authorize_current_user(auth_token: &str) -> Option<CurrentUser> {
    if auth_token.is_empty() {
        return None;
    }
    Some(CurrentUser {
        user_id: auth_token.to_string(),
    })
}

async fn test_auth_handler(
    // extract the current user, set by the middleware
    Extension(current_user): Extension<CurrentUser>,
) {
    println!("current_user: {}", current_user.user_id);
}
