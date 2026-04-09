use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use axum_extra::extract::cookie::CookieJar;
use crate::auth::jwt::{validate_jwt, AUTH_COOKIE_NAME};

pub async fn auth_middleware(
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = jar.get(AUTH_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    match validate_jwt(&token) {
        Ok(claims) => {
            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
