use std::path::{Path, PathBuf};
use std::time::Instant;

use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkScrollOptions, CommonMarkViewer};

use crate::watcher::{FileWatcher, DEBOUNCE};

pub struct MdViewer {
    path: Option<PathBuf>,
    text: String,
    error: Option<String>,
    cache: CommonMarkCache,
    watcher: Option<FileWatcher>,
    pending_reload: Option<Instant>,
    zoom: f32,
}

impl MdViewer {
    pub fn new(cc: &eframe::CreationContext<'_>, path: Option<PathBuf>) -> Self {
        cc.egui_ctx.set_theme(egui::ThemePreference::System);
        let mut app = Self {
            path: None,
            text: String::new(),
            error: None,
            cache: CommonMarkCache::default(),
            watcher: None,
            pending_reload: None,
            zoom: 1.0,
        };
        if let Some(path) = path {
            app.open_path(path, &cc.egui_ctx);
        }
        app
    }

    fn open_path(&mut self, path: PathBuf, ctx: &egui::Context) {
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!("{name} - MD Viewer")));
        self.path = Some(path.clone());
        self.read_file();
        self.watcher = match FileWatcher::new(&path, ctx.clone()) {
            Ok(watcher) => Some(watcher),
            Err(err) => {
                if self.error.is_none() {
                    self.error = Some(format!("File watcher unavailable: {err:#}"));
                }
                None
            }
        };
        self.pending_reload = None;
    }

    fn read_file(&mut self) {
        let Some(path) = self.path.clone() else {
            return;
        };
        self.text.clear();
        self.error = None;
        match std::fs::read_to_string(&path) {
            Ok(text) => self.text = text,
            Err(err) => self.error = Some(format!("Cannot read {}: {err}", path.display())),
        }
    }

    fn pick_file(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .pick_file()
        {
            self.open_path(path, ctx);
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let (open, reload, zoom_in, zoom_out, zoom_reset) = ctx.input_mut(|input| {
            let open = input.consume_key(egui::Modifiers::CTRL, egui::Key::O);
            let reload = input.consume_key(egui::Modifiers::CTRL, egui::Key::R);
            let zoom_in = input.consume_key(egui::Modifiers::CTRL, egui::Key::Plus)
                || input.consume_key(egui::Modifiers::CTRL, egui::Key::Equals)
                || input.consume_key(
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                    egui::Key::Plus,
                );
            let zoom_out = input.consume_key(egui::Modifiers::CTRL, egui::Key::Minus);
            let zoom_reset = input.consume_key(egui::Modifiers::CTRL, egui::Key::Num0);
            (open, reload, zoom_in, zoom_out, zoom_reset)
        });

        if open {
            self.pick_file(ctx);
        }
        if reload {
            self.read_file();
        }
        if zoom_in {
            self.zoom = (self.zoom * 1.1).min(3.0);
        }
        if zoom_out {
            self.zoom = (self.zoom / 1.1).max(0.5);
        }
        if zoom_reset {
            self.zoom = 1.0;
        }
        if ctx.zoom_factor() != self.zoom {
            ctx.set_zoom_factor(self.zoom);
        }
    }

    fn poll_watcher(&mut self, ctx: &egui::Context) {
        if let Some(watcher) = &self.watcher {
            if watcher.changed() {
                self.pending_reload = Some(Instant::now());
            }
        }
        if let Some(since) = self.pending_reload {
            if since.elapsed() >= DEBOUNCE {
                self.pending_reload = None;
                self.read_file();
            } else {
                ctx.request_repaint_after(DEBOUNCE);
            }
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            let name = self
                .path
                .as_ref()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "No file".to_owned());
            ui.label(egui::RichText::new(name).strong());
            if ui.button("Open...").clicked() {
                self.pick_file(ctx);
            }
            if ui
                .add_enabled(self.path.is_some(), egui::Button::new("Reload"))
                .clicked()
            {
                self.read_file();
            }
        });
        ui.separator();
    }

    fn body(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(error) = self.error.clone() {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(egui::RichText::new(error).color(egui::Color32::RED));
            });
            return;
        }
        if self.path.is_none() {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                if ui.button("Open file...").clicked() {
                    self.pick_file(ctx);
                }
            });
            return;
        }
        let uri_base = self
            .path
            .as_deref()
            .map(image_uri_base)
            .unwrap_or_else(|| "file:///".to_owned());
        CommonMarkViewer::new()
            .default_implicit_uri_scheme(uri_base)
            .show_scrollable(
                egui::Id::unique("mdview-document"),
                ui,
                &mut self.cache,
                &CommonMarkScrollOptions::default(),
                &self.text,
            );
    }
}

impl eframe::App for MdViewer {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.shortcuts(&ctx);
        self.poll_watcher(&ctx);
        egui::Panel::top("mdview-top-bar").show(ui, |ui| {
            self.top_bar(ui, &ctx);
        });
        egui::CentralPanel::default_margins().show(ui, |ui| {
            self.body(ui, &ctx);
        });
    }
}

/// `file://` base URI for images referenced relative to the Markdown file.
fn image_uri_base(path: &Path) -> String {
    let dir = path.parent().unwrap_or_else(|| Path::new(""));
    let dir = dir.to_string_lossy().replace('\\', "/");
    let dir = dir.trim_end_matches('/');
    if dir.starts_with('/') {
        format!("file://{dir}/")
    } else {
        format!("file:///{dir}/")
    }
}
