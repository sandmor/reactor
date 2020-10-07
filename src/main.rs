use lazy_static::lazy_static;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use orbtk::{prelude::*, theme, theming::config::*};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

mod distribute;
mod distribute_layout;
mod file_content_info;
mod files;
mod files_view;
mod items_view;
mod main_view;
use files_view::*;
use main_view::*;

lazy_static! {
    static ref WATCHING_DIRS: Mutex<WatchingDirs> = Mutex::new(WatchingDirs::new());
}

struct WatchingDirs {
    watcher: RecommendedWatcher,
    paths: HashMap<PathBuf, EventAdapter>,
}

impl WatchingDirs {
    pub fn new() -> WatchingDirs {
        let mut watcher: RecommendedWatcher = Watcher::new_immediate(|_res| {
            /*use notify::event::{Event, EventKind};
            let event:Event = match res {
                Ok(event) => event,
                Err(_) => {
                    return;
                },
            };
            /*for (instance_path, _) in {
            }*/
            match event.kind {
                _ => {}
            }*/
        })
        .unwrap();
        watcher
            .configure(notify::Config::PreciseEvents(true))
            .unwrap();
        WatchingDirs {
            watcher,
            paths: HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: impl AsRef<Path>, event_adapter: EventAdapter) {
        self.watcher
            .watch(path.as_ref(), RecursiveMode::NonRecursive)
            .unwrap();
        self.paths.insert(path.as_ref().into(), event_adapter);
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn main() {
    setup_logger().unwrap();
    Application::new()
        .theme(Theme::from_config(
            ThemeConfig::from(theme::LIGHT_THEME_RON)
                .extend(ThemeConfig::from(include_str!("theme.ron"))),
        ))
        .window(|ctx| {
            Window::new()
                .title("Reactor")
                .position((0.0, 0.0))
                .size(720.0, 576.0)
                .borderless(false)
                .resizeable(true)
                .child(MainView::new().build(ctx))
                .build(ctx)
        })
        .run();
}
