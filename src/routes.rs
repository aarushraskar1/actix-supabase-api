use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{CreateUser, UpdateUser, User},
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health)
        .service(create_user)
        .service(list_users)
        .service(get_user)
        .service(update_user)
        .service(delete_user);
}

#[get("/health")]
async fn health() -> impl Responder {
    web::Json(HealthResponse { status: "ok" })
}

#[post("/api/v1/users")]
async fn create_user(
    state: web::Data<AppState>,
    payload: web::Json<CreateUser>,
) -> Result<impl Responder, AppError> {
    validate_user_fields(&payload.name, &payload.email)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (name, email)
        VALUES ($1, $2)
        RETURNING id, name, email, created_at, updated_at
        "#,
    )
    .bind(payload.name.trim())
    .bind(payload.email.trim())
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::Database)?;

    Ok(web::Json(user))
}

#[get("/api/v1/users")]
async fn list_users(state: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT id, name, email, created_at, updated_at
        FROM users
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::Database)?;

    Ok(web::Json(users))
}

#[get("/api/v1/users/{id}")]
async fn get_user(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(user_id.into_inner())
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Database)?
    .ok_or(AppError::NotFound)?;

    Ok(web::Json(user))
}

#[put("/api/v1/users/{id}")]
async fn update_user(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
    payload: web::Json<UpdateUser>,
) -> Result<impl Responder, AppError> {
    if payload.name.is_none() && payload.email.is_none() {
        return Err(AppError::BadRequest(
            "at least one of name or email is required".to_owned(),
        ));
    }

    if let Some(name) = &payload.name {
        if name.trim().is_empty() {
            return Err(AppError::BadRequest("name must not be empty".to_owned()));
        }
    }
    if let Some(email) = &payload.email {
        if email.trim().is_empty() || !email.contains('@') {
            return Err(AppError::BadRequest("email must be valid".to_owned()));
        }
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET name = COALESCE($1, name),
            email = COALESCE($2, email),
            updated_at = NOW()
        WHERE id = $3
        RETURNING id, name, email, created_at, updated_at
        "#,
    )
    .bind(payload.name.as_deref().map(str::trim))
    .bind(payload.email.as_deref().map(str::trim))
    .bind(user_id.into_inner())
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::Database)?
    .ok_or(AppError::NotFound)?;

    Ok(web::Json(user))
}

#[delete("/api/v1/users/{id}")]
async fn delete_user(
    state: web::Data<AppState>,
    user_id: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id.into_inner())
        .execute(&state.pool)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    info!("user deleted");
    Ok(HttpResponse::NoContent().finish())
}

fn validate_user_fields(name: &str, email: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::BadRequest("name must not be empty".to_owned()));
    }
    if email.trim().is_empty() || !email.contains('@') {
        return Err(AppError::BadRequest("email must be valid".to_owned()));
    }
    Ok(())
}
