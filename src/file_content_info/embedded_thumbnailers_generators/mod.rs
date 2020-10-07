use image::DynamicImage;
use std::error::Error;
use std::path::Path;

mod images;

pub fn generate_from_path(path: impl AsRef<Path>) -> Result<Option<DynamicImage>, Box<dyn Error>> {
    if let Some(image) = images::generate_from_path(path)? {
        return Ok(Some(image));
    }
    Ok(None)
}
