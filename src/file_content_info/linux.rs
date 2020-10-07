use image::DynamicImage;
use lazy_static::lazy_static;
use log::error;
use mime::Mime;
use std::{
    collections::HashMap,
    env::temp_dir,
    error::Error,
    fs::{remove_file, File},
    io::{self, BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
};

lazy_static! {
    static ref THUMBNAILERS: HashMap<Mime, Vec<String>> = {
        let mut thumbnailers = HashMap::new();
        let folders = ["/usr/share/thumbnailers/"];
        for folder in folders.iter() {
            let dir = match Path::new(folder).read_dir() {
                Ok(dir) => dir,
                Err(_) => {
                    continue;
                }
            };
            for file in dir {
                let file = match file {
                    Ok(file) => file,
                    Err(_) => {
                        continue;
                    }
                };
                match file.file_type() {
                    Ok(t) if t.is_file() => {}
                    _ => {
                        continue;
                    }
                };
                let config = match parse_config_file(file.path()) {
                    Ok(config) => config,
                    Err(_) => {
                        continue;
                    }
                };
                for (header, values) in config {
                    if header != "Thumbnailer Entry" {
                        continue;
                    }
                    let exec = match values.get("Exec").filter(|e| !e.is_empty()) {
                        Some(exec) => exec
                            .split(' ')
                            .map(|s| s.to_owned())
                            .collect::<Vec<String>>(),
                        None => {
                            continue;
                        }
                    };
                    let mime = match values.get("MimeType") {
                        Some(mime) => mime,
                        _ => {
                            continue;
                        }
                    };
                    for mime in mime.split(';').filter_map(|v| v.parse().ok()) {
                        thumbnailers.insert(mime, exec.clone());
                    }
                }
            }
        }
        thumbnailers
    };
}

// Support other enconding than UTF-8
fn parse_config_file(
    path: impl AsRef<Path>,
) -> Result<Vec<(String, HashMap<String, String>)>, io::Error> {
    let mut sections = Vec::new();
    let mut lines = BufReader::new(File::open(path)?).lines();
    while let Some(mut line) = lines.next().transpose()?.map(|line| line.trim().to_owned()) {
        if line.is_empty() {
            while let Some(nline) = lines
                .next()
                .transpose()?
                .map(|nline| nline.trim().to_owned())
            {
                if !nline.is_empty() {
                    line = nline;
                    break;
                }
            }
        }
        if line.len() < 3 || !line.starts_with('[') || !line.ends_with(']') {
            return Ok(Vec::new());
        }
        let header = line[1..line.len() - 1].to_owned();
        let mut entries = HashMap::new();
        while let Some(line) = lines.next().transpose()? {
            let mut line = line.splitn(2, '=');
            let key = match line.next() {
                Some(key) => key.trim_end().to_owned(),
                None => {
                    continue;
                }
            };
            let value = match line.next() {
                Some(value) => value.trim_start().to_owned(),
                None => {
                    return Ok(Vec::new());
                }
            };
            entries.insert(key, value);
        }
        sections.push((header, entries));
    }
    Ok(sections)
}

pub fn for_path(
    path: impl AsRef<Path>,
    mime: Mime,
    size: u32,
) -> Result<Option<DynamicImage>, Box<dyn Error>> {
    if path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| e.to_ascii_lowercase() == "desktop")
        .is_some()
    {
        let config = parse_config_file(path.as_ref())?;
        for (header, values) in config {
            if header != "Desktop Entry" {
                continue;
            }
            if let Some(ref _icon) = values.get("Icon") {
                // TODO
            }
        }
    }
    if let Some(ref cmd) = THUMBNAILERS.get(&mime) {
        let mut output_path = temp_dir().join("reactor_thumbnail_output0");
        let mut count = 0;
        while output_path.exists() {
            output_path.pop();
            count += 1;
            output_path.push(&format!("reactor_thumbnail_output{}", count));
        }
        let mut args: Vec<String> = cmd[1..].to_vec();
        let input_arg = format!("{}", path.as_ref().display());
        let mut input_arg_used = false;
        let output_arg = format!("{}", output_path.display());
        let mut output_arg_used = false;
        for arg in args.iter_mut() {
            match &arg[..] {
                "%u" => {
                    *arg = input_arg.clone();
                    input_arg_used = true;
                }
                "%o" => {
                    *arg = output_arg.clone();
                    output_arg_used = true;
                }
                "%s" => {
                    *arg = format!("{}", size);
                }
                _ => {}
            }
        }
        if !input_arg_used {
            args.push(input_arg);
        }
        if !output_arg_used {
            args.push(output_arg);
        }
        let res = Command::new(&cmd.first().unwrap())
            .args(args)
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .output()?;
        if !res.status.success() {
            let _ = remove_file(output_path);
            return Ok(None);
        }
        let mut img_buf = vec![];
        File::open(&output_path)?.read_to_end(&mut img_buf)?;
        let img = image::load_from_memory(&img_buf)?;
        let _ = remove_file(output_path);
        return Ok(Some(img));
    }
    Ok(None)
}
