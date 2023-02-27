use sea_orm::Statement;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = r#"
        CREATE TABLE user_groups (
            user_id INTEGER REFERENCES users(user_id),
            group_id INTEGER REFERENCES access_groups(group_id),
            PRIMARY KEY (user_id, group_id)
        )"#;

        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql = "DROP TABLE IF EXISTS user_groups";
        let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
        manager.get_connection().execute(stmt).await.map(|_| ())
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[allow(dead_code)]
#[derive(Iden)]
enum UserGroup {
    Table,
    UserId,
    GroupId,
}
