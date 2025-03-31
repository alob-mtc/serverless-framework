use entity::{
    auth::{ActiveModel as AuthModel, Column as AuthColumn},
    function::{ActiveModel as FunctionModel, Column, Model},
    prelude::{Auth, Function},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DbConn, EntityTrait, QueryFilter,
};
use tracing::error;

pub struct FunctionDBRepo;

impl FunctionDBRepo {
    /// Finds a function by its name in the database.
    ///
    /// # Arguments
    ///
    /// * `conn` - A reference to the database connection.
    /// * `name` - The name of the function to find.
    ///
    /// # Returns
    ///
    /// * `Some(Model)` if the function exists; otherwise, `None`.
    pub async fn find_function_by_name(conn: &DbConn, name: &str) -> Option<Model> {
        Function::find()
            .filter(Column::Name.eq(name))
            .one(conn)
            .await
            .ok()?
    }

    /// Creates a new function in the database.
    ///
    /// This function first ensures that an auth record with id 1 exists. If not,
    /// it creates a default auth entry. Then, it inserts a new function record.
    ///
    /// # Arguments
    ///
    /// * `conn` - A reference to the database connection.
    /// * `function` - The function model to insert.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success, or an error of type `sea_orm::DbErr` if insertion fails.
    pub async fn create_function(conn: &DbConn, function: Model) -> Result<(), sea_orm::DbErr> {
        // Ensure that the auth record with id 1 exists.
        match Auth::find().filter(AuthColumn::Id.eq(1)).one(conn).await {
            Ok(Some(_)) => {
                // Auth exists, nothing to do.
            }
            Ok(None) => {
                // Create a default auth record.
                let auth_model = AuthModel {
                    email: Set("test@gmail.com".to_string()),
                    password: Set("secret".to_string()),
                    ..Default::default()
                };
                auth_model.save(conn).await?;
            }
            Err(e) => {
                error!("Error finding auth: {}", e);
                return Err(e);
            }
        }

        // Create the function model record.
        let function_model = FunctionModel {
            auth_id: Set(1), // TODO: Replace with a dynamic auth_id if needed.
            name: Set(function.name),
            runtime: Set(function.runtime),
            ..Default::default()
        };
        function_model.save(conn).await?;
        Ok(())
    }
}
