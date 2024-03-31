use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FileResponse {
    pub product_id: i32,
    pub path: PathBuf,
}

impl FileResponse {
    #[must_use]
    pub fn new(path: &Path, product_id: i32) -> Self {
        Self {
            product_id: product_id,
            path: path.to_path_buf(),
        }
    }
}