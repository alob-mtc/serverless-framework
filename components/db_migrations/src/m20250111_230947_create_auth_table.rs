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
                    .col(uuid(Auth::Uuid))
                    .to_owned(),
            )
            .await?;

        // Create a unique index on email
        manager
            .create_index(
                Index::create()
                    .name("idx-auth-email-unique")
                    .table(Auth::Table)
                    .col(Auth::Email)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index first
        manager
            .drop_index(Index::drop().name("idx-auth-email-unique").to_owned())
            .await?;

        // Then drop table
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
    Uuid,
}
