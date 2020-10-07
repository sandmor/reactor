use image::DynamicImage;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn generate_from_path(path: impl AsRef<Path>) -> Result<Option<DynamicImage>, Box<dyn Error>> {
    let ext = match path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str().map(|s| s.to_owned()))
    {
        Some(ext) => ext,
        None => {
            return Ok(None);
        }
    };
    match &ext[..] {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "tiff" | "tif" | "webp" | "pnm"
        | "pbm" | "pgm" | "ppm" | "pam" | "dds" | "tga" | "ff" => {
            use image::*;
            let file = BufReader::new(File::open(path)?);
            let format = match &ext[..] {
                "png" => ImageFormat::Png,
                "jpg" | "jpeg" => ImageFormat::Jpeg,
                "gif" => ImageFormat::Gif,
                "webp" => ImageFormat::WebP,
                "tiff" | "tif" => ImageFormat::Tiff,
                "pnm" | "pbm" | "pgm" | "ppm" | "pam" => ImageFormat::Pnm,
                "tga" => ImageFormat::Tga,
                "dds" => ImageFormat::Dds,
                "bmp" => ImageFormat::Bmp,
                "ico" => ImageFormat::Ico,
                "ff" => ImageFormat::Farbfeld,
                _ => unreachable!(),
            };
            let img = load(file, format)?;
            Ok(Some(img))
        }
        _ => Ok(None),
    }
}
