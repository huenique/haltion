pub use sea_orm_migration::prelude::*;

mod m20230130_073549_create_employees_table;
mod m20230209_110348_create_otps_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20230209_110348_create_otps_table::Migration)]
    }
}
