use std::{path::PathBuf, fs};
use rand::Rng;

use crate::consts::DEFAULT_THUMBNAIL_PATH;


pub fn get_thumbnail_path(
    id: &String,
    category: &String,
    thumbnail_root_path: &str,
) -> Result<PathBuf, actix_web::Error> {
    let striped_option = thumbnail_root_path.strip_suffix('/');
    let striped_thumbnail_root_path: &str = match striped_option {
        Some(striped) => striped,
        None => thumbnail_root_path,
    };
    let result = fs::read_dir(format!(
        "{}/{}/{}",
        striped_thumbnail_root_path, category, id
    ));
    let files: Vec<_> = match result {
        Ok(read_dir) => read_dir.collect(),
        Err(_) => return Ok(PathBuf::from(DEFAULT_THUMBNAIL_PATH))
    };
    if files.is_empty() {
        return Ok(PathBuf::from(DEFAULT_THUMBNAIL_PATH));
    }
    let mut rng = rand::thread_rng();
    let random = rng.gen_range(0..(files.len() - 1));
    match &files[random] {
        Ok(dir) => Ok(dir.path()),
        Err(err) => {
            Err(err).map_err(|err| actix_web::error::ErrorInternalServerError(err.to_string()))
        }
    }
}
