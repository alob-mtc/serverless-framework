use entity::{
    function::{ActiveModel as FunctionModel, Model},
    prelude::Function,
};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbConn, EntityTrait};
pub struct FunctionDBRepo;

impl FunctionDBRepo {
    pub async fn find_function_by_name(conn: &DbConn, name: &str) -> Option<Model> {
        // TODO: fix
        Function::find().one(conn).await.unwrap()
    }
    pub async fn create_function(conn: &DbConn, function: Model) {
        FunctionModel {
            auth_id: Set(1), // this should change
            name: Set(function.name),
            runtime: Set(function.runtime),
            ..Default::default()
        }
        .save(conn)
        .await
        .unwrap();
    }
}
