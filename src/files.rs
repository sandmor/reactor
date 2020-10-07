use crate::file_content_info::FileContentInfo;
use mime::Mime;
use std::{
    cmp::Ordering,
    fs::FileType,
    io,
    path::{Path, PathBuf},
    sync::Arc
};
use parking_lot::Mutex;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub file_name: String,
    pub file_type: FileType,
    pub content_info: Arc<Mutex<Option<FileContentInfo>>>,
    pub extension_mime: Option<Mime>
}

#[derive(Debug, Clone)]
pub struct Directory {
    files: Vec<FileInfo>,
    path: PathBuf,
}

impl PartialEq for Directory {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl Directory {
    pub fn new(path: impl AsRef<Path>) -> Result<Directory, io::Error> {
        let mut dir = Self {
            files: Vec::new(),
            path: path.as_ref().to_path_buf(),
        };
        dir.change_path(path)?;
        Ok(dir)
    }

    fn change_path(&mut self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let path = path.as_ref().canonicalize()?;
        self.files.clear();
        for item in path.read_dir()? {
            let item = item?;
            let meta = item.metadata()?;
            let file_type = meta.file_type();
            let file_name = item.file_name().to_string_lossy().into_owned();
            if file_name.starts_with(".") {
                continue;
            }
            let content_info = Arc::new(Mutex::new(None));
            //let content_info = crate::file_content_info::for_path(item.path(), 48).unwrap();
            if file_type.is_file() {
                let path = item.path();
                let content_info = content_info.clone();
                rayon::spawn(move || {
                    let done_content_info = crate::file_content_info::for_path(path, 48).unwrap();
                    content_info.lock().replace(done_content_info);
                });
            }
            let info = FileInfo {
                file_name,
                file_type,
                content_info: content_info,
                extension_mime: mime_guess::from_path(item.path()).first()
            };
            self.files.push(info);
        }
        self.files
            .sort_unstable_by(|a, b| match (a.file_type.is_dir(), b.file_type.is_dir()) {
                (true, true) | (false, false) => a.file_name.cmp(&b.file_name),
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
            });
        self.path = path;
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<&FileInfo> {
        self.files.get(index)
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn files(&self) -> impl Iterator<Item=&FileInfo> {
        self.files.iter()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn go_up(&mut self) -> Result<(), io::Error> {
        self.path.pop();
        self.change_path(self.path.clone())?;
        Ok(())
    }

    pub fn go_deeper(&mut self, dir_index: usize) -> Result<(), io::Error> {
        self.change_path(self.path.clone().join(&self.files[dir_index].file_name))?;
        Ok(())
    }

    pub fn set_path(&mut self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        self.change_path(path)?;
        Ok(())
    }
}

impl Default for Directory {
    fn default() -> Self {
        Self::new(".").unwrap()
    }
}

pub fn amend_path(path: impl AsRef<Path>) -> Result<Option<PathBuf>, io::Error> {
    let broken_path = path.as_ref();
    let mut path = broken_path.clone().to_path_buf();
    while !path.exists() {
        if !path.pop() {
            return Ok(Some(broken_path.to_path_buf()));
        }
    }
    let valid_path_segs_count = path.iter().count();
    for segment in broken_path.iter().skip(valid_path_segs_count) {
        path.push(segment);
        if !path.exists() {
            path.pop();
            let searching_name = match segment.to_str() {
                Some(n) => n.to_lowercase(),
                None => {
                    return Ok(None);
                }
            };
            let mut found = false;
            for sibling in path.read_dir()? {
                let sibling_name_os = sibling?.file_name();
                let sibling_name = match sibling_name_os.to_str() {
                    Some(n) => n,
                    None => {
                        continue;
                    }
                };
                if sibling_name.to_lowercase() == searching_name {
                    path.push(sibling_name);
                    found = true;
                    break;
                }
            }
            if !found || !path.exists() {
                return Ok(None);
            }
        }
    }
    Ok(Some(path))
}
