use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::models::_entities::product_images::Model;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductImageResponse {
    pub id: i32,
    pub image: String,
}

impl ProductImageResponse {
    #[must_use]
    pub fn new(product_image: &Model) -> Self {
        Self {
            id: product_image.id,
            image: product_image.image.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageResponse {
    pub path: PathBuf,
}

impl ImageResponse {
    #[must_use]
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }
}
