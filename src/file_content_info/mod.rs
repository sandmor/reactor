use image::{DynamicImage, RgbaImage};
use mime::Mime;
use std::{error::Error, path::Path};

mod embedded_thumbnailers_generators;

#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileContentInfo {
    pub thumbnail: Option<RgbaImage>,
    pub mime: Option<Mime>,
}

pub fn for_path(
    path: impl AsRef<Path>,
    thumbnails_size: u32,
) -> Result<FileContentInfo, Box<dyn Error>> {
    let mime = mime_guess::from_path(path.as_ref())
        .first()
        .or(tree_magic::from_filepath(path.as_ref()).parse().ok());
    let mut thumbnail: Option<DynamicImage> = None;
    #[cfg(target_os = "linux")]
    if let Some(mime) = mime.clone() {
        if let Some(t) = linux::for_path(path.as_ref(), mime, thumbnails_size)? {
            thumbnail = Some(t);
        }
    }
    if thumbnail.is_none() {
        let ext = match path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str().map(|s| s.to_owned()))
        {
            Some(ext) => ext,
            None => "".to_owned(),
        };
        if let Some(t) = embedded_thumbnailers_generators::generate_from_path(path.as_ref())? {
            thumbnail = Some(t);
        }
    }
    let thumbnail = thumbnail.map(|t| t.thumbnail(thumbnails_size, thumbnails_size).into_rgba());
    Ok(FileContentInfo { thumbnail, mime })
}
