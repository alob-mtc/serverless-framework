use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use entity::{
    auth::{ActiveModel as AuthModel, Column as AuthColumn, Entity as Auth, Model as AuthUser},
    prelude::Auth as AuthEntity,
};
use rand_core::OsRng;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter,
};
use tracing::{error, info};
use uuid::Uuid;

pub struct AuthDBRepo;

impl AuthDBRepo {
    /// Registers a new user with the provided email and password
    ///
    /// # Arguments
    ///
    /// * `conn` - A reference to the database connection
    /// * `email` - The email address for the new user
    /// * `password` - The password for the new user
    ///
    /// # Returns
    ///
    /// * `Ok(AuthUser)` - The newly created user
    /// * `Err(DbErr)` - If registration fails (e.g., duplicate email)
    pub async fn register(
        conn: &DbConn,
        email: String,
        password: String,
    ) -> Result<AuthUser, DbErr> {
        // Check if user with this email already exists
        if let Some(_) = AuthEntity::find()
            .filter(AuthColumn::Email.eq(&email))
            .one(conn)
            .await?
        {
            return Err(DbErr::Custom("Email already registered".to_string()));
        }

        // Hash the password with Argon2
        let hashed_password = Self::hash_password(&password)?;

        // Create the user model
        let user = AuthModel {
            id: Default::default(),
            email: Set(email),
            password: Set(hashed_password),
            uuid: Set(Uuid::new_v4()),
        };

        // Save the user to the database
        Ok(user.insert(conn).await?)
    }

    /// Login a user with the provided email and password
    ///
    /// # Arguments
    ///
    /// * `conn` - A reference to the database connection
    /// * `email` - The email address of the user
    /// * `password` - The password for the user
    ///
    /// # Returns
    ///
    /// * `Ok(AuthUser)` - The logged in user
    /// * `Err(DbErr)` - If login fails (e.g., invalid credentials)
    pub async fn login(conn: &DbConn, email: String, password: String) -> Result<AuthUser, DbErr> {
        // Find the user by email
        let user = match AuthEntity::find()
            .filter(AuthColumn::Email.eq(&email))
            .one(conn)
            .await?
        {
            Some(user) => user,
            None => return Err(DbErr::Custom("Invalid credentials".to_string())),
        };

        // Verify the password
        if !Self::verify_password(&password, &user.password)? {
            return Err(DbErr::Custom("Invalid credentials".to_string()));
        }

        Ok(user)
    }

    /// Find a user by their UUID
    ///
    /// # Arguments
    ///
    /// * `conn` - A reference to the database connection
    /// * `uuid` - The UUID of the user to find
    ///
    /// # Returns
    ///
    /// * `Ok(Some(AuthUser))` - The user, if found
    /// * `Ok(None)` - If no user with the UUID exists
    /// * `Err(DbErr)` - If an error occurs during the database operation
    pub async fn find_by_uuid(conn: &DbConn, uuid: Uuid) -> Result<Option<AuthUser>, DbErr> {
        AuthEntity::find()
            .filter(AuthColumn::Uuid.eq(uuid))
            .one(conn)
            .await
    }

    /// Hash a password using Argon2
    fn hash_password(password: &str) -> Result<String, DbErr> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        // Hash the password with Argon2
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| DbErr::Custom(format!("Failed to hash password: {}", e)))
    }

    /// Verify a password against a previously hashed password
    fn verify_password(password: &str, hash: &str) -> Result<bool, DbErr> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| DbErr::Custom(format!("Failed to parse password hash: {}", e)))?;

        // Verify the password
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
