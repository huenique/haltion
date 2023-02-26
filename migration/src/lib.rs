pub use sea_orm_migration::prelude::*;

mod m20230209_110348_create_otps_table;
mod m20230225_062319_create_users_table;
mod m20230225_062404_create_access_groups_table;
mod m20230225_062412_create_permissions_table;
mod m20230225_064453_create_user_groups_table;
mod m20230225_064502_create_group_permissions_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230209_110348_create_otps_table::Migration),
            Box::new(m20230225_062319_create_users_table::Migration),
            Box::new(m20230225_062404_create_access_groups_table::Migration),
            Box::new(m20230225_062412_create_permissions_table::Migration),
            Box::new(m20230225_064453_create_user_groups_table::Migration),
            Box::new(m20230225_064502_create_group_permissions_table::Migration),
        ]
    }
}
