use sea_orm_migration::prelude::*;

mod m20251015_085200_create_editions;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20251015_085200_create_editions::Migration)]
    }
}
