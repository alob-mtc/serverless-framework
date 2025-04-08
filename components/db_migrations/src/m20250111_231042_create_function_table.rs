use crate::m20250111_230947_create_auth_table::Auth;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Function::Table)
                    .if_not_exists()
                    .col(pk_auto(Function::Id))
                    .col(string(Function::Name))
                    .col(string(Function::Runtime).default("go"))
                    .col(uuid(Function::Uuid))
                    .col(integer(Function::AuthId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-function-auth_id")
                            .from(Function::Table, Function::AuthId)
                            .to(Auth::Table, Auth::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create a unique index on name and auth_id combination
        // This prevents the same user from creating two functions with the same name
        manager
            .create_index(
                Index::create()
                    .name("idx-function-name-auth-unique")
                    .table(Function::Table)
                    .col(Function::Name)
                    .col(Function::AuthId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index first
        manager
            .drop_index(
                Index::drop()
                    .name("idx-function-name-auth-unique")
                    .to_owned(),
            )
            .await?;

        // Then drop table
        manager
            .drop_table(Table::drop().table(Function::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Function {
    Table,
    Id,
    Name,
    Runtime,
    AuthId,
    Uuid,
}
