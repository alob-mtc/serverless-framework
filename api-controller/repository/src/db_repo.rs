use entity::{
    auth::{ActiveModel as AuthModel, Column as AuthColumn},
    function::{ActiveModel as FunctionModel, Column, Model},
    prelude::{Auth, Function},
};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter};

pub struct FunctionDBRepo;

impl FunctionDBRepo {
    // Find a function by its name
    pub async fn find_function_by_name(conn: &DbConn, name: &str) -> Option<Model> {
        Function::find()
            .filter(Column::Name.eq(name))
            .one(conn)
            .await
            .ok()?
    }

    // Create a new function
    pub async fn create_function(conn: &DbConn, function: Model) {
        match Auth::find().filter(AuthColumn::Id.eq(1)).one(conn).await {
            Ok(v) => {
                if v.is_none() {
                    AuthModel {
                        email: Set("test@gmail.com".to_string()),
                        password: Set("secret".to_string()),
                        ..Default::default()
                    }
                    .save(conn)
                    .await
                    .expect("Failed to create auth in DB");
                }
            }
            Err(e) => {
                println!("Error finding auth: {}", e);
            }
        }

        // Example: change `auth_id` to something dynamic.
        FunctionModel {
            auth_id: Set(1), // This could come from another data source
            name: Set(function.name),
            runtime: Set(function.runtime),
            ..Default::default()
        }
        .save(conn)
        .await
        .expect("Failed to create function in DB");
    }
}
