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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
}
