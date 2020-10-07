use log::{debug, error};
use orbtk::prelude::*;
use std::{ops::{Deref, DerefMut}, path::PathBuf};

use crate::files::*;
use crate::files_view::*;
use crate::items_view::*;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct PlacesWrapper(Vec<(String, PathBuf)>);

impl Deref for PlacesWrapper {
    type Target = Vec<(String, PathBuf)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PlacesWrapper {
    fn deref_mut(&mut self) -> &mut Vec<(String, PathBuf)> {
        &mut self.0
    }
}

#[derive(Debug)]
enum Event {
    ClickOnDirContent(usize),
    Undo,
    Redo,
    MoveUp,
    GoToHome,
    RequestManualPathChange,
}

#[derive(Default, AsAny)]
struct MainViewState {
    event: Option<Event>,
    path_history: Vec<PathBuf>,
    path_history_cursor: usize,
}

into_property_source!(PlacesWrapper);

widget!(MainView<MainViewState> {
    places: PlacesWrapper
});

impl State for MainViewState {
    fn init(&mut self, _registry: &mut Registry, ctx: &mut Context) {
        use dirs::*;
        let mut places = Vec::new();
        audio_dir().map(|d| places.push(("Audio".to_owned(), d)));
        desktop_dir().map(|d| places.push(("Desktop".to_owned(), d)));
        document_dir().map(|d| places.push(("Documents".to_owned(), d)));
        download_dir().map(|d| places.push(("Downloads".to_owned(), d)));
        picture_dir().map(|d| places.push(("Pictures".to_owned(), d)));
        public_dir().map(|d| places.push(("Public".to_owned(), d)));
        video_dir().map(|d| places.push(("Videos".to_owned(), d)));
        //ListView::count_set(&mut ctx.child("places"), places.len());
        MainView::places_set(&mut ctx.widget(), PlacesWrapper(places));
        self.path_history.push(
            FilesView::path_ref(&ctx.child("files_view"))
                .as_path()
                .to_owned(),
        );
        self.update_path_editor(ctx);
    }

    fn update(&mut self, _: &mut Registry, ctx: &mut Context) {
        if let Some(event) = self.event.take() {
            /*match event {
                Event::ClickOnDirContent(index) => {
                    let file = FilesView::directory_ref(&mut ctx.child("files_view"))
                        .get(index)
                        .cloned()
                        .unwrap();
                    if file.file_type.is_dir() {
                        FilesView::directory_mut(&mut ctx.child("files_view"))
                            .go_deeper(index)
                            .unwrap();
                        self.update_path_history(ctx);
                    }
                }
                Event::Undo => {
                    if self.path_history_cursor != 0 {
                        FilesView::directory_mut(&mut ctx.child("files_view"))
                            .set_path(self.path_history[self.path_history_cursor - 1].clone())
                            .unwrap();
                        self.path_history_cursor -= 1;
                        Button::enabled_set(&mut ctx.child("redo"), true);
                        self.update_path_editor(ctx);
                    }
                    if self.path_history_cursor == 0 {
                        Button::enabled_set(&mut ctx.child("undo"), false);
                    }
                }
                Event::Redo => {
                    if self.path_history_cursor + 1 < self.path_history.len() {
                        self.path_history_cursor += 1;
                        FilesView::directory_mut(&mut ctx.child("files_view"))
                            .set_path(self.path_history[self.path_history_cursor].clone())
                            .unwrap();
                        Button::enabled_set(&mut ctx.child("undo"), true);
                        self.update_path_editor(ctx);
                    }
                    if self.path_history_cursor + 1 >= self.path_history.len() {
                        Button::enabled_set(&mut ctx.child("redo"), false);
                    }
                }
                Event::MoveUp => {
                    FilesView::directory_mut(&mut ctx.child("files_view"))
                        .go_up()
                        .unwrap();
                    self.update_path_history(ctx);
                }
                Event::GoToHome => {
                    if let Some(path) = dirs::home_dir() {
                        debug!("Going to home");
                        if let Err(e) =
                            FilesView::directory_mut(&mut ctx.child("files_view")).set_path(path)
                        {
                            error!("Failed to go to home directory: {}", e);
                        }
                        self.update_path_history(ctx);
                    }
                }
                Event::RequestManualPathChange => {
                    let mut path = PathBuf::from(TextBox::text_clone(&ctx.child("path_editor")));
                    if !path.is_dir() {
                        match crate::files::amend_path(&path) {
                            Ok(Some(p)) => {
                                path = p;
                            }
                            Ok(None) => {}
                            Err(e) => {
                                error!("Error trying to amend a path: {}", e);
                            }
                        }
                    }
                    if path.is_dir() {
                        if let Err(e) =
                            FilesView::directory_mut(&mut ctx.child("files_view")).set_path(&path)
                        {
                            error!("Failed to go to {}: {}", path.display(), e);
                        }
                        self.update_path_history(ctx);
                    } else {
                        self.update_path_editor(ctx);
                    }
                }
            }*/
        }
    }
}

impl MainViewState {
    fn event(&mut self, event: impl Into<Option<Event>>) {
        self.event = event.into();
    }

    fn update_path_history(&mut self, ctx: &mut Context) {
        let current_path = FilesView::path_ref(&ctx.child("files_view"))
            .as_path()
            .to_owned();
        if self.path_history[self.path_history_cursor] != current_path {
            self.path_history.truncate(self.path_history_cursor + 1);
            self.path_history.push(current_path);
            self.path_history_cursor = self.path_history.len() - 1;
            Button::enabled_set(&mut ctx.child("undo"), true);
            self.update_path_editor(ctx);
        }
    }

    fn update_path_editor(&mut self, ctx: &mut Context) {
        let mut path = FilesView::path_ref(&ctx.child("files_view"))
            .to_string_lossy()
            .into_owned();
        #[cfg(unix)]
        if !path.ends_with('/') && (cfg!(not(target_os = "redox")) || path.ends_with(':')) {
            path.push('/');
        }
        #[cfg(windows)]
        if !path.ends_with('\\') {
            path.push('\\');
        }
        TextBox::text_set(&mut ctx.child("path_editor"), path);
    }
}

impl Template for MainView {
    fn template(self, id: Entity, ctx: &mut BuildContext) -> Self {
        self.name("MainView").child(
            Grid::new()
                .columns(Columns::create().push(Column::default()).push("auto"))
                .rows(Rows::create().push("auto").push(Row::default()))
                .child(
                    Grid::new()
                        .columns(Columns::create().push("auto").push(Column::default()))
                        .rows(Rows::create().push(Row::default()))
                        .child(
                            Stack::new()
                                .orientation(Orientation::Horizontal)
                                .attach(Grid::column(0))
                                .child(
                                    Button::new()
                                        .style("button_single_content")
                                        .id("undo")
                                        .icon(material_icons_font::MD_UNDO)
                                        .on_click(move |states, _| {
                                            states.get_mut::<MainViewState>(id).event(Event::Undo);
                                            true
                                        })
                                        .min_width(0)
                                        .enabled(false)
                                        .build(ctx),
                                )
                                .child(
                                    Button::new()
                                        .style("button_single_content")
                                        .id("redo")
                                        .icon(material_icons_font::MD_REDO)
                                        .on_click(move |states, _| {
                                            states.get_mut::<MainViewState>(id).event(Event::Redo);
                                            true
                                        })
                                        .min_width(0)
                                        .enabled(false)
                                        .build(ctx),
                                )
                                .child(
                                    Button::new()
                                        .style("button_single_content")
                                        .icon(material_icons_font::MD_ARROW_UPWARD)
                                        .on_click(move |states, _| {
                                            states
                                                .get_mut::<MainViewState>(id)
                                                .event(Event::MoveUp);
                                            true
                                        })
                                        .min_width(0)
                                        .build(ctx),
                                )
                                .child(
                                    Button::new()
                                        .style("button_single_content")
                                        .icon(material_icons_font::MD_HOME)
                                        .on_click(move |states, _| {
                                            states
                                                .get_mut::<MainViewState>(id)
                                                .event(Event::GoToHome);
                                            true
                                        })
                                        .min_width(0)
                                        .enabled(dirs::home_dir().is_some())
                                        .build(ctx),
                                )
                                .build(ctx),
                        )
                        .child(
                            TextBox::new()
                                .id("path_editor")
                                .attach(Grid::column(1))
                                .on_activate(move |states, _| {
                                    states
                                        .get_mut::<MainViewState>(id)
                                        .event(Event::RequestManualPathChange);
                                })
                                .build(ctx),
                        )
                        .attach(Grid::column(0))
                        .attach(Grid::row(0))
                        .build(ctx),
                )
                .child(
                    FilesView::new()
                        .id("files_view")
                        .attach(Grid::column(1))
                        .attach(Grid::row(1))
                        .path(PathBufWrapper::from(PathBuf::from(".")))
                        .on_selection_changed(move |states, _, change| {
                            states
                                .get_mut::<MainViewState>(id)
                                .event(Event::ClickOnDirContent(change[0]));
                        })
                        .build(ctx),
                )
                .build(ctx),
        )
    }
}
