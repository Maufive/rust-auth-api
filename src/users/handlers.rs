use std::sync::{ Arc, Mutex };

use argon2::{ password_hash::SaltString, Argon2, PasswordHasher };
use axum::{ extract::{ Path, State }, http::StatusCode, response::IntoResponse, Json };
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use uuid::Uuid;
use rand_core::OsRng;

use crate::{ authentication::{ new_session, SessionToken }, users::UserResponse, AppState };

use super::{ CreateUserSchema, UpdateUserSchema, User };

type Database = sqlx::PgPool;
type Random = Arc<Mutex<ChaCha8Rng>>;

pub async fn health_check_handler() -> impl IntoResponse {
    const MESSAGE: &str = "API Services ";

    let json_response = serde_json::json!({
        "status": "ok",
        "message": MESSAGE
    });

    return Json(json_response);
}

fn create_password_hash(password: &str) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response =
                serde_json::json!({
                "status": "error",
                "message": format!("Error while hashing password: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
        .map(|hash| hash.to_string())?;

    Ok(hashed_password)
}

async fn get_user(
    id: Uuid,
    data: &Arc<AppState>
) -> Result<User, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx
        ::query_as::<_, User>(r#"SELECT * FROM users WHERE id = $1"#)
        .bind(id)
        .fetch_one(&data.db).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    Ok(user)
}

pub async fn signup(
    database: &Database,
    random: &Random,
    body: CreateUserSchema
) -> Result<SessionToken, (StatusCode, Json<serde_json::Value>)> {
    let id = uuid::Uuid::new_v4();

    println!("Creating user properties: {:?}", &body);

    let hashed_password = create_password_hash(&body.password)?;

    let query_result = sqlx
        ::query(
            r#"INSERT INTO users (id, first_name, last_name, email, password) VALUES ($1, $2, $3, $4, $5)"#
        )
        .bind(&id)
        .bind(&body.first_name)
        .bind(&body.last_name)
        .bind(&body.email)
        .bind(hashed_password)
        .execute(database).await
        .map_err(|err: sqlx::Error| err.to_string());

    // Duplicate error check
    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            let error_response =
                serde_json::json!({
                "status": "error",
                "message": "User already exists"
            });

            return Err((StatusCode::CONFLICT, Json(error_response)));
        }

        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", err)})),
        ));
    }

    return Ok(new_session(database, random, id).await);
}

pub async fn get_user_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = get_user(id, &data).await?;
    let user_response = map_user_to_response(&user);
    let json =
        json!({
        "status": "success",
        "data": {
            "user": user_response
        }
    });

    return Ok(Json(json));
}

pub async fn update_user_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateUserSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let update_result = sqlx
        ::query(
            r#"UPDATE users SET first_name = COALESCE($1, first_name), last_name = COALESCE($2, last_name), email = COALESCE($3, email) WHERE id = $4"#
        )
        .bind(&body.first_name)
        .bind(&body.last_name)
        .bind(&body.email)
        .bind(&id)
        .execute(&data.db).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    // If no data affected (or deleted when wanted to update)
    if update_result.rows_affected() == 0 {
        let error_response =
            serde_json::json!({
                "status": "error",
                "message": format!("User with ID: {} not found", id)
            });

        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    // Get updated data
    let updated_user = get_user(id, &data).await?;

    let user_response =
        serde_json::json!({
            "status": "success",
            "data": serde_json::json!({
                "user": map_user_to_response(&updated_user)
            })
        });

    return Ok(Json(user_response));
}

pub async fn delete_user_handler(
    Path(id): Path<Uuid>,
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx
        ::query(r#"DELETE FROM users WHERE id = $1 RETURNING *"#)
        .bind(id)
        .execute(&data.db).await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                let error_response =
                    serde_json::json!({
                        "status": "error",
                        "message": format!("User with ID: {} not found", id)
                    });

                return Err((StatusCode::NOT_FOUND, Json(error_response)));
            }

            let user_response =
                serde_json::json!({
                    "status": "success",
                    "message": "User deleted successfully"
                });

            Ok(Json(user_response))
        }
        Err(e) => {
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ))
        }
    }
}

pub async fn get_all_users_handler(State(data): State<Arc<AppState>>) -> Result<
    impl IntoResponse,
    (StatusCode, Json<serde_json::Value>)
> {
    let users = sqlx
        ::query_as::<_, User>(r#"SELECT * FROM users"#)
        .fetch_all(&data.db).await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    let users_response =
        serde_json::json!({
            "status": "success",
            "data": serde_json::json!({
                "users": users.iter().map(map_user_to_response).collect::<Vec<UserResponse>>()
            })
        });

    Ok(Json(users_response))
}

// Convert DB model to JSON response
fn map_user_to_response(user: &User) -> UserResponse {
    UserResponse {
        id: user.id.to_owned(),
        first_name: user.first_name.to_owned(),
        last_name: user.last_name.to_owned(),
        email: user.email.to_owned(),
        role: user.role.to_owned(),
        created_at: user.created_at.to_owned(),
        updated_at: user.updated_at.to_owned(),
    }
}
