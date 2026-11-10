//! The main window. Panel positions are fixed pixel boxes in a 1170 x 752 layout.

use crate::favs::{self, Location};
use crate::engine::{self, Counter, Criteria};
use crate::files::{self, Entry, Filters, TreeNode};
use eframe::egui::{self, Align, Align2, Color32, FontFamily, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, pos2, vec2};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const WIN_W: f32 = 1170.0;
pub const WIN_H: f32 = 752.0;
/// Layout boxes are measured with the window frame included: content starts at (8, 31).
const OX: f32 = 8.0;
const OY: f32 = 31.0;

const BG: Color32 = Color32::from_rgb(0xF0, 0xF0, 0xF0);
const BANNER: Color32 = Color32::from_rgb(0x4D, 0x60, 0x82);
const ROW: Color32 = Color32::from_rgb(0xFF, 0xF8, 0xF0);
const NOTICE: Color32 = Color32::from_rgb(0xDE, 0xE7, 0xF9);
const RENAME_FACE: Color32 = Color32::from_rgb(0xC8, 0xD3, 0xF1);
const YELLOW: Color32 = Color32::from_rgb(0xFF, 0xD6, 0x00);
const EDGE: Color32 = Color32::from_rgb(0x7A, 0x7A, 0x7A);
const TEXT: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x1E);
const GREY_TEXT: Color32 = Color32::from_rgb(0x8C, 0x8C, 0x8C);
const SEL: Color32 = Color32::from_rgb(0xCC, 0xE8, 0xFF);
const SEL_EDGE: Color32 = Color32::from_rgb(0x99, 0xD1, 0xFF);
const BTN: Color32 = Color32::from_rgb(0xE1, 0xE1, 0xE1);
const BTN_EDGE: Color32 = Color32::from_rgb(0xAD, 0xAD, 0xAD);
const HOVER: Color32 = Color32::from_rgb(0xE5, 0xF1, 0xFB);
const HOVER_EDGE: Color32 = Color32::from_rgb(0x00, 0x78, 0xD7);
const FOLDER: Color32 = Color32::from_rgb(0xF8, 0xC8, 0x51);
const INVALID: Color32 = Color32::from_rgb(0xD0, 0x10, 0x10);

fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_min_max(pos2(x0 - OX, y0 - OY), pos2(x1 - OX, y1 - OY))
}

fn font(size: f32) -> FontId {
    FontId::proportional(size)
}
fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

/// A criteria box: its place on screen and its place in the dialog template.
#[derive(Clone, Copy)]
struct Pan {
    px: Rect,
    dx: f32,
    dy: f32,
    sx: f32,
    sy: f32,
}

impl Pan {
    fn new(px: Rect, d: [f32; 4]) -> Self {
        Pan { px, dx: d[0], dy: d[1], sx: px.width() / d[2], sy: px.height() / d[3] }
    }
    fn at(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(
            pos2((self.px.min.x + (x - self.dx) * self.sx).round(), (self.px.min.y + (y - self.dy) * self.sy).round()),
            vec2((w * self.sx).round(), (h * self.sy).round()),
        )
    }
}

#[derive(Clone)]
enum Modal {
    Msg(String, String),
    Preview,
    Order(Vec<u8>),
    Translations,
    About,
}

#[derive(Clone, Copy, PartialEq)]
enum Act {
    New,
    Open,
    Save,
    SaveAs,
    Exit,
    SelectAll,
    DeselectAll,
    Invert,
    Refresh,
    ResetAll,
    Revert,
    Preview,
    Rename,
    Undo,
    Order,
    Translations,
    NotAvailable(&'static str),
    About,
    Up,
    Back,
    Forward,
    Browse,
    ClearList,
    ShowTree,
}

pub struct App {
    crit: Criteria,
    saved: (Criteria, Filters, Location),
    shown_for: Option<(Criteria, Vec<bool>)>,
    filters: Filters,
    listed_with: Filters,
    mask_edit: String,
    loc: Location,
    cur_dir: PathBuf,
    path_text: String,
    back: Vec<PathBuf>,
    fwd: Vec<PathBuf>,
    items: Vec<Entry>,
    tree: Vec<TreeNode>,
    tree_scroll: bool,
    show_tree: bool,
    anchor: Option<usize>,
    // List sort: column index and ascending; None keeps the folder listing order.
    sort: Option<(usize, bool)>,
    auto_refresh: bool,
    modal: Option<Modal>,
    undo: Vec<(PathBuf, PathBuf)>,
    favourite: Option<PathBuf>,
    pending: Vec<Act>,
    status_note: String,
    shot: Option<PathBuf>,
    frames: u32,
    shot_sent: bool,
    badge: egui::TextureHandle,
}

fn settings_file() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("Bplus").join("settings.txt"))
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
    let fdir = Path::new(&windir).join("Fonts");
    let mut bold_family = Vec::new();
    if let Ok(b) = std::fs::read(fdir.join("segoeui.ttf")) {
        fonts.font_data.insert("segoeui".into(), Arc::new(egui::FontData::from_owned(b)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "segoeui".into());
    }
    if let Ok(b) = std::fs::read(fdir.join("segoeuib.ttf")) {
        fonts.font_data.insert("segoeuib".into(), Arc::new(egui::FontData::from_owned(b)));
        bold_family.push("segoeuib".to_string());
    }
    if let Ok(b) = std::fs::read(fdir.join("seguisym.ttf")) {
        fonts.font_data.insert("seguisym".into(), Arc::new(egui::FontData::from_owned(b)));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().push("seguisym".into());
    }
    let prop = fonts.families[&FontFamily::Proportional].clone();
    bold_family.extend(prop);
    fonts.families.insert(FontFamily::Name("bold".into()), bold_family);
    ctx.set_fonts(fonts);
}

fn set_style(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.all_styles_mut(|s| {
        s.visuals = egui::Visuals::light();
        s.text_styles.insert(egui::TextStyle::Body, font(12.0));
        s.text_styles.insert(egui::TextStyle::Button, font(12.0));
        s.text_styles.insert(egui::TextStyle::Small, font(11.0));
        s.text_styles.insert(egui::TextStyle::Heading, bold(14.0));
        s.spacing.button_padding = vec2(6.0, 2.0);
        s.spacing.item_spacing = vec2(6.0, 3.0);
        s.spacing.interact_size.y = 20.0;
        let v = &mut s.visuals;
        v.panel_fill = BG;
        v.window_fill = BG;
        v.extreme_bg_color = Color32::WHITE;
        v.text_edit_bg_color = Some(Color32::WHITE);
        v.override_text_color = Some(TEXT);
        v.selection.bg_fill = Color32::from_rgb(0x99, 0xC9, 0xEF);
        v.selection.stroke = Stroke::new(1.0, TEXT);
        v.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
        v.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
        v.widgets.inactive.weak_bg_fill = Color32::WHITE;
        v.widgets.inactive.bg_fill = Color32::WHITE;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, EDGE);
        v.widgets.hovered.weak_bg_fill = Color32::WHITE;
        v.widgets.hovered.bg_fill = Color32::WHITE;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x17, 0x17, 0x17));
        v.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
        v.widgets.hovered.expansion = 0.0;
        v.widgets.active.weak_bg_fill = Color32::WHITE;
        v.widgets.active.bg_stroke = Stroke::new(1.0, HOVER_EDGE);
        v.widgets.active.corner_radius = egui::CornerRadius::ZERO;
        v.widgets.active.expansion = 0.0;
        v.widgets.open.weak_bg_fill = Color32::WHITE;
        v.widgets.open.bg_stroke = Stroke::new(1.0, HOVER_EDGE);
        v.widgets.open.corner_radius = egui::CornerRadius::ZERO;
        v.window_corner_radius = egui::CornerRadius::ZERO;
        v.menu_corner_radius = egui::CornerRadius::ZERO;
    });
}

impl App {
    pub fn new(ctx: &egui::Context, start: Option<PathBuf>, shot: Option<PathBuf>) -> Self {
        install_fonts(ctx);
        set_style(ctx);
        let last = settings_file()
            .and_then(|f| std::fs::read_to_string(f).ok())
            .and_then(|t| t.lines().find_map(|l| l.strip_prefix("last_path=").map(PathBuf::from)))
            .filter(|p| p.is_dir());
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("C:\\"));
        let cur = start.or(last).unwrap_or(home);
        let crit = Criteria::default();
        let filters = Filters::default();
        let loc = Location { on: false, path: String::new(), copy_not_move: true, keep_structure: false };
        let mut app = App {
            saved: (crit.clone(), filters.clone(), loc.clone()),
            crit,
            shown_for: None,
            listed_with: filters.clone(),
            mask_edit: filters.mask.clone(),
            filters,
            loc,
            cur_dir: cur.clone(),
            path_text: cur.to_string_lossy().into_owned(),
            back: Vec::new(),
            fwd: Vec::new(),
            items: Vec::new(),
            tree: files::drive_roots(),
            tree_scroll: true,
            show_tree: true,
            anchor: None,
            sort: None,
            auto_refresh: true,
            modal: None,
            undo: Vec::new(),
            favourite: None,
            pending: Vec::new(),
            status_note: String::new(),
            shot,
            frames: 0,
            shot_sent: false,
            badge: ctx.load_texture(
                "bplus-mark",
                egui::ColorImage::from_rgba_unmultiplied([48, 48], &crate::icon::rgba(48)),
                egui::TextureOptions::LINEAR,
            ),
        };
        app.go_to(cur, false);
        app
    }

    // ---------------- folders and list ----------------

    fn go_to(&mut self, dir: PathBuf, remember: bool) {
        if !dir.is_dir() {
            self.modal = Some(Modal::Msg("Bplus Rename".into(), format!("The folder '{}' could not be found.", dir.display())));
            return;
        }
        if remember && dir != self.cur_dir {
            self.back.push(self.cur_dir.clone());
            self.fwd.clear();
        }
        self.cur_dir = dir.clone();
        self.path_text = dir.to_string_lossy().into_owned();
        for t in &mut self.tree {
            t.reveal(&dir);
        }
        self.tree_scroll = true;
        self.refresh();
        if self.shot.is_none() {
            if let Some(f) = settings_file() {
                let _ = std::fs::create_dir_all(f.parent().unwrap());
                let _ = std::fs::write(f, format!("last_path={}\n", dir.display()));
            }
        }
    }

    fn refresh(&mut self) {
        self.items = files::list_folder(&self.cur_dir, &self.filters);
        self.listed_with = self.filters.clone();
        self.anchor = None;
        self.shown_for = None;
        self.apply_sort();
    }

    // Folders stay first; ties fall back to the name so the order is stable.
    fn apply_sort(&mut self) {
        let Some((col, asc)) = self.sort else { return };
        let lower = |s: &str| s.to_lowercase();
        self.items.sort_by(|a, b| {
            let o = match col {
                1 => lower(&a.new_name).cmp(&lower(&b.new_name)),
                2 => lower(&a.sub_dir).cmp(&lower(&b.sub_dir)),
                3 => type_label(a).cmp(&type_label(b)),
                4 => a.size.cmp(&b.size),
                5 => a.info.created.cmp(&b.info.created),
                6 => a.info.modified.cmp(&b.info.modified),
                7 => a.info.accessed.cmp(&b.info.accessed),
                8 => a.new_name.chars().count().cmp(&b.new_name.chars().count()),
                _ => std::cmp::Ordering::Equal,
            }
            .then_with(|| lower(&a.name).cmp(&lower(&b.name)));
            b.info.is_dir.cmp(&a.info.is_dir).then(if asc { o } else { o.reverse() })
        });
        self.anchor = None;
        // Numbering follows list order, so the New Name column must be rebuilt.
        self.shown_for = None;
    }

    fn recompute(&mut self) {
        let mut counter = Counter::new();
        let active = self.crit.is_active();
        for e in &mut self.items {
            e.new_name = if e.selected && active { self.crit.new_name(&e.info, &mut counter) } else { e.name.clone() };
        }
        self.shown_for = Some((self.crit.clone(), self.items.iter().map(|e| e.selected).collect()));
    }

    fn selected_count(&self) -> usize {
        self.items.iter().filter(|e| e.selected).count()
    }

    // ---------------- renaming ----------------

    fn do_rename(&mut self) {
        self.recompute();
        let jobs: Vec<(usize, PathBuf, PathBuf)> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, e)| e.selected && e.new_name != e.name)
            .map(|(i, e)| {
                let parent = e.info.path.parent().unwrap_or(Path::new("")).to_path_buf();
                (i, e.info.path.clone(), parent.join(&e.new_name))
            })
            .collect();
        let processed = self.selected_count();
        let loc_dir = if self.loc.on && !self.loc.path.trim().is_empty() { Some(PathBuf::from(self.loc.path.trim())) } else { None };
        if jobs.is_empty() && (loc_dir.is_none() || processed == 0) {
            self.modal = Some(Modal::Msg("Bplus Rename".into(), "There are no selected items that need renaming.".into()));
            return;
        }
        if jobs.iter().any(|(i, _, _)| engine::has_invalid(&self.items[*i].new_name)) {
            self.modal = Some(Modal::Msg(
                "Bplus Rename".into(),
                format!(
                    "One or more filenames contains an invalid character. The following characters are invalid in Windows file names: {}",
                    engine::INVALID_CHARS
                ),
            ));
            return;
        }
        // Two names in the batch may not end up the same.
        let mut seen = std::collections::HashSet::new();
        for (_, _, to) in &jobs {
            if !seen.insert(to.to_string_lossy().to_lowercase()) {
                self.modal = Some(Modal::Msg(
                    "Bplus Rename".into(),
                    format!("Item '{}' already exists. To prevent duplicate names, enable the option 'Prevent Duplicates'.", to.display()),
                ));
                return;
            }
        }
        let sources: std::collections::HashSet<String> = jobs.iter().map(|(_, f, _)| f.to_string_lossy().to_lowercase()).collect();
        let mut errors = Vec::new();
        let mut done: Vec<(PathBuf, PathBuf)> = Vec::new();
        let start = std::time::Instant::now();

        if let Some(dest) = &loc_dir {
            // Copy/Move to Location (13): the renamed items go to another folder.
            if let Err(e) = std::fs::create_dir_all(dest) {
                errors.push(format!("{}: {e}", dest.display()));
            }
            for e in self.items.iter().filter(|e| e.selected && !e.info.is_dir) {
                let mut target = dest.clone();
                if self.loc.keep_structure && !e.sub_dir.is_empty() {
                    target = target.join(&e.sub_dir);
                    let _ = std::fs::create_dir_all(&target);
                }
                let target = target.join(&e.new_name);
                if target.exists() {
                    errors.push(format!("Skipped. File name already exists. {}", target.display()));
                    continue;
                }
                let res = if self.loc.copy_not_move {
                    std::fs::copy(&e.info.path, &target).map(|_| ())
                } else {
                    std::fs::rename(&e.info.path, &target)
                };
                match res {
                    Ok(()) => {
                        if !self.loc.copy_not_move {
                            done.push((target, e.info.path.clone()));
                        }
                    }
                    Err(err) => errors.push(format!("Item '{}' could not be renamed.\nError: {err}", e.info.path.display())),
                }
            }
        } else {
            // Deepest items first, so renaming a folder never moves an item still waiting.
            let mut jobs = jobs;
            jobs.sort_by_key(|(_, f, _)| std::cmp::Reverse(f.components().count()));
            let mut stage2 = Vec::new();
            for (_, from, to) in &jobs {
                let to_lower = to.to_string_lossy().to_lowercase();
                let from_lower = from.to_string_lossy().to_lowercase();
                let clash = to.exists() && to_lower != from_lower;
                if clash && !sources.contains(&to_lower) {
                    errors.push(format!(
                        "Item '{}' already exists. To prevent duplicate names, enable the option 'Prevent Duplicates'.",
                        to.display()
                    ));
                    continue;
                }
                if clash || to_lower == from_lower {
                    // Swap through a temporary name (also needed for case-only changes).
                    let tmp = from.with_file_name(format!("~bplus_{}_{}", std::process::id(), stage2.len()));
                    match std::fs::rename(from, &tmp) {
                        Ok(()) => stage2.push((tmp, to.clone(), from.clone())),
                        Err(err) => errors.push(format!("Item '{}' could not be renamed.\nError: {err}", from.display())),
                    }
                    continue;
                }
                match std::fs::rename(from, to) {
                    Ok(()) => done.push((to.clone(), from.clone())),
                    Err(err) => errors.push(format!("Item '{}' could not be renamed.\nError: {err}", from.display())),
                }
            }
            for (tmp, to, orig) in stage2 {
                match std::fs::rename(&tmp, &to) {
                    Ok(()) => done.push((to, orig)),
                    Err(err) => {
                        let _ = std::fs::rename(&tmp, &orig);
                        errors.push(format!("Item '{}' could not be renamed.\nError: {err}", orig.display()));
                    }
                }
            }
        }
        let renamed = done.len();
        if !done.is_empty() {
            self.undo = done;
        }
        let mut msg = format!(
            "{processed} item(s) processed, {renamed} item(s) renamed. {} item(s) required no name change.",
            processed.saturating_sub(renamed + errors.len())
        );
        if !errors.is_empty() {
            msg.push_str(&format!(
                "\n\n{} error(s).\n\n{}",
                errors.len(),
                errors.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
            ));
        }
        msg.push_str(&format!("\n\nElapsed time: {:.2} s", start.elapsed().as_secs_f32()));
        self.status_note = format!("{renamed} item(s) renamed.");
        self.refresh();
        self.modal = Some(Modal::Msg("Bplus Rename".into(), msg));
    }

    fn do_undo(&mut self) {
        if self.undo.is_empty() {
            self.modal = Some(Modal::Msg("Bplus Rename".into(), "There is nothing to undo.".into()));
            return;
        }
        let mut ok = 0;
        let mut errors = Vec::new();
        for (now, was) in self.undo.iter().rev() {
            match std::fs::rename(now, was) {
                Ok(()) => ok += 1,
                Err(e) => errors.push(format!("{}: {e}", now.display())),
            }
        }
        self.undo.clear();
        self.refresh();
        let mut msg = format!("{ok} item(s) successfully processed.");
        if !errors.is_empty() {
            msg.push_str(&format!("\n\n{} error(s).\n{}", errors.len(), errors.join("\n")));
        }
        self.modal = Some(Modal::Msg("Undo Rename".into(), msg));
    }

    fn run(&mut self, ctx: &egui::Context, a: Act) {
        match a {
            Act::New => {
                self.crit = Criteria::default();
                self.filters = Filters::default();
                self.mask_edit = self.filters.mask.clone();
                self.loc = Location { copy_not_move: true, ..Default::default() };
                self.favourite = None;
                self.saved = (self.crit.clone(), self.filters.clone(), self.loc.clone());
            }
            Act::Open => {
                if let Some(p) = rfd::FileDialog::new().add_filter("Favourites", &["bplus"]).add_filter("All files", &["*"]).pick_file() {
                    if let Ok(t) = std::fs::read_to_string(&p) {
                        let mut c = Criteria::default();
                        let mut f = Filters::default();
                        let mut l = Location { copy_not_move: true, ..Default::default() };
                        favs::load(&t, &mut c, &mut f, &mut l);
                        self.crit = c;
                        self.filters = f;
                        self.mask_edit = self.filters.mask.clone();
                        self.loc = l;
                        self.saved = (self.crit.clone(), self.filters.clone(), self.loc.clone());
                        self.favourite = Some(p);
                    }
                }
            }
            Act::Save | Act::SaveAs => {
                let path = if a == Act::Save { self.favourite.clone() } else { None };
                let path = path.or_else(|| {
                    rfd::FileDialog::new().add_filter("Favourites", &["bplus"]).set_file_name("favourite.bplus").save_file()
                });
                if let Some(p) = path {
                    let text = favs::save(&self.crit, &self.filters, &self.loc);
                    match std::fs::write(&p, text) {
                        Ok(()) => {
                            self.saved = (self.crit.clone(), self.filters.clone(), self.loc.clone());
                            self.favourite = Some(p);
                        }
                        Err(e) => self.modal = Some(Modal::Msg("Bplus Rename".into(), format!("Could not save: {e}"))),
                    }
                }
            }
            Act::Exit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Act::SelectAll => self.items.iter_mut().for_each(|e| e.selected = true),
            Act::DeselectAll => self.items.iter_mut().for_each(|e| e.selected = false),
            Act::Invert => self.items.iter_mut().for_each(|e| e.selected = !e.selected),
            Act::Refresh => {
                self.filters.mask = self.mask_edit.clone();
                self.refresh();
            }
            Act::ResetAll => {
                let order = self.crit.order.clone();
                self.crit = Criteria { order, ..Criteria::default() };
            }
            Act::Revert => {
                self.crit = self.saved.0.clone();
                self.filters = self.saved.1.clone();
                self.mask_edit = self.filters.mask.clone();
                self.loc = self.saved.2.clone();
            }
            Act::Preview => {
                self.recompute();
                self.modal = Some(Modal::Preview);
            }
            Act::Rename => self.do_rename(),
            Act::Undo => self.do_undo(),
            Act::Order => self.modal = Some(Modal::Order(self.crit.order.clone())),
            Act::Translations => self.modal = Some(Modal::Translations),
            Act::NotAvailable(what) => {
                self.modal = Some(Modal::Msg("Bplus Rename".into(), format!("{what} is not available in this version.")))
            }
            Act::About => self.modal = Some(Modal::About),
            Act::Up => {
                if let Some(p) = self.cur_dir.parent().map(|p| p.to_path_buf()) {
                    self.go_to(p, true);
                }
            }
            Act::Back => {
                if let Some(p) = self.back.pop() {
                    self.fwd.push(self.cur_dir.clone());
                    self.go_to(p, false);
                }
            }
            Act::Forward => {
                if let Some(p) = self.fwd.pop() {
                    self.back.push(self.cur_dir.clone());
                    self.go_to(p, false);
                }
            }
            Act::Browse => {
                if let Some(p) = rfd::FileDialog::new().set_directory(&self.cur_dir).pick_folder() {
                    self.go_to(p, true);
                }
            }
            Act::ClearList => {
                self.items.clear();
                self.anchor = None;
            }
            Act::ShowTree => self.show_tree = !self.show_tree,
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        use egui::{Key, KeyboardShortcut, Modifiers};
        let typing = ctx.egui_wants_keyboard_input();
        let list = [
            (Modifiers::COMMAND, Key::O, Act::Open, true),
            (Modifiers::COMMAND, Key::S, Act::Save, true),
            (Modifiers::COMMAND, Key::N, Act::New, true),
            (Modifiers::COMMAND, Key::A, Act::SelectAll, false),
            (Modifiers::COMMAND, Key::D, Act::DeselectAll, false),
            (Modifiers::COMMAND, Key::I, Act::Invert, false),
            (Modifiers::NONE, Key::F5, Act::Refresh, true),
            (Modifiers::NONE, Key::F11, Act::ShowTree, true),
            (Modifiers::NONE, Key::F7, Act::ClearList, true),
            (Modifiers::COMMAND, Key::T, Act::ResetAll, true),
            (Modifiers::COMMAND, Key::E, Act::Revert, true),
            (Modifiers::COMMAND, Key::P, Act::Preview, true),
            (Modifiers::COMMAND, Key::R, Act::Rename, true),
            (Modifiers::COMMAND, Key::Z, Act::Undo, false),
            (Modifiers::COMMAND, Key::F8, Act::Order, true),
            (Modifiers::COMMAND, Key::F6, Act::Translations, true),
        ];
        for (m, k, a, always) in list {
            if (always || !typing) && ctx.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(m, k))) {
                self.pending.push(a);
            }
        }
    }
}

// ---------------- small painted controls ----------------

fn text_at(ui: &egui::Ui, rect: Rect, s: &str, f: FontId, c: Color32, align: Align2) {
    let pos = match align {
        Align2::RIGHT_CENTER => pos2(rect.max.x, rect.center().y),
        Align2::CENTER_CENTER => rect.center(),
        _ => pos2(rect.min.x, rect.center().y),
    };
    ui.painter().text(pos, align, s, f, c);
}

fn label(ui: &egui::Ui, rect: Rect, s: &str) {
    text_at(ui, rect, s, font(12.0), TEXT, Align2::LEFT_CENTER);
}

fn edit(ui: &mut egui::Ui, rect: Rect, s: &mut String, id: &str, enabled: bool) -> bool {
    let te = egui::TextEdit::singleline(s)
        .id_salt(id)
        .font(font(12.0))
        .margin(egui::Margin::symmetric(3, 1))
        .vertical_align(Align::Center)
        .min_size(rect.size())
        .desired_width(rect.width());
    ui.add_enabled_ui(enabled, |ui| ui.put(rect, te)).inner.changed()
}

/// An edit box with a drop-down chevron and grip on the right.
fn edit_combo(ui: &mut egui::Ui, rect: Rect, s: &mut String, id: &str) -> egui::Response {
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, Color32::WHITE);
    p.rect_stroke(rect, 0.0, Stroke::new(1.0, EDGE), StrokeKind::Inside);
    let inner = Rect::from_min_max(rect.min, pos2(rect.max.x - 24.0, rect.max.y));
    let te = egui::TextEdit::singleline(s)
        .id_salt(id)
        .font(font(12.0))
        .frame(egui::Frame::NONE)
        .margin(egui::Margin::symmetric(4, 1))
        .vertical_align(Align::Center)
        .min_size(inner.size())
        .desired_width(inner.width());
    let resp = ui.put(inner, te);
    chevron(&p, pos2(rect.max.x - 17.0, rect.center().y), 3.5, TEXT);
    grip(&p, pos2(rect.max.x - 7.0, rect.center().y));
    resp
}

fn chevron(p: &egui::Painter, c: Pos2, s: f32, col: Color32) {
    let st = Stroke::new(1.0, col);
    p.line_segment([pos2(c.x - s, c.y - s * 0.5), pos2(c.x, c.y + s * 0.5)], st);
    p.line_segment([pos2(c.x, c.y + s * 0.5), pos2(c.x + s, c.y - s * 0.5)], st);
}

fn grip(p: &egui::Painter, c: Pos2) {
    for dy in [-3.0, 0.0, 3.0] {
        for dx in [-1.5, 1.5] {
            p.rect_filled(Rect::from_center_size(pos2(c.x + dx, c.y + dy), vec2(1.5, 1.5)), 0.0, TEXT);
        }
    }
}

/// A list icon: two bullets and lines, as in front of the "Match", "With" and "Excep." boxes.
fn list_icon(p: &egui::Painter, c: Pos2) {
    for dy in [-3.0, 3.0] {
        p.rect_filled(Rect::from_center_size(pos2(c.x - 4.0, c.y + dy), vec2(2.0, 2.0)), 0.0, TEXT);
        p.line_segment([pos2(c.x - 1.0, c.y + dy), pos2(c.x + 6.0, c.y + dy)], Stroke::new(1.2, TEXT));
    }
}

fn combo(ui: &mut egui::Ui, rect: Rect, id: &str, v: &mut usize, items: &[&str]) -> bool {
    let mut changed = false;
    let cur = items.get(*v).copied().unwrap_or("").to_string();
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().interact_size.y = rect.height();
        ui.spacing_mut().button_padding = vec2(4.0, 1.0);
        egui::ComboBox::from_id_salt(id)
            .width(rect.width())
            .height(320.0)
            .selected_text(cur)
            .icon(|ui, r, _v, _open| chevron(ui.painter(), pos2(r.center().x, r.center().y), 3.5, TEXT))
            .show_ui(ui, |ui| {
            for (i, it) in items.iter().enumerate() {
                if ui.selectable_value(v, i, *it).changed() {
                    changed = true;
                }
            }
        });
    });
    changed
}

fn check(ui: &mut egui::Ui, rect: Rect, v: &mut bool, text: &str, id: &str) -> bool {
    let resp = ui.interact(rect, ui.id().with(id), Sense::click());
    if resp.clicked() {
        *v = !*v;
    }
    let p = ui.painter();
    let b = Rect::from_min_size(pos2(rect.min.x, (rect.center().y - 6.5).round()), vec2(13.0, 13.0));
    p.rect_filled(b, 0.0, Color32::WHITE);
    let edge = if resp.hovered() { HOVER_EDGE } else { Color32::from_rgb(0x33, 0x33, 0x33) };
    p.rect_stroke(b, 0.0, Stroke::new(1.0, edge), StrokeKind::Inside);
    if *v {
        let st = Stroke::new(1.6, TEXT);
        p.line_segment([pos2(b.min.x + 3.0, b.center().y), pos2(b.min.x + 5.5, b.max.y - 3.5)], st);
        p.line_segment([pos2(b.min.x + 5.5, b.max.y - 3.5), pos2(b.max.x - 2.5, b.min.y + 3.0)], st);
    }
    if !text.is_empty() {
        label(ui, Rect::from_min_max(pos2(b.max.x + 4.0, rect.min.y), rect.max), text);
    }
    resp.clicked()
}

fn button(ui: &mut egui::Ui, rect: Rect, text: &str, id: &str) -> bool {
    let resp = ui.interact(rect, ui.id().with(id), Sense::click());
    let p = ui.painter();
    let (fill, edge) = if resp.hovered() { (HOVER, HOVER_EDGE) } else { (BTN, BTN_EDGE) };
    p.rect_filled(rect, 0.0, fill);
    p.rect_stroke(rect, 0.0, Stroke::new(1.0, edge), StrokeKind::Inside);
    text_at(ui, rect, text, font(12.0), TEXT, Align2::CENTER_CENTER);
    resp.clicked()
}

fn small_r(ui: &mut egui::Ui, rect: Rect, id: &str) -> bool {
    let resp = ui.interact(rect, ui.id().with(id), Sense::click());
    let p = ui.painter();
    p.rect_filled(rect, 2.0, if resp.hovered() { HOVER } else { Color32::from_rgb(0xFB, 0xFB, 0xFB) });
    p.rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(0xB5, 0xB5, 0xB5)), StrokeKind::Inside);
    text_at(ui, rect, "R", font(11.5), TEXT, Align2::CENTER_CENTER);
    resp.clicked()
}

fn spin(ui: &mut egui::Ui, rect: Rect, s: &mut String, id: &str, min: i64, enabled: bool) -> bool {
    let arrows_w = 14.0;
    let er = Rect::from_min_max(rect.min, pos2(rect.max.x - arrows_w, rect.max.y));
    let mut changed = edit(ui, er, s, id, enabled);
    let up = Rect::from_min_max(pos2(rect.max.x - arrows_w, rect.min.y), pos2(rect.max.x, rect.center().y));
    let dn = Rect::from_min_max(pos2(rect.max.x - arrows_w, rect.center().y), rect.max);
    for (k, rr) in [(1i64, up), (-1, dn)] {
        let resp = ui.interact(rr, ui.id().with((id, k)), if enabled { Sense::click() } else { Sense::hover() });
        let p = ui.painter();
        p.rect_filled(rr, 0.0, if resp.hovered() && enabled { HOVER } else { Color32::from_rgb(0xF4, 0xF4, 0xF4) });
        p.rect_stroke(rr, 0.0, Stroke::new(1.0, Color32::from_rgb(0xAC, 0xAC, 0xAC)), StrokeKind::Inside);
        let c = rr.center();
        let col = if enabled { TEXT } else { GREY_TEXT };
        let tri = if k > 0 {
            vec![pos2(c.x - 3.0, c.y + 1.5), pos2(c.x + 3.0, c.y + 1.5), pos2(c.x, c.y - 1.5)]
        } else {
            vec![pos2(c.x - 3.0, c.y - 1.5), pos2(c.x + 3.0, c.y - 1.5), pos2(c.x, c.y + 1.5)]
        };
        p.add(egui::Shape::convex_polygon(tri, col, Stroke::NONE));
        if resp.clicked() {
            let v = s.trim().parse::<i64>().unwrap_or(0) + k;
            *s = v.max(min).to_string();
            changed = true;
        }
    }
    changed
}

fn group(ui: &mut egui::Ui, pan: &Pan, title: &str) {
    let p = ui.painter();
    let top = pan.px.min.y + 7.0;
    let fr = Rect::from_min_max(pos2(pan.px.min.x, top), pan.px.max);
    p.rect_stroke(fr.translate(vec2(1.0, 1.0)), 0.0, Stroke::new(1.0, Color32::WHITE), StrokeKind::Inside);
    p.rect_stroke(fr, 0.0, Stroke::new(1.0, Color32::from_rgb(0xD5, 0xD5, 0xD5)), StrokeKind::Inside);
    let galley = p.layout_no_wrap(title.to_string(), bold(12.0), TEXT);
    let tr = Rect::from_min_size(pos2(pan.px.min.x + 6.0, top - galley.size().y / 2.0), galley.size() + vec2(6.0, 0.0));
    p.rect_filled(tr, 0.0, BG);
    p.galley(pos2(tr.min.x + 3.0, tr.min.y), galley, TEXT);
}

fn folder_icon(p: &egui::Painter, at: Pos2) {
    let body = Rect::from_min_size(pos2(at.x, at.y + 2.0), vec2(16.0, 11.0));
    p.rect_filled(Rect::from_min_size(at, vec2(7.0, 3.0)), 1.0, Color32::from_rgb(0xE8, 0xB0, 0x30));
    p.rect_filled(body, 1.0, FOLDER);
    p.line_segment(
        [pos2(body.min.x, body.min.y + 1.0), pos2(body.max.x, body.min.y + 1.0)],
        Stroke::new(1.0, Color32::from_rgb(0xFF, 0xE0, 0x8A)),
    );
}

fn file_icon(p: &egui::Painter, at: Pos2) {
    let body = Rect::from_min_size(pos2(at.x + 2.0, at.y), vec2(12.0, 15.0));
    p.rect_filled(body, 0.0, Color32::WHITE);
    p.rect_stroke(body, 0.0, Stroke::new(1.0, Color32::from_rgb(0x8A, 0x8A, 0x8A)), StrokeKind::Inside);
    for i in 0..3 {
        let y = body.min.y + 5.0 + i as f32 * 3.0;
        p.line_segment([pos2(body.min.x + 3.0, y), pos2(body.max.x - 3.0, y)], Stroke::new(1.0, Color32::from_rgb(0xB0, 0xB8, 0xC8)));
    }
}

fn refresh_icon(p: &egui::Painter, c: Pos2) {
    let col = Color32::from_rgb(0x1E, 0x88, 0xE5);
    p.circle_stroke(c, 4.5, Stroke::new(1.8, col));
    p.rect_filled(Rect::from_min_size(pos2(c.x + 1.0, c.y - 6.0), vec2(4.0, 4.0)), 0.0, col);
}

fn down_triangle(p: &egui::Painter, t: Pos2) {
    p.add(egui::Shape::convex_polygon(
        vec![pos2(t.x - 4.0, t.y - 2.0), pos2(t.x + 4.0, t.y - 2.0), pos2(t.x, t.y + 2.5)],
        TEXT,
        Stroke::NONE,
    ));
}

fn crit_name(id: u8) -> &'static str {
    match id {
        1 => "RegEx (1)",
        2 => "Name (2)",
        3 => "Replace (3)",
        4 => "Case (4)",
        5 => "Remove (5)",
        6 => "Move/Copy Parts (6)",
        7 => "Add (7)",
        8 => "Auto Date (8)",
        9 => "Append Folder Name (9)",
        10 => "Numbering (10)",
        _ => "Extension (11)",
    }
}

// ---------------- the window ----------------

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.shortcuts(&ctx);
        let full = ui.max_rect();
        ui.painter().rect_filled(full, 0.0, BG);

        // Recalculate the New Name column when the criteria (with Auto Refresh on) or the selection change.
        let sel: Vec<bool> = self.items.iter().map(|e| e.selected).collect();
        let (crit_changed, sel_changed) = match &self.shown_for {
            Some((c, s)) => (c != &self.crit, s != &sel),
            None => (true, true),
        };
        if sel_changed || (crit_changed && self.auto_refresh) {
            self.recompute();
        }

        self.menu_bar(ui);
        self.banner_and_nav(ui);
        self.tree_and_list(ui);
        self.criteria_panels(ui);
        self.bottom(ui);
        self.modals(&ctx);

        if self.filters != self.listed_with {
            self.refresh();
        }
        let acts: Vec<Act> = std::mem::take(&mut self.pending);
        for a in acts {
            self.run(&ctx, a);
        }
        self.screenshot(&ctx);
    }
}

impl App {
    fn screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.shot.clone() else { return };
        self.frames += 1;
        ctx.request_repaint();
        if self.frames >= 120 && !self.shot_sent {
            self.shot_sent = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(file) = std::fs::File::create(&path) {
                let w = std::io::BufWriter::new(file);
                let mut enc = png::Encoder::new(w, img.size[0] as u32, img.size[1] as u32);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                if let Ok(mut wr) = enc.write_header() {
                    let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
                    let _ = wr.write_image_data(&bytes);
                }
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let bar = Rect::from_min_size(ui.max_rect().min, vec2(ui.max_rect().width(), 20.0));
        ui.painter().rect_filled(bar, 0.0, Color32::WHITE);
        let mut acts = Vec::new();
        ui.scope_builder(egui::UiBuilder::new().max_rect(bar.translate(vec2(6.0, 0.0))), |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let mut item = |ui: &mut egui::Ui, text: &str, a: Act| {
                    if ui.button(text).clicked() {
                        acts.push(a);
                    }
                };
                ui.menu_button("File", |ui| {
                    item(ui, "New\tCtrl+N", Act::New);
                    item(ui, "Open...\tCtrl+O", Act::Open);
                    item(ui, "Save\tCtrl+S", Act::Save);
                    item(ui, "Save As...", Act::SaveAs);
                    ui.separator();
                    item(ui, "Exit", Act::Exit);
                });
                ui.menu_button("Actions", |ui| {
                    item(ui, "Select All\tCtrl+A", Act::SelectAll);
                    item(ui, "Deselect All\tCtrl+D", Act::DeselectAll);
                    item(ui, "Invert Selection\tCtrl+I", Act::Invert);
                    ui.separator();
                    item(ui, "Refresh Files\tF5", Act::Refresh);
                    item(ui, "Show/Hide Tree\tF11", Act::ShowTree);
                    item(ui, "Clear All Items from Current List\tF7", Act::ClearList);
                    ui.separator();
                    item(ui, "Reset All Renaming Criteria\tCtrl+T", Act::ResetAll);
                    item(ui, "Revert All Criteria to Last Saved\tCtrl+E", Act::Revert);
                    item(ui, "Preview\tCtrl+P", Act::Preview);
                    item(ui, "Rename\tCtrl+R", Act::Rename);
                    ui.separator();
                    item(ui, "Undo Rename\tCtrl+Z", Act::Undo);
                });
                ui.menu_button("Display Options", |ui| {
                    item(ui, "Show/Hide Tree\tF11", Act::ShowTree);
                });
                ui.menu_button("Renaming Options", |ui| {
                    item(ui, "Change Renaming Criteria Order...\tCtrl+F8", Act::Order);
                });
                ui.menu_button("Special", |ui| {
                    item(ui, "Change Renaming Criteria Order...\tCtrl+F8", Act::Order);
                    ui.separator();
                    item(ui, "Change File Attributes...", Act::NotAvailable("Change File Attributes"));
                    item(ui, "Change File Timestamps...", Act::NotAvailable("Change File Timestamps"));
                    item(ui, "Character Translations...\tCtrl+F6", Act::Translations);
                    item(ui, "Javascript Renaming...", Act::NotAvailable("Javascript Renaming"));
                });
                ui.menu_button("Tools", |ui| {
                    item(ui, "Preferences...", Act::NotAvailable("Preferences"));
                });
                ui.menu_button("Help", |ui| {
                    item(ui, "About", Act::About);
                });
            });
        });
        self.pending.extend(acts);
    }

    fn banner_and_nav(&mut self, ui: &mut egui::Ui) {
        let p = ui.painter().clone();
        let banner = r(8.0, 51.0, 1178.0, 89.0);
        p.rect_filled(banner, 0.0, BANNER);
        p.text(pos2(banner.min.x + 10.0, banner.center().y), Align2::LEFT_CENTER, "Bplus Rename", font(18.0), Color32::WHITE);
        // App badge at the right of the banner.
        let badge = Rect::from_center_size(pos2(banner.max.x - 26.0, banner.center().y), vec2(28.0, 28.0));
        p.image(self.badge.id(), badge, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);

        let nav_y0 = 93.0;
        let nav_y1 = 117.0;
        let mid = (nav_y0 + nav_y1) / 2.0 - OY;
        let arrow = |ui: &mut egui::Ui, x: f32, txt: &str, id: &str, enabled: bool| -> bool {
            let rr = Rect::from_center_size(pos2(x - OX, mid), vec2(18.0, 20.0));
            let resp = ui.interact(rr, ui.id().with(id), Sense::click());
            if resp.hovered() && enabled {
                ui.painter().rect_filled(rr, 0.0, HOVER);
            }
            let col = if enabled { Color32::from_rgb(0x50, 0x50, 0x50) } else { GREY_TEXT };
            ui.painter().text(rr.center(), Align2::CENTER_CENTER, txt, font(15.0), col);
            resp.clicked() && enabled
        };
        if arrow(ui, 21.0, "←", "back", !self.back.is_empty()) {
            self.pending.push(Act::Back);
        }
        if arrow(ui, 41.0, "→", "fwd", !self.fwd.is_empty()) {
            self.pending.push(Act::Forward);
        }
        {
            let rr = Rect::from_center_size(pos2(65.0 - OX, mid), vec2(20.0, 20.0));
            let resp = ui.interact(rr, ui.id().with("up"), Sense::click());
            if resp.hovered() {
                p.rect_filled(rr, 0.0, HOVER);
            }
            folder_icon(&p, pos2(rr.min.x + 3.0, rr.min.y + 5.0));
            let blue = Stroke::new(1.6, Color32::from_rgb(0x1F, 0x5F, 0xBF));
            p.line_segment([pos2(rr.min.x + 6.0, rr.max.y - 4.0), pos2(rr.min.x + 6.0, rr.min.y + 1.0)], blue);
            p.line_segment([pos2(rr.min.x + 3.0, rr.min.y + 4.0), pos2(rr.min.x + 6.0, rr.min.y + 1.0)], blue);
            p.line_segment([pos2(rr.min.x + 9.0, rr.min.y + 4.0), pos2(rr.min.x + 6.0, rr.min.y + 1.0)], blue);
            if resp.clicked() {
                self.pending.push(Act::Up);
            }
        }
        // Path box with its browse and history buttons.
        let pb = r(80.0, nav_y0, 1070.0, nav_y1);
        p.rect_filled(pb, 0.0, Color32::WHITE);
        p.rect_stroke(pb, 0.0, Stroke::new(1.0, EDGE), StrokeKind::Inside);
        let inner = Rect::from_min_max(pb.min, pos2(pb.max.x - 42.0, pb.max.y));
        let te = egui::TextEdit::singleline(&mut self.path_text)
            .id_salt("path")
            .font(font(12.0))
            .frame(egui::Frame::NONE)
            .margin(egui::Margin::symmetric(4, 1))
            .vertical_align(Align::Center)
            .min_size(inner.size())
            .desired_width(inner.width());
        let resp = ui.put(inner, te);
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let pth = PathBuf::from(self.path_text.trim());
            self.go_to(pth, true);
        }
        let fb = Rect::from_center_size(pos2(pb.max.x - 32.0, pb.center().y), vec2(18.0, 16.0));
        let fresp = ui.interact(fb, ui.id().with("browse"), Sense::click());
        folder_icon(&p, pos2(fb.min.x + 1.0, fb.min.y + 1.0));
        if fresp.clicked() {
            self.pending.push(Act::Browse);
        }
        p.line_segment(
            [pos2(pb.max.x - 20.0, pb.min.y + 1.0), pos2(pb.max.x - 20.0, pb.max.y - 1.0)],
            Stroke::new(1.0, Color32::from_rgb(0xDD, 0xDD, 0xDD)),
        );
        down_triangle(&p, pos2(pb.max.x - 10.0, pb.center().y));

        // Tool icons: refresh, select all, show only affected, clear list, menu.
        let icon_btn = |ui: &mut egui::Ui, x: f32, id: &str| -> (Rect, bool) {
            let rr = Rect::from_center_size(pos2(x - OX, mid), vec2(18.0, 18.0));
            let resp = ui.interact(rr, ui.id().with(id), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(rr, 0.0, HOVER);
            }
            (rr, resp.clicked())
        };
        let (rr, c) = icon_btn(ui, 1083.0, "refresh");
        refresh_icon(&p, rr.center());
        if c {
            self.pending.push(Act::Refresh);
        }
        let (rr, c) = icon_btn(ui, 1103.0, "selall");
        for i in 0..3 {
            let y = rr.min.y + 5.0 + i as f32 * 4.0;
            p.rect_filled(Rect::from_min_size(pos2(rr.min.x + 3.0, y - 1.0), vec2(2.5, 2.5)), 0.0, TEXT);
            p.line_segment([pos2(rr.min.x + 7.0, y), pos2(rr.max.x - 3.0, y)], Stroke::new(1.2, TEXT));
        }
        if c {
            self.pending.push(Act::SelectAll);
        }
        let (rr, c) = icon_btn(ui, 1122.0, "affected");
        for i in 0..3 {
            let y = rr.min.y + 5.0 + i as f32 * 4.0;
            p.line_segment([pos2(rr.min.x + 3.0, y), pos2(rr.max.x - 3.0, y)], Stroke::new(1.4, TEXT));
        }
        if c {
            self.pending.push(Act::DeselectAll);
        }
        let (rr, c) = icon_btn(ui, 1141.0, "clear");
        let red = Stroke::new(2.2, Color32::from_rgb(0xD3, 0x2F, 0x2F));
        p.line_segment([rr.min + vec2(4.0, 4.0), rr.max - vec2(4.0, 4.0)], red);
        p.line_segment([pos2(rr.max.x - 4.0, rr.min.y + 4.0), pos2(rr.min.x + 4.0, rr.max.y - 4.0)], red);
        if c {
            self.pending.push(Act::ClearList);
        }
        let (rr, _) = icon_btn(ui, 1163.0, "dropdown");
        down_triangle(&p, rr.center());
    }

    fn tree_and_list(&mut self, ui: &mut egui::Ui) {
        if self.show_tree {
            self.tree_ui(ui, r(8.0, 123.0, 220.0, 266.0));
            self.list_ui(ui, r(226.0, 123.0, 1170.0, 266.0));
        } else {
            self.list_ui(ui, r(8.0, 123.0, 1170.0, 266.0));
        }
    }

    fn tree_ui(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let p = ui.painter().clone();
        p.rect_filled(rect, 0.0, Color32::WHITE);
        p.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(0xAB, 0xAD, 0xB3)), StrokeKind::Inside);
        let mut toggles: Vec<Vec<usize>> = Vec::new();
        let mut select: Option<PathBuf> = None;
        let cur = self.cur_dir.clone();
        let scroll = std::mem::take(&mut self.tree_scroll);
        let tree = &self.tree;
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect.shrink(1.0)), |ui| {
            egui::ScrollArea::both().id_salt("tree").auto_shrink([false, false]).show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                tree_rows(ui, tree, 0, &mut Vec::new(), &cur, scroll, &mut toggles, &mut select);
            });
        });
        for t in toggles {
            if let Some(n) = node_at(&mut self.tree, &t) {
                n.load();
                n.expanded = !n.expanded;
            }
        }
        if let Some(s) = select {
            self.go_to(s, true);
        }
    }

    fn list_ui(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let p = ui.painter().clone();
        p.rect_filled(rect, 0.0, Color32::WHITE);
        p.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(0xAB, 0xAD, 0xB3)), StrokeKind::Inside);
        let cols: [(&str, f32); 9] = [
            ("Name", 460.0),
            ("New Name", 470.0),
            ("Sub Dir.", 80.0),
            ("Type", 110.0),
            ("Size", 80.0),
            ("Created", 135.0),
            ("Modified", 135.0),
            ("Accessed", 135.0),
            ("Length", 60.0),
        ];
        let total_w: f32 = cols.iter().map(|c| c.1).sum();
        let header_h = 24.0;
        let row_h = 19.0;
        let n = self.items.len();
        let mut clicked: Option<(usize, egui::Modifiers, bool)> = None;
        let mut sort_click: Option<usize> = None;
        let sort = self.sort.unwrap_or((0, true));
        let items = &self.items;
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect.shrink(1.0)), |ui| {
            egui::ScrollArea::both().id_salt("list").auto_shrink([false, false]).show_viewport(ui, |ui, vp| {
                let content = vec2(total_w.max(ui.available_width()), header_h + n as f32 * row_h);
                let (all, _) = ui.allocate_exact_size(content, Sense::hover());
                let origin = all.min;
                let p = ui.painter().clone();
                let first = ((vp.min.y - header_h) / row_h).floor().max(0.0) as usize;
                let last = ((((vp.max.y - header_h) / row_h).ceil().max(0.0)) as usize + 1).min(n);
                for i in first..last {
                    let e = &items[i];
                    let row = Rect::from_min_size(pos2(origin.x, origin.y + header_h + i as f32 * row_h), vec2(content.x, row_h));
                    let resp = ui.interact(row, ui.id().with(("row", i)), Sense::click());
                    if resp.clicked() {
                        clicked = Some((i, ui.input(|inp| inp.modifiers), false));
                    }
                    if resp.double_clicked() {
                        clicked = Some((i, egui::Modifiers::NONE, true));
                    }
                    let fill = if e.selected {
                        SEL
                    } else if resp.hovered() {
                        HOVER
                    } else {
                        ROW
                    };
                    p.rect_filled(row, 0.0, fill);
                    if e.selected {
                        p.rect_stroke(row, 0.0, Stroke::new(1.0, SEL_EDGE), StrokeKind::Inside);
                    }
                    let invalid = e.selected && e.new_name != e.name && engine::has_invalid(&e.new_name);
                    let ty = type_label(e);
                    let fmt = |d: &Option<chrono::DateTime<chrono::Local>>| {
                        d.map(|d| d.format("%d/%m/%Y %H:%M:%S").to_string()).unwrap_or_default()
                    };
                    let vals = [
                        e.name.clone(),
                        e.new_name.clone(),
                        e.sub_dir.clone(),
                        ty,
                        if e.info.is_dir { String::new() } else { files::human_size(e.size) },
                        fmt(&e.info.created),
                        fmt(&e.info.modified),
                        fmt(&e.info.accessed),
                        e.new_name.chars().count().to_string(),
                    ];
                    let mut x = origin.x;
                    for (k, (_, w)) in cols.iter().enumerate() {
                        let cell = Rect::from_min_size(pos2(x, row.min.y), vec2(*w - 2.0, row_h));
                        let clip = p.with_clip_rect(cell.intersect(p.clip_rect()));
                        let mut tx = cell.min.x + 6.0;
                        if k == 0 {
                            if e.info.is_dir {
                                folder_icon(&clip, pos2(tx, cell.min.y + 3.0));
                            } else {
                                file_icon(&clip, pos2(tx, cell.min.y + 2.0));
                            }
                            tx += 20.0;
                        }
                        let col = if k == 1 && invalid {
                            INVALID
                        } else if e.hidden {
                            GREY_TEXT
                        } else {
                            TEXT
                        };
                        if k == 4 || k == 8 {
                            clip.text(pos2(cell.max.x - 6.0, cell.center().y), Align2::RIGHT_CENTER, &vals[k], font(12.0), col);
                        } else {
                            clip.text(pos2(tx, cell.center().y), Align2::LEFT_CENTER, &vals[k], font(12.0), col);
                        }
                        x += w;
                    }
                }
                // Header row stays at the top of the view.
                let hy = origin.y + vp.min.y;
                let hr = Rect::from_min_size(pos2(origin.x, hy), vec2(content.x, header_h));
                p.rect_filled(hr, 0.0, Color32::WHITE);
                let mut x = origin.x;
                for (k, (name, w)) in cols.iter().enumerate() {
                    let cell = Rect::from_min_size(pos2(x, hy), vec2(*w, header_h));
                    // Registered after the rows so the header wins the hit test where they overlap.
                    let hresp = ui.interact(cell, ui.id().with(("head", k)), Sense::click());
                    if hresp.hovered() {
                        p.rect_filled(cell, 0.0, HOVER);
                    }
                    if hresp.clicked() {
                        sort_click = Some(k);
                    }
                    p.text(pos2(cell.min.x + 6.0, cell.center().y), Align2::LEFT_CENTER, *name, font(12.0), Color32::from_rgb(0x2C, 0x2C, 0x2C));
                    if k == sort.0 {
                        let gx = cell.min.x + 6.0 + p.layout_no_wrap(name.to_string(), font(12.0), TEXT).size().x + 14.0;
                        let c = pos2(gx, cell.center().y);
                        let d = if sort.1 { 1.0 } else { -1.0 };
                        p.add(egui::Shape::convex_polygon(
                            vec![pos2(c.x - 4.0, c.y + 2.5 * d), pos2(c.x + 4.0, c.y + 2.5 * d), pos2(c.x, c.y - 2.5 * d)],
                            Color32::from_rgb(0x1E, 0x3C, 0xD8),
                            Stroke::NONE,
                        ));
                    }
                    p.line_segment(
                        [pos2(cell.max.x - 1.0, hy + 2.0), pos2(cell.max.x - 1.0, hy + header_h - 2.0)],
                        Stroke::new(1.0, Color32::from_rgb(0xE5, 0xE5, 0xE5)),
                    );
                    x += w;
                }
            });
        });
        if let Some(k) = sort_click {
            // Same column flips the direction; a new column starts ascending.
            self.sort = Some((k, if k == sort.0 { !sort.1 } else { true }));
            self.apply_sort();
            return;
        }
        if let Some((i, m, dbl)) = clicked {
            if dbl {
                if self.items[i].info.is_dir {
                    let p = self.items[i].info.path.clone();
                    self.go_to(p, true);
                }
            } else if m.shift {
                let a = self.anchor.unwrap_or(i);
                let (lo, hi) = (a.min(i), a.max(i));
                if !m.command {
                    self.items.iter_mut().for_each(|e| e.selected = false);
                }
                for e in &mut self.items[lo..=hi] {
                    e.selected = true;
                }
            } else if m.command {
                self.items[i].selected = !self.items[i].selected;
                self.anchor = Some(i);
            } else {
                self.items.iter_mut().for_each(|e| e.selected = false);
                self.items[i].selected = true;
                self.anchor = Some(i);
            }
        }
    }

    fn criteria_panels(&mut self, ui: &mut egui::Ui) {
        let c = &mut self.crit;

        // ---- RegEx (1) ----
        let pn = Pan::new(r(16.0, 279.0, 256.0, 375.0), [4.0, 127.0, 106.0, 52.0]);
        group(ui, &pn, "RegEx (1)");
        check(ui, pn.at(87.0, 128.0, 10.0, 8.0), &mut c.regex.on, "", "rx_on");
        if small_r(ui, pn.at(97.0, 127.0, 9.0, 10.0), "rx_r") {
            c.regex = Criteria::default().regex;
        }
        label(ui, pn.at(8.0, 141.0, 23.0, 8.0), "Match");
        list_icon(ui.painter(), pn.at(32.0, 141.0, 6.0, 6.0).center());
        edit_combo(ui, pn.at(42.0, 139.0, 63.0, 12.0), &mut c.regex.matcher, "rx_m");
        label(ui, pn.at(8.0, 154.0, 27.0, 8.0), "Replace");
        edit_combo(ui, pn.at(42.0, 153.0, 63.0, 12.0), &mut c.regex.replace, "rx_rep");
        check(ui, pn.at(9.0, 167.0, 37.0, 10.0), &mut c.regex.inc_ext, "Inc.Ext.", "rx_ext");
        check(ui, pn.at(49.0, 167.0, 33.0, 10.0), &mut c.regex.simple, "Simple", "rx_simple");
        check(ui, pn.at(87.0, 167.0, 21.0, 10.0), &mut c.regex.v2, "v2", "rx_v2");

        // ---- Name (2) ----
        let pn = Pan::new(r(16.0, 378.0, 256.0, 466.0), [4.0, 181.0, 106.0, 46.0]);
        group(ui, &pn, "Name (2)");
        check(ui, pn.at(87.0, 182.0, 10.0, 8.0), &mut c.name.on, "", "nm_on");
        if small_r(ui, pn.at(97.0, 181.0, 9.0, 10.0), "nm_r") {
            c.name = Criteria::default().name;
        }
        label(ui, pn.at(8.0, 197.0, 20.0, 8.0), "Name");
        combo(ui, pn.at(30.0, 195.0, 73.0, 12.0), "nm_mode", &mut c.name.mode, &engine::NAME_MODES);
        edit_combo(ui, pn.at(30.0, 210.0, 73.0, 12.0), &mut c.name.text, "nm_text");

        // ---- Replace (3) ----
        let pn = Pan::new(r(262.0, 279.0, 484.0, 375.0), [113.0, 127.0, 95.0, 52.0]);
        group(ui, &pn, "Replace (3)");
        check(ui, pn.at(185.0, 128.0, 10.0, 8.0), &mut c.replace.on, "", "rp_on");
        if small_r(ui, pn.at(195.0, 127.0, 9.0, 10.0), "rp_r") {
            c.replace = Criteria::default().replace;
        }
        label(ui, pn.at(117.0, 141.0, 28.0, 8.0), "Replace");
        edit_combo(ui, pn.at(148.0, 139.0, 55.0, 12.0), &mut c.replace.find, "rp_find");
        label(ui, pn.at(117.0, 154.0, 17.0, 8.0), "With");
        list_icon(ui.painter(), pn.at(137.0, 154.0, 6.0, 6.0).center());
        edit_combo(ui, pn.at(148.0, 153.0, 55.0, 12.0), &mut c.replace.with, "rp_with");
        check(ui, pn.at(118.0, 167.0, 54.0, 10.0), &mut c.replace.match_case, "Match Case", "rp_mc");
        check(ui, pn.at(173.0, 167.0, 29.0, 10.0), &mut c.replace.first, "First", "rp_first");

        // ---- Case (4) ----
        let pn = Pan::new(r(262.0, 378.0, 484.0, 466.0), [113.0, 181.0, 95.0, 46.0]);
        group(ui, &pn, "Case (4)");
        check(ui, pn.at(185.0, 182.0, 10.0, 8.0), &mut c.case.on, "", "cs_on");
        if small_r(ui, pn.at(195.0, 181.0, 9.0, 10.0), "cs_r") {
            c.case = Criteria::default().case;
        }
        combo(ui, pn.at(118.0, 195.0, 84.0, 12.0), "cs_mode", &mut c.case.mode, &engine::CASE_MODES);
        label(ui, pn.at(117.0, 212.0, 24.0, 8.0), "Excep.");
        list_icon(ui.painter(), pn.at(139.0, 212.0, 6.0, 6.0).center());
        edit_combo(ui, pn.at(148.0, 210.0, 53.0, 12.0), &mut c.case.except, "cs_ex");

        // ---- Remove (5) ----
        let pn = Pan::new(r(490.0, 279.0, 680.0, 466.0), [211.0, 127.0, 110.0, 100.0]);
        group(ui, &pn, "Remove (5)");
        let rm = &mut c.remove;
        check(ui, pn.at(299.0, 128.0, 10.0, 8.0), &mut rm.on, "", "rm_on");
        if small_r(ui, pn.at(309.0, 127.0, 9.0, 10.0), "rm_r") {
            *rm = Criteria::default().remove;
        }
        label(ui, pn.at(214.0, 141.0, 20.0, 8.0), "First n");
        spin(ui, pn.at(236.0, 139.0, 27.0, 12.0), &mut rm.first_n, "rm_first", 0, true);
        label(ui, pn.at(268.0, 141.0, 20.0, 8.0), "Last n");
        spin(ui, pn.at(290.0, 139.0, 27.0, 12.0), &mut rm.last_n, "rm_last", 0, true);
        label(ui, pn.at(214.0, 153.0, 18.0, 8.0), "From");
        spin(ui, pn.at(236.0, 152.0, 27.0, 12.0), &mut rm.from, "rm_from", 0, true);
        label(ui, pn.at(273.0, 153.0, 8.0, 8.0), "to");
        spin(ui, pn.at(290.0, 152.0, 27.0, 12.0), &mut rm.to, "rm_to", 0, true);
        label(ui, pn.at(214.0, 167.0, 19.0, 8.0), "Chars");
        edit(ui, pn.at(236.0, 166.0, 21.0, 12.0), &mut rm.chars, "rm_chars", true);
        label(ui, pn.at(263.0, 167.0, 22.0, 8.0), "Words");
        edit(ui, pn.at(287.0, 166.0, 30.0, 12.0), &mut rm.words, "rm_words", true);
        label(ui, pn.at(214.0, 181.0, 16.0, 8.0), "Crop");
        combo(ui, pn.at(236.0, 180.0, 38.0, 12.0), "rm_crop", &mut rm.crop, &engine::CROP_MODES);
        edit(ui, pn.at(287.0, 179.0, 30.0, 12.0), &mut rm.crop_marker, "rm_marker", true);
        check(ui, pn.at(215.0, 195.0, 30.0, 10.0), &mut rm.digits, "Digits", "rm_digits");
        check(ui, pn.at(250.0, 195.0, 30.0, 10.0), &mut rm.high, "High", "rm_high");
        check(ui, pn.at(287.0, 192.0, 30.0, 10.0), &mut rm.trim, "Trim", "rm_trim");
        check(ui, pn.at(215.0, 205.0, 29.0, 10.0), &mut rm.ds, "D/S", "rm_ds");
        check(ui, pn.at(245.0, 205.0, 41.0, 10.0), &mut rm.accents, "Accents", "rm_acc");
        check(ui, pn.at(287.0, 202.0, 32.0, 10.0), &mut rm.alpha, "Chars", "rm_alpha");
        check(ui, pn.at(215.0, 215.0, 29.0, 10.0), &mut rm.sym, "Sym.", "rm_sym");
        label(ui, pn.at(248.0, 215.0, 35.0, 8.0), "Lead Dots");
        combo(ui, pn.at(287.0, 212.0, 30.0, 12.0), "rm_dots", &mut rm.lead_dots, &engine::LEAD_DOTS);

        // ---- Add (7) ----
        let pn = Pan::new(r(685.0, 279.0, 803.0, 466.0), [324.0, 127.0, 70.0, 100.0]);
        group(ui, &pn, "Add (7)");
        let ad = &mut c.add;
        check(ui, pn.at(372.0, 128.0, 10.0, 8.0), &mut ad.on, "", "ad_on");
        if small_r(ui, pn.at(382.0, 127.0, 9.0, 10.0), "ad_r") {
            *ad = Criteria::default().add;
        }
        label(ui, pn.at(327.0, 141.0, 18.0, 8.0), "Prefix");
        edit_combo(ui, pn.at(348.0, 139.0, 42.0, 12.0), &mut ad.prefix, "ad_pre");
        label(ui, pn.at(327.0, 154.0, 18.0, 8.0), "Insert");
        edit_combo(ui, pn.at(348.0, 153.0, 42.0, 12.0), &mut ad.insert, "ad_ins");
        label(ui, pn.at(330.0, 170.0, 22.0, 8.0), "at pos.");
        spin(ui, pn.at(356.0, 169.0, 31.0, 12.0), &mut ad.at, "ad_at", -999, true);
        label(ui, pn.at(327.0, 187.0, 21.0, 8.0), "Suffix");
        edit_combo(ui, pn.at(348.0, 186.0, 42.0, 12.0), &mut ad.suffix, "ad_suf");
        check(ui, pn.at(330.0, 202.0, 60.0, 10.0), &mut ad.word_space, "Word Space", "ad_ws");
        {
            let e = Rect::from_center_size(pn.at(380.0, 213.0, 10.0, 10.0).center(), vec2(9.0, 9.0));
            let st = Stroke::new(1.0, Color32::from_rgb(0x60, 0x60, 0x60));
            ui.painter().rect_stroke(e, 0.0, st, StrokeKind::Inside);
            ui.painter().rect_stroke(e.shrink(2.5), 0.0, st, StrokeKind::Inside);
        }

        // ---- Auto Date (8) ----
        let pn = Pan::new(r(810.0, 279.0, 958.0, 466.0), [396.0, 127.0, 87.0, 100.0]);
        group(ui, &pn, "Auto Date (8)");
        let dt = &mut c.date;
        check(ui, pn.at(462.0, 128.0, 10.0, 8.0), &mut dt.on, "", "dt_on");
        if small_r(ui, pn.at(472.0, 127.0, 9.0, 10.0), "dt_r") {
            *dt = Criteria::default().date;
        }
        label(ui, pn.at(400.0, 141.0, 22.0, 8.0), "Mode");
        combo(ui, pn.at(423.0, 139.0, 57.0, 12.0), "dt_mode", &mut dt.mode, &engine::DATE_MODES);
        label(ui, pn.at(400.0, 153.0, 17.0, 8.0), "Type");
        combo(ui, pn.at(423.0, 153.0, 57.0, 12.0), "dt_type", &mut dt.kind, &engine::DATE_TYPES);
        label(ui, pn.at(401.0, 169.0, 14.0, 8.0), "Fmt");
        combo(ui, pn.at(423.0, 167.0, 57.0, 12.0), "dt_fmt", &mut dt.fmt, &engine::DATE_FORMATS);
        label(ui, pn.at(404.0, 184.0, 17.0, 8.0), "Sep.");
        edit(ui, pn.at(423.0, 182.0, 16.0, 12.0), &mut dt.sep, "dt_sep", true);
        label(ui, pn.at(443.0, 184.0, 16.0, 8.0), "Seg.");
        edit(ui, pn.at(461.0, 182.0, 18.0, 12.0), &mut dt.seg, "dt_seg", true);
        label(ui, pn.at(401.0, 197.0, 25.0, 8.0), "Custom");
        edit_combo(ui, pn.at(431.0, 196.0, 48.0, 12.0), &mut dt.custom, "dt_custom");
        check(ui, pn.at(402.0, 212.0, 29.0, 10.0), &mut dt.century, "Cent.", "dt_cent");
        label(ui, pn.at(437.0, 213.0, 13.0, 8.0), "Off.");
        spin(ui, pn.at(451.0, 211.0, 26.0, 12.0), &mut dt.offset, "dt_off", -999, true);

        // ---- Numbering (10) ----
        let pn = Pan::new(r(966.0, 279.0, 1168.0, 466.0), [486.0, 127.0, 117.0, 100.0]);
        group(ui, &pn, "Numbering (10)");
        let nb = &mut c.number;
        check(ui, pn.at(581.0, 128.0, 10.0, 8.0), &mut nb.on, "", "nb_on");
        if small_r(ui, pn.at(591.0, 127.0, 9.0, 10.0), "nb_r") {
            *nb = Criteria::default().number;
        }
        label(ui, pn.at(489.0, 141.0, 22.0, 8.0), "Mode");
        combo(ui, pn.at(512.0, 139.0, 36.0, 12.0), "nb_mode", &mut nb.mode, &engine::NUM_MODES);
        label(ui, pn.at(556.0, 141.0, 8.0, 8.0), "at");
        spin(ui, pn.at(566.0, 139.0, 31.0, 12.0), &mut nb.at, "nb_at", -999, true);
        label(ui, pn.at(490.0, 154.0, 16.0, 8.0), "Start");
        spin(ui, pn.at(509.0, 153.0, 31.0, 12.0), &mut nb.start, "nb_start", -99999, true);
        label(ui, pn.at(551.0, 153.0, 15.0, 8.0), "Incr.");
        spin(ui, pn.at(566.0, 153.0, 31.0, 12.0), &mut nb.incr, "nb_incr", -99999, true);
        label(ui, pn.at(490.0, 167.0, 14.0, 8.0), "Pad");
        spin(ui, pn.at(509.0, 166.0, 31.0, 12.0), &mut nb.pad, "nb_pad", 0, true);
        label(ui, pn.at(551.0, 167.0, 16.0, 8.0), "Sep.");
        edit(ui, pn.at(566.0, 167.0, 24.0, 12.0), &mut nb.sep, "nb_sep", true);
        label(ui, pn.at(489.0, 179.0, 20.0, 8.0), "Break");
        spin(ui, pn.at(509.0, 178.0, 31.0, 12.0), &mut nb.brk, "nb_brk", 0, true);
        check(ui, pn.at(565.0, 180.0, 31.0, 10.0), &mut nb.folder, "Folder", "nb_folder");
        label(ui, pn.at(490.0, 195.0, 17.0, 8.0), "Type");
        let types = engine::num_types();
        let tref: Vec<&str> = types.iter().map(|s| s.as_str()).collect();
        combo(ui, pn.at(511.0, 193.0, 84.0, 12.0), "nb_type", &mut nb.kind, &tref);
        label(ui, pn.at(490.0, 210.0, 19.0, 8.0), "Case");
        combo(ui, pn.at(511.0, 208.0, 45.0, 12.0), "nb_case", &mut nb.case, &engine::NUM_CASE);

        // ---- Move/Copy Parts (6) ----
        let pn = Pan::new(r(16.0, 467.0, 546.0, 519.0), [4.0, 229.0, 307.0, 28.0]);
        group(ui, &pn, "Move/Copy Parts (6)");
        let mc = &mut c.movecopy;
        check(ui, pn.at(290.0, 230.0, 10.0, 8.0), &mut mc.on, "", "mc_on");
        if small_r(ui, pn.at(300.0, 229.0, 9.0, 10.0), "mc_r") {
            *mc = Criteria::default().movecopy;
        }
        combo(ui, pn.at(10.0, 241.0, 65.0, 12.0), "mc_src", &mut mc.source, &engine::MC_SOURCE);
        spin(ui, pn.at(79.0, 241.0, 28.0, 12.0), &mut mc.count, "mc_n", 0, true);
        let ranged = mc.source >= 5;
        spin(ui, pn.at(111.0, 241.0, 28.0, 12.0), &mut mc.range_to, "mc_to", 0, ranged);
        combo(ui, pn.at(149.0, 241.0, 41.0, 12.0), "mc_tgt", &mut mc.target, &engine::MC_TARGET);
        let to_pos = mc.target == 3;
        spin(ui, pn.at(194.0, 241.0, 32.0, 12.0), &mut mc.pos, "mc_pos", 1, to_pos);
        label(ui, pn.at(237.0, 243.0, 16.0, 8.0), "Sep.");
        edit(ui, pn.at(257.0, 241.0, 47.0, 12.0), &mut mc.sep, "mc_sep", true);

        // ---- Append Folder Name (9) ----
        let pn = Pan::new(r(672.0, 467.0, 958.0, 519.0), [316.0, 229.0, 167.0, 28.0]);
        group(ui, &pn, "Append Folder Name (9)");
        let fo = &mut c.folder;
        check(ui, pn.at(462.0, 230.0, 10.0, 8.0), &mut fo.on, "", "fo_on");
        if small_r(ui, pn.at(472.0, 229.0, 9.0, 10.0), "fo_r") {
            *fo = Criteria::default().folder;
        }
        label(ui, pn.at(322.0, 243.0, 20.0, 8.0), "Name");
        combo(ui, pn.at(345.0, 241.0, 36.0, 12.0), "fo_mode", &mut fo.mode, &engine::FOLDER_MODES);
        label(ui, pn.at(386.0, 243.0, 16.0, 8.0), "Sep.");
        edit(ui, pn.at(403.0, 241.0, 20.0, 12.0), &mut fo.sep, "fo_sep", true);
        label(ui, pn.at(428.0, 243.0, 22.0, 8.0), "Levels");
        spin(ui, pn.at(452.0, 241.0, 25.0, 12.0), &mut fo.levels, "fo_lv", 1, true);

        // ---- Extension (11) ----
        let pn = Pan::new(r(966.0, 467.0, 1168.0, 519.0), [486.0, 229.0, 117.0, 28.0]);
        group(ui, &pn, "Extension (11)");
        let ex = &mut c.ext;
        check(ui, pn.at(581.0, 230.0, 10.0, 8.0), &mut ex.on, "", "ex_on");
        if small_r(ui, pn.at(591.0, 229.0, 9.0, 10.0), "ex_r") {
            *ex = Criteria::default().ext;
        }
        combo(ui, pn.at(494.0, 241.0, 43.0, 12.0), "ex_mode", &mut ex.mode, &engine::EXT_MODES);
        edit_combo(ui, pn.at(540.0, 241.0, 57.0, 12.0), &mut ex.text, "ex_text");

        // Divider above the selection area.
        let dv = r(16.0, 530.0, 1168.0, 532.0);
        ui.painter().line_segment([dv.left_top(), dv.right_top()], Stroke::new(1.0, Color32::from_rgb(0xA0, 0xA0, 0xA0)));
        ui.painter().line_segment([dv.left_bottom(), dv.right_bottom()], Stroke::new(1.0, Color32::WHITE));

        self.filter_panel(ui);
        self.location_panels(ui);
        self.special_panel(ui);
    }

    fn filter_panel(&mut self, ui: &mut egui::Ui) {
        let pn = Pan::new(r(16.0, 541.0, 674.0, 650.0), [4.0, 269.0, 383.0, 58.0]);
        group(ui, &pn, "Filters (12)");
        if small_r(ui, pn.at(376.0, 269.0, 9.0, 10.0), "fl_r") {
            self.filters = Filters::default();
            self.mask_edit = self.filters.mask.clone();
        }
        label(ui, pn.at(7.0, 283.0, 18.0, 10.0), "Mask");
        list_icon(ui.painter(), pn.at(27.0, 283.0, 6.0, 6.0).center());
        let resp = edit_combo(ui, pn.at(37.0, 281.0, 93.0, 12.0), &mut self.mask_edit, "fl_mask");
        if resp.lost_focus() && self.mask_edit != self.filters.mask {
            self.filters.mask = self.mask_edit.clone();
        }
        let f = &mut self.filters;
        let rf = pn.at(133.0, 281.0, 13.0, 11.0);
        refresh_icon(ui.painter(), rf.center());
        if ui.interact(rf, ui.id().with("fl_ref"), Sense::click()).clicked() {
            self.pending.push(Act::Refresh);
        }
        // Small "list" picture under the mask label.
        let pic = pn.at(8.0, 294.0, 10.0, 10.0);
        ui.painter().rect_stroke(
            Rect::from_center_size(pic.center(), vec2(10.0, 9.0)),
            0.0,
            Stroke::new(1.0, Color32::from_rgb(0x50, 0x50, 0x50)),
            StrokeKind::Inside,
        );
        check(ui, pn.at(31.0, 296.0, 56.0, 10.0), &mut f.match_case, "Match Case", "fl_mc");
        check(ui, pn.at(91.0, 296.0, 37.0, 10.0), &mut f.regex, "RegEx", "fl_rx");
        check(ui, pn.at(154.0, 281.0, 38.0, 10.0), &mut f.folders, "Folders", "fl_fo");
        check(ui, pn.at(154.0, 294.0, 30.0, 10.0), &mut f.files, "Files", "fl_fi");
        check(ui, pn.at(198.0, 281.0, 39.0, 10.0), &mut f.hidden, "Hidden", "fl_hid");
        check(ui, pn.at(189.0, 294.0, 49.0, 10.0), &mut f.subfolders, "Subfolders", "fl_sub");
        label(ui, pn.at(241.0, 295.0, 10.0, 8.0), "Lvl");
        edit(ui, pn.at(253.0, 293.0, 13.0, 12.0), &mut f.level, "fl_lvl", true);
        label(ui, pn.at(263.0, 281.0, 35.0, 8.0), "Name Min");
        spin(ui, pn.at(302.0, 279.0, 24.0, 12.0), &mut f.name_min, "fl_nmin", 0, true);
        label(ui, pn.at(331.0, 281.0, 14.0, 8.0), "Max");
        spin(ui, pn.at(347.0, 279.0, 25.0, 12.0), &mut f.name_max, "fl_nmax", 0, true);
        label(ui, pn.at(269.0, 294.0, 29.0, 8.0), "Path Min");
        spin(ui, pn.at(302.0, 293.0, 24.0, 12.0), &mut f.path_min, "fl_pmin", 0, true);
        label(ui, pn.at(331.0, 294.0, 14.0, 8.0), "Max");
        spin(ui, pn.at(347.0, 293.0, 25.0, 12.0), &mut f.path_max, "fl_pmax", 0, true);
        label(ui, pn.at(116.0, 310.0, 35.0, 10.0), "Condition");
        edit_combo(ui, pn.at(153.0, 309.0, 206.0, 12.0), &mut f.condition, "fl_cond");
        let rf = pn.at(361.0, 308.0, 13.0, 11.0);
        refresh_icon(ui.painter(), rf.center());
        if ui.interact(rf, ui.id().with("fl_ref2"), Sense::click()).clicked() {
            self.pending.push(Act::Refresh);
        }
    }

    fn location_panels(&mut self, ui: &mut egui::Ui) {
        let pn = Pan::new(r(684.0, 541.0, 1168.0, 595.0), [391.0, 269.0, 212.0, 29.0]);
        group(ui, &pn, "Copy/Move to Location (13)");
        if small_r(ui, pn.at(591.0, 269.0, 9.0, 10.0), "lc_r") {
            self.loc = Location { copy_not_move: true, ..Default::default() };
        }
        let ic = pn.at(395.0, 283.0, 10.0, 10.0);
        folder_icon(ui.painter(), pos2(ic.min.x, ic.min.y - 1.0));
        label(ui, pn.at(408.0, 283.0, 19.0, 10.0), "Path");
        let before = self.loc.path.clone();
        edit_combo(ui, pn.at(426.0, 281.0, 50.0, 12.0), &mut self.loc.path, "lc_path");
        if self.loc.path != before {
            self.loc.on = !self.loc.path.trim().is_empty();
        }
        let fb = pn.at(477.0, 281.0, 13.0, 11.0);
        folder_icon(ui.painter(), pos2(fb.min.x + 2.0, fb.min.y + 2.0));
        if ui.interact(fb, ui.id().with("lc_browse"), Sense::click()).clicked() {
            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                self.loc.path = p.to_string_lossy().into_owned();
                self.loc.on = true;
            }
        }
        check(ui, pn.at(495.0, 282.0, 64.0, 10.0), &mut self.loc.copy_not_move, "Copy not Move", "lc_copy");
        check(ui, pn.at(559.0, 282.0, 42.0, 10.0), &mut self.loc.keep_structure, "Keep Str.", "lc_keep");

        let pn = Pan::new(r(686.0, 599.0, 852.0, 647.0), [391.0, 300.0, 99.0, 26.0]);
        group(ui, &pn, "Name Segment");
        let sg = &mut self.crit.segment;
        check(ui, pn.at(468.0, 300.0, 10.0, 8.0), &mut sg.on, "", "sg_on");
        if small_r(ui, pn.at(477.0, 299.0, 9.0, 10.0), "sg_r") {
            *sg = Criteria::default().segment;
        }
        label(ui, pn.at(398.0, 313.0, 16.0, 8.0), "From");
        spin(ui, pn.at(416.0, 311.0, 27.0, 12.0), &mut sg.from, "sg_from", 0, true);
        label(ui, pn.at(445.0, 313.0, 10.0, 8.0), "To");
        spin(ui, pn.at(456.0, 311.0, 27.0, 12.0), &mut sg.to, "sg_to", 0, true);
    }

    fn special_panel(&mut self, ui: &mut egui::Ui) {
        let pn = Pan::new(r(16.0, 653.0, 742.0, 720.0), [4.0, 329.0, 421.0, 37.0]);
        group(ui, &pn, "Special (14)");
        let custom_order = self.crit.order != (1..=11).collect::<Vec<u8>>();
        let tr_set = self.crit.translations_on && !self.crit.translations.is_empty();
        let items: [(f32, f32, &str, bool, Act); 5] = [
            (12.0, 23.0, "Order", custom_order, Act::Order),
            (53.0, 64.0, "Change File Attributes", false, Act::NotAvailable("Change File Attributes")),
            (145.0, 156.0, "Change File Timestamps", false, Act::NotAvailable("Change File Timestamps")),
            (244.0, 255.0, "Character Translations", tr_set, Act::Translations),
            (338.0, 349.0, "Javascript Renaming", false, Act::NotAvailable("Javascript Renaming")),
        ];
        let colors = [
            Color32::from_rgb(0x3A, 0x7B, 0xD5),
            Color32::from_rgb(0x2E, 0x7D, 0x32),
            Color32::from_rgb(0x6A, 0x6A, 0x9A),
            Color32::from_rgb(0x55, 0x55, 0x55),
        ];
        for (k, (cx, lx, name, set, act)) in items.into_iter().enumerate() {
            let mut v = set;
            if check(ui, pn.at(cx, 345.0, 8.0, 8.0), &mut v, "", &format!("sp_{k}")) {
                self.pending.push(act);
            }
            let lr = pn.at(lx, 341.0, 80.0, 8.0);
            let resp = ui.interact(lr, ui.id().with(("sp_l", k)), Sense::click());
            label(ui, lr, name);
            if resp.clicked() {
                self.pending.push(act);
            }
            let status = if set { "Set" } else { "Not Set" };
            if k == 0 {
                label(ui, pn.at(23.0, 351.0, 25.0, 8.0), status);
            } else {
                let b = Rect::from_center_size(pn.at(lx, 351.0, 10.0, 9.0).center(), vec2(13.0, 12.0));
                ui.painter().rect_filled(b, 1.0, Color32::WHITE);
                ui.painter().rect_stroke(b, 1.0, Stroke::new(1.0, colors[k - 1]), StrokeKind::Inside);
                ui.painter().rect_filled(Rect::from_min_size(b.min, vec2(13.0, 3.0)), 0.0, colors[k - 1]);
                label(ui, pn.at(lx + 13.0, 351.0, 23.0, 8.0), "Status:");
                label(ui, pn.at(lx + 37.0, 351.0, 30.0, 8.0), status);
            }
        }
    }

    fn bottom(&mut self, ui: &mut egui::Ui) {
        let p = ui.painter().clone();
        // Brand block.
        let br = r(838.0, 662.0, 930.0, 712.0);
        p.text(br.min, Align2::LEFT_TOP, "Bplus Rename", bold(12.0), TEXT);
        p.text(br.min + vec2(0.0, 17.0), Align2::LEFT_TOP, "64-bit", bold(12.0), TEXT);
        p.text(br.min + vec2(0.0, 34.0), Align2::LEFT_TOP, concat!("Version ", env!("CARGO_PKG_VERSION")), font(12.0), TEXT);
        // Expand list / detach / help icons.
        let ex = Rect::from_center_size(r(957.0, 657.0, 970.0, 670.0).center(), vec2(10.0, 10.0));
        p.rect_stroke(ex, 0.0, Stroke::new(1.0, TEXT), StrokeKind::Inside);
        p.rect_stroke(ex.shrink(2.5), 0.0, Stroke::new(1.0, TEXT), StrokeKind::Inside);
        let dt = Rect::from_center_size(r(957.0, 682.0, 970.0, 695.0).center(), vec2(11.0, 9.0));
        p.rect_stroke(dt, 0.0, Stroke::new(1.0, TEXT), StrokeKind::Inside);
        p.rect_filled(Rect::from_min_size(dt.min, vec2(11.0, 2.0)), 0.0, TEXT);
        let hp = r(956.0, 706.0, 971.0, 721.0);
        p.circle_filled(hp.center(), 7.0, Color32::from_rgb(0x20, 0x20, 0x20));
        p.text(hp.center(), Align2::CENTER_CENTER, "?", bold(11.0), Color32::WHITE);
        if ui.interact(hp, ui.id().with("help"), Sense::click()).clicked() {
            self.pending.push(Act::About);
        }

        // Open (favourites) and Auto Refresh.
        let ob = r(996.0, 615.0, 1056.0, 635.0);
        let oresp = ui.interact(ob, ui.id().with("open"), Sense::click());
        if oresp.hovered() {
            p.rect_filled(ob, 0.0, HOVER);
            p.rect_stroke(ob, 0.0, Stroke::new(1.0, HOVER_EDGE), StrokeKind::Inside);
        }
        p.text(pos2(ob.min.x + 8.0, ob.center().y), Align2::LEFT_CENTER, "Open", font(12.0), TEXT);
        down_triangle(&p, pos2(ob.max.x - 10.0, ob.center().y));
        if oresp.clicked() {
            self.pending.push(Act::Open);
        }
        let ar = r(1062.0, 615.0, 1162.0, 635.0);
        let aresp = ui.interact(ar, ui.id().with("autoref"), Sense::click());
        if aresp.clicked() {
            self.auto_refresh = !self.auto_refresh;
        }
        p.text(pos2(ar.min.x + 8.0, ar.center().y), Align2::LEFT_CENTER, "Auto Refresh:", font(11.5), TEXT);
        let c = pos2(ar.max.x - 12.0, ar.center().y);
        if self.auto_refresh {
            let st = Stroke::new(2.0, Color32::from_rgb(0x3C, 0xA0, 0x3C));
            p.line_segment([pos2(c.x - 5.0, c.y), pos2(c.x - 1.5, c.y + 4.0)], st);
            p.line_segment([pos2(c.x - 1.5, c.y + 4.0), pos2(c.x + 6.0, c.y - 5.0)], st);
        } else {
            let st = Stroke::new(2.0, Color32::from_rgb(0xC0, 0x30, 0x30));
            p.line_segment([pos2(c.x - 4.0, c.y - 4.0), pos2(c.x + 4.0, c.y + 4.0)], st);
            p.line_segment([pos2(c.x + 4.0, c.y - 4.0), pos2(c.x - 4.0, c.y + 4.0)], st);
        }
        p.rect_filled(r(996.0, 643.0, 1162.0, 647.0), 0.0, YELLOW);

        if button(ui, r(992.0, 657.0, 1056.0, 677.0), "Preview", "b_prev") {
            self.pending.push(Act::Preview);
        }
        if button(ui, r(992.0, 681.0, 1056.0, 701.0), "Reset", "b_reset") {
            self.pending.push(Act::ResetAll);
        }
        if button(ui, r(992.0, 705.0, 1056.0, 725.0), "Revert", "b_revert") {
            self.pending.push(Act::Revert);
        }
        let rn = r(1072.0, 641.0, 1164.0, 725.0);
        let rresp = ui.interact(rn, ui.id().with("rename"), Sense::click());
        let can = self.items.iter().any(|e| e.selected);
        let face = if rresp.hovered() && can { Color32::from_rgb(0xB4, 0xC4, 0xEE) } else { RENAME_FACE };
        p.rect_filled(rn, 0.0, face);
        p.rect_stroke(rn, 0.0, Stroke::new(1.0, Color32::from_rgb(0xA8, 0xB4, 0xD0)), StrokeKind::Inside);
        let tcol = if can { TEXT } else { Color32::from_rgb(0x8A, 0x92, 0xA8) };
        p.text(rn.center(), Align2::CENTER_CENTER, "Rename", font(12.5), tcol);
        if rresp.clicked() {
            self.pending.push(Act::Rename);
        }

        // Notice bar.
        let nb = r(8.0, 732.0, 1178.0, 756.0);
        p.rect_filled(nb, 0.0, NOTICE);
        let mut job = egui::text::LayoutJob::default();
        let fmt = |f: FontId, c: Color32, u: bool| egui::TextFormat {
            font_id: f,
            color: c,
            underline: if u { Stroke::new(1.0, c) } else { Stroke::NONE },
            ..Default::default()
        };
        job.append("Bplus Rename renames ", 0.0, fmt(font(12.0), TEXT, false));
        job.append("many files and folders at once", 0.0, fmt(bold(12.0), TEXT, false));
        job.append(". Keep a backup of important files before renaming.  ", 0.0, fmt(font(12.0), TEXT, false));
        job.append("More Info", 0.0, fmt(bold(12.0), Color32::from_rgb(0x00, 0x33, 0x99), true));
        let gal = p.layout_job(job);
        let tp = pos2(nb.min.x + 10.0, nb.center().y - gal.size().y / 2.0);
        let link = Rect::from_min_size(tp, gal.size());
        p.galley(tp, gal, TEXT);
        if ui.interact(link, ui.id().with("moreinfo"), Sense::click()).clicked() {
            self.pending.push(Act::About);
        }

        // Status bar.
        let sb = r(8.0, 760.0, 1178.0, 783.0);
        p.rect_filled(sb, 0.0, BG);
        let mut left = format!("{} Objects ({} Selected)", self.items.len(), self.selected_count());
        if !self.status_note.is_empty() {
            left.push_str("   ");
            left.push_str(&self.status_note);
        }
        p.text(pos2(sb.min.x + 2.0, sb.center().y), Align2::LEFT_CENTER, left, font(11.5), TEXT);
        p.text(pos2(sb.min.x + 463.0, sb.center().y), Align2::LEFT_CENTER, self.cur_dir.to_string_lossy(), font(11.5), TEXT);
    }

    fn modals(&mut self, ctx: &egui::Context) {
        let Some(m) = self.modal.clone() else { return };
        let mut close = false;
        let mut next: Option<Modal> = None;
        let ok = |ui: &mut egui::Ui, text: &str| ui.add(egui::Button::new(text).min_size(vec2(80.0, 24.0))).clicked();
        match m {
            Modal::Msg(title, text) => {
                egui::Window::new(title)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.set_max_width(440.0);
                        ui.add_space(6.0);
                        ui.label(text);
                        ui.add_space(10.0);
                        ui.vertical_centered(|ui| {
                            if ok(ui, "OK") {
                                close = true;
                            }
                        });
                    });
            }
            Modal::About => {
                egui::Window::new("About Bplus Rename")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(egui::RichText::new("Bplus Rename 1.0").font(bold(14.0)));
                        ui.label("Rename many files and folders at once with the numbered criteria:");
                        ui.label("RegEx, Name, Replace, Case, Remove, Move/Copy, Add, Auto Date,");
                        ui.label("Append Folder Name, Numbering and Extension, applied in that order.");
                        ui.add_space(6.0);
                        ui.label("Select items in the list, set the criteria, check the New Name column, then press Rename.");
                        ui.label("Ctrl+Z undoes the last rename.");
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            if ok(ui, "OK") {
                                close = true;
                            }
                        });
                    });
            }
            Modal::Preview => {
                let changes: Vec<(String, String, bool)> = self
                    .items
                    .iter()
                    .filter(|e| e.selected && e.new_name != e.name)
                    .map(|e| (e.name.clone(), e.new_name.clone(), engine::has_invalid(&e.new_name)))
                    .collect();
                egui::Window::new("Preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_size([620.0, 420.0])
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(format!("{} item(s) will be renamed.", changes.len()));
                        ui.separator();
                        egui::ScrollArea::vertical().max_height(320.0).auto_shrink([false, false]).show(ui, |ui| {
                            egui::Grid::new("pv").striped(true).num_columns(3).show(ui, |ui| {
                                ui.label(egui::RichText::new("Name").font(bold(12.0)));
                                ui.label("");
                                ui.label(egui::RichText::new("New Name").font(bold(12.0)));
                                ui.end_row();
                                for (a, b, bad) in changes.iter().take(2000) {
                                    ui.label(a.as_str());
                                    ui.label("→");
                                    ui.label(egui::RichText::new(b.as_str()).color(if *bad { INVALID } else { TEXT }));
                                    ui.end_row();
                                }
                            });
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ok(ui, "Rename") {
                                close = true;
                                self.pending.push(Act::Rename);
                            }
                            if ok(ui, "Close") {
                                close = true;
                            }
                        });
                    });
            }
            Modal::Order(mut order) => {
                let mut keep = true;
                egui::Window::new("Renaming Criteria Order")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label("Change and re-arrange the order of the renaming criteria.");
                        ui.add_space(4.0);
                        ui.label("Order:");
                        let mut swap: Option<(usize, usize)> = None;
                        let len = order.len();
                        for (i, id) in order.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add_sized([180.0, 20.0], egui::Label::new(crit_name(*id)));
                                if ui.add_enabled(i > 0, egui::Button::new("Up")).clicked() {
                                    swap = Some((i, i - 1));
                                }
                                if ui.add_enabled(i + 1 < len, egui::Button::new("Down")).clicked() {
                                    swap = Some((i, i + 1));
                                }
                            });
                        }
                        if let Some((a, b)) = swap {
                            order.swap(a, b);
                        }
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ok(ui, "Reset") {
                                order = (1..=11).collect();
                            }
                            if ok(ui, "OK") {
                                self.crit.order = order.clone();
                                keep = false;
                            }
                            if ok(ui, "Cancel") {
                                keep = false;
                            }
                        });
                    });
                if keep {
                    next = Some(Modal::Order(order));
                } else {
                    close = true;
                }
            }
            Modal::Translations => {
                egui::Window::new("Character Translations")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.checkbox(&mut self.crit.translations_on, "Enable character translations");
                        ui.label("Each pair replaces the text on the left with the text on the right.");
                        ui.add_space(4.0);
                        let mut remove = None;
                        for (i, (a, b)) in self.crit.translations.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(a).desired_width(120.0));
                                ui.label("=");
                                ui.add(egui::TextEdit::singleline(b).desired_width(120.0));
                                if ui.button("Delete").clicked() {
                                    remove = Some(i);
                                }
                            });
                        }
                        if let Some(i) = remove {
                            self.crit.translations.remove(i);
                        }
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button("Add").clicked() {
                                self.crit.translations.push((String::new(), String::new()));
                                self.crit.translations_on = true;
                            }
                            if ok(ui, "OK") {
                                close = true;
                            }
                        });
                    });
            }
        }
        if close {
            self.modal = None;
        } else if let Some(m) = next {
            self.modal = Some(m);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn tree_rows(
    ui: &mut egui::Ui,
    nodes: &[TreeNode],
    depth: usize,
    chain: &mut Vec<usize>,
    cur: &Path,
    scroll: bool,
    toggles: &mut Vec<Vec<usize>>,
    select: &mut Option<PathBuf>,
) {
    for (i, n) in nodes.iter().enumerate() {
        chain.push(i);
        let is_cur = n.path == cur;
        let gal = ui.painter().layout_no_wrap(n.label.clone(), if is_cur { bold(12.0) } else { font(12.0) }, TEXT);
        let x0 = 4.0 + depth as f32 * 16.0;
        let width = x0 + 38.0 + gal.size().x + 6.0;
        let (row, resp) = ui.allocate_exact_size(vec2(width.max(ui.available_width()), 19.0), Sense::click());
        let p = ui.painter().clone();
        let arrow_r = Rect::from_min_size(pos2(row.min.x + x0, row.min.y), vec2(14.0, 19.0));
        if n.has_children {
            let c = arrow_r.center();
            let col = Color32::from_rgb(0x60, 0x60, 0x60);
            if n.expanded {
                chevron(&p, c, 3.5, col);
            } else {
                let st = Stroke::new(1.0, col);
                p.line_segment([pos2(c.x - 1.5, c.y - 3.5), pos2(c.x + 2.0, c.y)], st);
                p.line_segment([pos2(c.x + 2.0, c.y), pos2(c.x - 1.5, c.y + 3.5)], st);
            }
        }
        let lab = Rect::from_min_size(pos2(row.min.x + x0 + 16.0, row.min.y + 1.0), vec2(22.0 + gal.size().x + 4.0, 17.0));
        if is_cur {
            p.rect_filled(lab, 0.0, SEL);
            p.rect_stroke(lab, 0.0, Stroke::new(1.0, SEL_EDGE), StrokeKind::Inside);
            if scroll {
                ui.scroll_to_rect(row, Some(Align::Center));
            }
        } else if resp.hovered() {
            p.rect_filled(lab, 0.0, HOVER);
        }
        folder_icon(&p, pos2(lab.min.x + 2.0, lab.min.y + 2.0));
        p.galley(pos2(lab.min.x + 22.0, row.center().y - gal.size().y / 2.0), gal, TEXT);
        if resp.clicked() {
            let pos = resp.interact_pointer_pos().unwrap_or_default();
            if n.has_children && arrow_r.contains(pos) {
                toggles.push(chain.clone());
            } else {
                *select = Some(n.path.clone());
            }
        }
        if resp.double_clicked() && n.has_children {
            toggles.push(chain.clone());
        }
        if n.expanded {
            if let Some(ch) = &n.children {
                tree_rows(ui, ch, depth + 1, chain, cur, scroll, toggles, select);
            }
        }
        chain.pop();
    }
}

fn node_at<'a>(nodes: &'a mut [TreeNode], chain: &[usize]) -> Option<&'a mut TreeNode> {
    let (first, rest) = chain.split_first()?;
    let n = nodes.get_mut(*first)?;
    if rest.is_empty() {
        Some(n)
    } else {
        node_at(n.children.as_mut()?.as_mut_slice(), rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rename_and_undo_on_disk() {
        let dir = std::env::temp_dir().join(format!("bplus_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for n in ["b.txt", "a.txt", "c.JPG"] {
            std::fs::write(dir.join(n), n).unwrap();
        }
        let ctx = egui::Context::default();
        // A screenshot path keeps the test from touching the saved last folder.
        let mut app = App::new(&ctx, Some(dir.clone()), Some(dir.join("unused.png")));
        assert_eq!(app.items.len(), 3);
        app.items.iter_mut().for_each(|e| e.selected = true);
        app.crit.name.mode = 2; // Fixed
        app.crit.name.text = "photo".into();
        app.crit.number.mode = 2; // Suffix
        app.crit.number.sep = "_".into();
        app.crit.number.pad = "2".into();
        app.crit.ext.mode = 1; // Lower
        app.do_rename();
        let mut names: Vec<String> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        names.sort();
        assert_eq!(names, vec!["photo_01.txt", "photo_02.txt", "photo_03.jpg"]);
        app.do_undo();
        let mut names: Vec<String> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        names.sort();
        assert_eq!(names, vec!["a.txt", "b.txt", "c.JPG"]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

fn type_label(e: &Entry) -> String {
    let ext = Path::new(&e.name).extension().map(|s| s.to_string_lossy().to_uppercase()).unwrap_or_default();
    if e.info.is_dir {
        "File folder".to_string()
    } else if ext.is_empty() {
        "File".into()
    } else {
        format!("{ext} File")
    }
}
