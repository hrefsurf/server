
use argon2::password_hash::{rand_core::OsRng, SaltString, PasswordHasher};
use axum::{extract::State, response::{Html, IntoResponse, Redirect, Response}, Form};
use chrono::Utc;
use serde::Deserialize;
use sqlx::MySqlPool;
use tera::Context;

use crate::{errors, models::{self, allocated_user::AllocatedUser, authenticated_user_record::AuthenticatedUserRecord, user::User}, state::AppState};

struct HashedPasswordRecord {
    hash: String,
    salt: String
}

fn hash_incoming_password(
    password: String
) -> anyhow::Result<HashedPasswordRecord> {
    // TODO: Check parameters with rich errors!
    // TODO: Make testable!

    // MARK: Password hashing
    let salt = SaltString::generate(&mut OsRng);
    let argon = argon2::Argon2::default();

    let hash_result = argon.hash_password(password.as_bytes(), &salt);
    if hash_result.is_err() {
        return Err(anyhow::anyhow!("Could not hash password!"))
    }
    let hash = hash_result.unwrap().to_string();

    Ok(
        HashedPasswordRecord {
            hash,
            salt: salt.to_string()
        }
    )
}

async fn check_user_allocation(
    db_pool: &sqlx::MySqlPool,
    proposed_user: models::allocated_user::AllocatedUser,
) -> anyhow::Result<Option<AllocatedUser>>{
    // Check user is allocated
    let fetched_row: Option<AllocatedUser> = 
        sqlx::query_as!(
            AllocatedUser,
            "SELECT * FROM AllocatedUser WHERE username = ? AND secret = ?",
            proposed_user.username,
            proposed_user.secret
        )
        .fetch_optional(db_pool)
        .await?;

    Ok(fetched_row)
}

async fn remove_allocated_user(
    db_pool: &sqlx::MySqlPool,
    allocated_user: AllocatedUser
) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM AllocatedUser WHERE username = ? AND secret = ?",
        allocated_user.username,
        allocated_user.secret
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

async fn insert_user_into_db(
    user: models::user::User,
    db_pool: &sqlx::MySqlPool
) -> anyhow::Result<User> {
    sqlx::query!(
            "INSERT INTO User(id, username, email, description, created) VALUES (?, ?, ?, ?, ?);",
            user.id.to_string(),
            user.username,
            user.email,
            user.description,
            user.created
        )
        .execute(db_pool)
        .await?;

    Ok(user)
}

async fn insert_user_authentication_details_into_db(
    db_pool: &MySqlPool,
    authenticated_user_record: AuthenticatedUserRecord
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO Authentication (user_id, pass_hash, salt, stale, updated) VALUES (?, ?, ?, ?, ?);",
        authenticated_user_record.user_id.to_string(),
        authenticated_user_record.pass_hash,
        authenticated_user_record.salt,
        authenticated_user_record.stale,
        authenticated_user_record.updated
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

#[derive(Debug)]
enum SignupFlowErrors {
    UserNotAllocated
}

impl std::fmt::Display for SignupFlowErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserNotAllocated => write!(f, "You haven't been invited to use hrefsurf yet, or you provided an incorrect username/secret. We cannot specify more for security reasons.")
        }
    }
}

async fn perform_signup_flow(
    db_pool: &sqlx::MySqlPool,
    form_details: SignupFormDetails,
) -> anyhow::Result<()> {
    // MARK: Check for allocated user
    // TODO: Check username uniqueness!
    let allocated_user = AllocatedUser { 
        username: form_details.username.clone(), 
        secret: form_details.secret
    };
    let allocated_user 
        = check_user_allocation(db_pool, allocated_user).await?;
    if allocated_user.is_none() {
        return Err(anyhow::anyhow!(SignupFlowErrors::UserNotAllocated))
    }
    let allocated_user = allocated_user.unwrap();

    let user = User {
        id: uuid::Uuid::new_v4(),
        username: form_details.username,
        // TODO: Validate email
        email: form_details.email,
        description: "".to_owned(),
        created: Utc::now().naive_utc()
    };
    let password_record = hash_incoming_password(
        form_details.password
    )?;
    let authenticated_record = AuthenticatedUserRecord {
        user_id: user.id.clone(),
        pass_hash: password_record.hash,
        salt: password_record.salt,
        stale: false,
        updated: Utc::now().naive_utc()
    };


    // MARK: Make changes to database
    // Begin transaction for subsequent mutable changes to DB
    let add_user_transaction = db_pool.begin().await?;

    remove_allocated_user(db_pool, allocated_user).await?;
    insert_user_into_db(user, db_pool).await?;
    insert_user_authentication_details_into_db(db_pool, authenticated_record).await?;

    add_user_transaction.commit().await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct SignupFormDetails {
    pub username: String,
    pub password: String,
    pub email: String,
    pub secret: String,
}

fn render_signup(
    state: AppState
) -> Result<Html<String>, errors::HandlerErrors> {
    let context = Context::new();
    let render = state
        .tera
        .render("auth/signup.html", &context)?;

    Ok(Html(render))
}

#[axum_macros::debug_handler]
pub async fn get(
    State(state): State<AppState>
) -> Result<Html<String>, errors::HandlerErrors> {
    render_signup(state)
}

#[axum_macros::debug_handler]
pub async fn post(
    State(state): State<AppState>,
    Form(form_details): Form<SignupFormDetails>
) -> Result<Response, errors::HandlerErrors> {
    let result = perform_signup_flow(&state.db_pool, form_details).await;
    if result.is_err() {
        return Ok(render_signup(state).into_response()); // TODO: Prepopulate form
    }

    Ok(Redirect::to("/").into_response())
}
