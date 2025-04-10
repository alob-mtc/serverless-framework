pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250111_230947_create_auth_table::Migration),
            Box::new(m20250111_231042_create_function_table::Migration),
        ]
    }
}
mod m20250111_230947_create_auth_table;
mod m20250111_231042_create_function_table;
