use sea_orm_migration::prelude::*;

mod m20251015_085200_create_editions;
mod m20251015_105110_add_views;
mod m20251105_094423_add_title_to_edition;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251015_085200_create_editions::Migration),
            Box::new(m20251015_105110_add_views::Migration),
            Box::new(m20251105_094423_add_title_to_edition::Migration),
        ]
    }
}
