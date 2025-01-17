use entity::{
    function::{ActiveModel as FunctionModel, Column, Model},
    prelude::Function,
};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbConn, EntityTrait, QueryFilter, ColumnTrait};

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
