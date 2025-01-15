use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Auth::Table)
                    .if_not_exists()
                    .col(pk_auto(Auth::Id))
                    .col(string(Auth::Email))
                    .col(string(Auth::Password))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Auth::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum Auth {
    Table,
    Id,
    Email,
    Password,
}
