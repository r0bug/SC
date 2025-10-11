use crate::validation;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use core_domain::User;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use local_store::repositories::UserRepository;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use uuid::Uuid;

const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub email_verified: bool,
    pub active: bool,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            email_verified: user.email_verified,
            active: user.active,
        }
    }
}

pub struct AuthService {
    pool: Arc<Pool<Sqlite>>,
}

impl AuthService {
    pub fn new(pool: Arc<Pool<Sqlite>>) -> Self {
        Self { pool }
    }

    pub async fn signup(&self, req: SignupRequest) -> Result<AuthResponse, AuthError> {
        // Validate inputs
        validation::validate_email(&req.email).map_err(|_| AuthError::InvalidCredentials)?;
        validation::validate_name(&req.name).map_err(|_| AuthError::InvalidCredentials)?;
        validation::validate_password(&req.password).map_err(|_| AuthError::InvalidCredentials)?;

        let repo = UserRepository::new(&self.pool);

        // Check if user already exists
        if let Ok(_) = repo.get_by_email(&req.email).await {
            return Err(AuthError::UserAlreadyExists);
        }

        // Hash password
        let password_hash =
            hash(req.password.as_bytes(), DEFAULT_COST).map_err(|_| AuthError::InternalError)?;

        // Generate API token
        let api_token = generate_api_token();

        // Create user
        let user = User {
            id: Uuid::new_v4(),
            email: req.email,
            name: req.name,
            password_hash,
            api_token: Some(api_token),
            email_verified: false,
            active: true,
            preferences: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login_at: None,
        };

        repo.create(&user)
            .await
            .map_err(|_| AuthError::InternalError)?;

        // Generate JWT
        let token = generate_jwt(&user.id)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AuthError> {
        // Validate inputs
        validation::validate_email(&req.email).map_err(|_| AuthError::InvalidCredentials)?;

        let repo = UserRepository::new(&self.pool);

        // Get user by email
        let mut user = repo
            .get_by_email(&req.email)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        // Verify password
        verify(req.password.as_bytes(), &user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        // Check if user is active
        if !user.active {
            return Err(AuthError::UserInactive);
        }

        // Update last login
        user.last_login_at = Some(chrono::Utc::now());
        repo.update(&user)
            .await
            .map_err(|_| AuthError::InternalError)?;

        // Generate JWT
        let token = generate_jwt(&user.id)?;

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub async fn get_user_by_token(&self, token: &str) -> Result<User, AuthError> {
        // Try JWT first
        if let Ok(user_id) = validate_jwt(token) {
            let repo = UserRepository::new(&self.pool);
            return repo
                .get_by_id(user_id)
                .await
                .map_err(|_| AuthError::InvalidToken);
        }

        // Try API token
        let repo = UserRepository::new(&self.pool);
        repo.get_by_api_token(token)
            .await
            .map_err(|_| AuthError::InvalidToken)
    }

    pub async fn refresh_token(&self, old_token: &str) -> Result<String, AuthError> {
        let user_id = validate_jwt(old_token)?;
        generate_jwt(&user_id)
    }
}

fn generate_jwt(user_id: &Uuid) -> Result<String, AuthError> {
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::hours(24)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|_| AuthError::InternalError)
}

fn validate_jwt(token: &str) -> Result<Uuid, AuthError> {
    let token_data: TokenData<Claims> = decode(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map_err(|_| AuthError::InvalidToken)?;

    Uuid::parse_str(&token_data.claims.sub).map_err(|_| AuthError::InvalidToken)
}

fn generate_api_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    InvalidToken,
    UserAlreadyExists,
    UserInactive,
    InternalError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthError::UserInactive => (StatusCode::FORBIDDEN, "User account is inactive"),
            AuthError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        let body = Json(serde_json::json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

// Authentication middleware
#[derive(Clone)]
pub struct AuthUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    crate::state::AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AuthError::InvalidToken)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::InvalidToken)?;

        // Get app state from state
        let app_state = crate::state::AppState::from_ref(state);

        // Validate token and get user
        let user = app_state.auth_service.get_user_by_token(token).await?;

        // Check if user is active and email is verified
        if !user.active {
            return Err(AuthError::UserInactive);
        }

        Ok(AuthUser(user))
    }
}
