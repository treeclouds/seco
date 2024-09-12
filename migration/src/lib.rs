#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_users;
mod m20231103_114510_notes;

mod m20240327_124407_add_users_first_name_last_name;
mod m20240327_130532_delete_users_name;
mod m20240327_131345_index_users_more_fields;
mod m20240328_042743_products;
mod m20240328_095301_delete_notes;
mod m20240329_041116_product_images;
mod m20240329_164332_add_users_is_active_is_superuser;
mod m20240828_113705_categories;
mod m20240830_142527_alter_parent_id_categories;
mod m20240903_112445_alter_products_category;
mod m20240909_030131_add_users_location_and_image;
mod m20240909_031109_index_users_location;
mod m20240909_031322_index_products_brand;
mod m20240909_112205_wishlists;
mod m20240912_033907_offerings;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_users::Migration),
            Box::new(m20231103_114510_notes::Migration),
            Box::new(m20240327_124407_add_users_first_name_last_name::Migration),
            Box::new(m20240327_130532_delete_users_name::Migration),
            Box::new(m20240327_131345_index_users_more_fields::Migration),
            Box::new(m20240328_042743_products::Migration),
            Box::new(m20240328_095301_delete_notes::Migration),
            Box::new(m20240329_041116_product_images::Migration),
            Box::new(m20240329_164332_add_users_is_active_is_superuser::Migration),
            Box::new(m20240828_113705_categories::Migration),
            Box::new(m20240830_142527_alter_parent_id_categories::Migration),
            Box::new(m20240903_112445_alter_products_category::Migration),
            Box::new(m20240909_030131_add_users_location_and_image::Migration),
            Box::new(m20240909_031109_index_users_location::Migration),
            Box::new(m20240909_031322_index_products_brand::Migration),
            Box::new(m20240909_112205_wishlists::Migration),
            Box::new(m20240912_033907_offerings::Migration),
        ]
    }
}