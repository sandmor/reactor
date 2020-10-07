use crate::{files::Directory, items_view::ItemsView, WATCHING_DIRS, file_content_info::FileContentInfo};
use log::{debug, error, trace};
use orbtk::prelude::*;
use std::{path::PathBuf, ops::{Deref, DerefMut}, fs::FileType};
use mime::Mime;

#[derive(Debug)]
pub enum Event {
    SelectionChanged(Vec<usize>),
}

#[derive(Default, AsAny)]
struct FilesViewState {
    directory: Directory,
    event: Option<Event>,
    last_path: PathBuf,
}

impl State for FilesViewState {
    fn init(&mut self, _registry: &mut Registry, ctx: &mut Context) {
        trace!("FilesView init invoked");
        let path = ctx
            .widget()
            .get::<PathBufWrapper>("path")
            .as_path()
            .to_owned();
        self.directory = Directory::new(path.clone()).unwrap();
        ctx.widget().get_mut::<PathBufWrapper>("path").0 = self.directory.path().to_path_buf();
        for fi in self.directory.files() {
            ctx.widget().get_mut::<FilesInfo>("files_info").push(FileInfo { file_name: fi.file_name.clone(), file_type: fi.file_type, content_info: fi.content_info.lock().clone(), extension_mime: fi.extension_mime.clone() });
        }
        ItemsView::count_set(&mut ctx.child("directory_view"), self.directory.len());
        ItemsView::request_update_set(&mut ctx.child("directory_view"), true);
        WATCHING_DIRS.lock().unwrap().insert(path, ctx.event_adapter());
    }

    fn update(&mut self, _: &mut Registry, ctx: &mut Context) {
        trace!("FilesView update invoked");
        let current_path = ctx
            .widget()
            .get::<PathBufWrapper>("path")
            .as_path()
            .to_owned();
        if current_path != self.last_path {
            trace!("Updating FileView");
            ItemsView::count_set(&mut ctx.child("directory_view"), self.directory.len());
            ItemsView::request_update_set(&mut ctx.child("directory_view"), true);
            self.last_path = current_path;
        }
        if let Some(event) = self.event.take() {
            match event {
                Event::SelectionChanged(changes) => {
                    ctx.event_adapter().push_event_direct(
                        ctx.entity,
                        SelectionChangedEvent(ctx.entity, changes),
                    );
                }
            }
        }
    }
}

impl FilesViewState {
    fn event(&mut self, event: impl Into<Option<Event>>) {
        self.event = event.into();
    }
}

#[derive(Debug, Default, AsAny, PartialEq, Eq, Clone)]
pub struct PathBufWrapper(PathBuf);

into_property_source!(PathBufWrapper);

impl Deref for PathBufWrapper {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PathBufWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<PathBuf> for PathBufWrapper {
    fn from(path: PathBuf) -> PathBufWrapper {
        PathBufWrapper(path)
    }
}

#[derive(Debug, AsAny, PartialEq, Eq, Clone)]
pub struct FileInfo {
    file_name: String,
    file_type: FileType,
    content_info: Option<FileContentInfo>,
    extension_mime: Option<Mime>
}

impl FileInfo {
    pub fn media_type(&self) -> Option<Mime> {
        self.extension_mime.clone().or_else(|| self.content_info.as_ref().and_then(|ci| ci.mime.clone()))
    }
}

type FilesInfo = Vec<FileInfo>;

into_property_source!(FileInfo);

widget!(FilesView<FilesViewState>: SelectionChangedHandler {
    path: PathBufWrapper,
    files_info: FilesInfo
});

impl Template for FilesView {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("FilesView").child(
            ItemsView::new()
                .id("directory_view")
                .on_selection_changed(move |states, _, change| {
                    states
                        .get_mut::<FilesViewState>(id)
                        .event(Event::SelectionChanged(change));
                })
                .items_builder(move |bc, index| {
                    let entry = bc
                        .get_widget(id)
                        .get::<FilesInfo>("files_info")
                        .get(index)
                        .unwrap()
                        .clone();
                    let path = bc
                        .get_widget(id)
                        .get::<PathBufWrapper>("path")
                        .join(entry.file_name.clone());
                    let icon;
                    if entry.file_type.is_dir() {
                        icon = Err(material_icons_font::MD_FOLDER);
                    } else {
                        match entry.content_info.as_ref().and_then(|ci| ci.thumbnail.clone()) {
                            Some(image) => {
                                icon = Ok(Image::from_rgba_image(image).unwrap());
                            }
                            _ => {
                                if let Some(s) = entry.file_name.rfind('.') {
                                    let ext = &entry.file_name[s + 1..];
                                    icon = Err(match &ext.to_lowercase()[..] {
                                        "c" | "hpp" | "cpp" | "cxx" | "rb" | "py" | "rs" | "js" | "css"
                                        | "html" | "php" | "xml" => material_icons_font::MD_CODE,
                                        _ => match entry.media_type() {
                                            Some(mime) => match mime.type_() {
                                                mime::AUDIO => material_icons_font::MD_AUDIOTRACK,
                                                mime::IMAGE => material_icons_font::MD_IMAGE,
                                                mime::TEXT => material_icons_font::MD_TEXT_SNIPPET,
                                                _ => material_icons_font::MD_ARCHIVE,
                                            },
                                            None => material_icons_font::MD_ARCHIVE,
                                        },
                                    });
                                } else {
                                    icon = Err(material_icons_font::MD_ARCHIVE);
                                }
                            }
                        }
                    }
                    let mut file_name = String::with_capacity(
                        entry.file_name.len() + entry.file_name.len() / 10 + 1,
                    );
                    let mut i = 0;
                    for ch in entry.file_name.chars() {
                        file_name.push(ch);
                        i += 1;
                        if i == 10 {
                            i = 0;
                            file_name.push('\n');
                        }
                    }
                    let icon_widget = match icon {
                        Ok(image) => ImageWidget::new()
                            .image(image)
                            .h_align("center")
                            .v_align("center")
                            .build(bc),
                        Err(icon) => FontIconBlock::new()
                            .icon(icon)
                            .icon_size(48)
                            .h_align("center")
                            .v_align("center")
                            .build(bc),
                    };
                    Stack::new()
                        .child(icon_widget)
                        .child(
                            TextBlock::new()
                                .margin((0, 0, 0, 2))
                                .h_align("center")
                                .v_align("center")
                                .text(file_name)
                                .build(bc),
                        )
                        .build(bc)
                })
                .build(ctx),
        )
    }
}
