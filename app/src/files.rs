//! Folder listing, filters and the folder tree.

use crate::engine::{ItemInfo, wildcard_to_regex};
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct Filters {
    pub mask: String,
    pub match_case: bool,
    pub regex: bool,
    pub folders: bool,
    pub files: bool,
    pub hidden: bool,
    pub subfolders: bool,
    pub level: String,
    pub name_min: String,
    pub name_max: String,
    pub path_min: String,
    pub path_max: String,
    pub condition: String,
}

impl Default for Filters {
    fn default() -> Self {
        Filters {
            mask: "*".into(),
            match_case: false,
            regex: false,
            folders: true,
            files: true,
            hidden: false,
            subfolders: false,
            level: "0".into(),
            name_min: "0".into(),
            name_max: "0".into(),
            path_min: "0".into(),
            path_max: "0".into(),
            condition: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub info: ItemInfo,
    pub name: String,
    pub new_name: String,
    pub sub_dir: String,
    pub size: u64,
    pub hidden: bool,
    pub selected: bool,
}

fn num(s: &str) -> usize {
    s.trim().parse::<usize>().unwrap_or(0)
}

#[cfg(windows)]
fn is_hidden(meta: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x2 != 0
}
#[cfg(not(windows))]
fn is_hidden(_meta: &std::fs::Metadata) -> bool {
    false
}

fn to_local(t: std::io::Result<std::time::SystemTime>) -> Option<DateTime<Local>> {
    t.ok().map(DateTime::<Local>::from)
}

pub struct MaskMatcher {
    res: Vec<regex::Regex>,
}

impl MaskMatcher {
    pub fn new(f: &Filters) -> Self {
        let ci = if f.match_case { "" } else { "(?i)" };
        let mut res = Vec::new();
        if f.regex {
            if let Ok(r) = regex::Regex::new(&format!("{ci}{}", f.mask)) {
                res.push(r);
            }
        } else {
            for part in f.mask.split(';').map(str::trim).filter(|p| !p.is_empty()) {
                if let Ok(r) = regex::Regex::new(&format!("{ci}^{}$", wildcard_to_regex(part))) {
                    res.push(r);
                }
            }
        }
        MaskMatcher { res }
    }
    pub fn matches(&self, name: &str) -> bool {
        self.res.is_empty() || self.res.iter().any(|r| r.is_match(name))
    }
}

/// List a folder with the File/Folder Selection rules (12).
pub fn list_folder(dir: &Path, f: &Filters) -> Vec<Entry> {
    let mut out = Vec::new();
    let matcher = MaskMatcher::new(f);
    let max_level = if f.subfolders { match num(&f.level) { 0 => usize::MAX, n => n } } else { 0 };
    walk(dir, dir, 0, max_level, f, &matcher, &mut out);
    out
}

fn walk(root: &Path, dir: &Path, depth: usize, max_level: usize, f: &Filters, m: &MaskMatcher, out: &mut Vec<Entry>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut items: Vec<(PathBuf, std::fs::Metadata)> = rd
        .flatten()
        .filter_map(|e| e.metadata().ok().map(|m| (e.path(), m)))
        .collect();
    // Folders first, then files, each in logical (case-insensitive) order.
    items.sort_by(|a, b| {
        b.1.is_dir().cmp(&a.1.is_dir()).then_with(|| {
            let an = a.0.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            let bn = b.0.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            an.cmp(&bn)
        })
    });
    let sub_dir = dir.strip_prefix(root).map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
    let mut subdirs = Vec::new();
    for (path, meta) in items {
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let hidden = is_hidden(&meta);
        if hidden && !f.hidden {
            continue;
        }
        let is_dir = meta.is_dir();
        if is_dir && depth < max_level {
            subdirs.push(path.clone());
        }
        if (is_dir && !f.folders) || (!is_dir && !f.files) {
            continue;
        }
        if !matcher_ok(m, &name, is_dir) {
            continue;
        }
        let nl = name.chars().count();
        let pl = path.to_string_lossy().chars().count();
        if num(&f.name_min) > 0 && nl < num(&f.name_min) { continue; }
        if num(&f.name_max) > 0 && nl > num(&f.name_max) { continue; }
        if num(&f.path_min) > 0 && pl < num(&f.path_min) { continue; }
        if num(&f.path_max) > 0 && pl > num(&f.path_max) { continue; }
        out.push(Entry {
            info: ItemInfo {
                path: path.clone(),
                is_dir,
                created: to_local(meta.created()),
                modified: to_local(meta.modified()),
                accessed: to_local(meta.accessed()),
            },
            new_name: name.clone(),
            name,
            sub_dir: sub_dir.clone(),
            size: if is_dir { 0 } else { meta.len() },
            hidden,
            selected: false,
        });
    }
    for sd in subdirs {
        walk(root, &sd, depth + 1, max_level, f, m, out);
    }
}

fn matcher_ok(m: &MaskMatcher, name: &str, is_dir: bool) -> bool {
    // Folders are listed regardless of an extension mask such as *.jpg.
    is_dir || m.matches(name)
}

// ---------------- Folder tree ----------------

pub struct TreeNode {
    pub path: PathBuf,
    pub label: String,
    pub children: Option<Vec<TreeNode>>,
    pub expanded: bool,
    pub has_children: bool,
}

impl TreeNode {
    pub fn new(path: PathBuf, label: String) -> Self {
        let has_children = std::fs::read_dir(&path)
            .map(|rd| rd.flatten().any(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false)))
            .unwrap_or(false);
        TreeNode { path, label, children: None, expanded: false, has_children }
    }

    pub fn load(&mut self) {
        if self.children.is_some() {
            return;
        }
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(&self.path)
            .map(|rd| {
                rd.flatten()
                    .filter(|e| e.metadata().map(|m| m.is_dir() && !is_hidden(&m)).unwrap_or(false))
                    .map(|e| e.path())
                    .collect()
            })
            .unwrap_or_default();
        dirs.sort_by_key(|p| p.file_name().unwrap_or_default().to_string_lossy().to_lowercase());
        self.children = Some(
            dirs.into_iter()
                .map(|p| {
                    let l = p.file_name().unwrap_or_default().to_string_lossy().into_owned();
                    TreeNode::new(p, l)
                })
                .collect(),
        );
        self.has_children = !self.children.as_ref().unwrap().is_empty();
    }

    /// Expand the nodes leading to `target`.
    pub fn reveal(&mut self, target: &Path) {
        if target.starts_with(&self.path) && target != self.path {
            self.load();
            self.expanded = true;
            if let Some(ch) = &mut self.children {
                for c in ch.iter_mut() {
                    if target.starts_with(&c.path) {
                        c.reveal(target);
                    }
                }
            }
        }
    }
}

pub fn drive_roots() -> Vec<TreeNode> {
    let mut v = Vec::new();
    #[cfg(windows)]
    for l in b'A'..=b'Z' {
        let p = PathBuf::from(format!("{}:\\", l as char));
        if p.exists() {
            v.push(TreeNode::new(p, format!("({}:)", l as char)));
        }
    }
    #[cfg(not(windows))]
    v.push(TreeNode::new(PathBuf::from("/"), "/".into()));
    v
}

pub fn human_size(b: u64) -> String {
    if b < 1024 {
        format!("{b} B")
    } else if b < 1024 * 1024 {
        format!("{:.0} KB", (b as f64 / 1024.0).ceil())
    } else if b < 1024 * 1024 * 1024 {
        format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", b as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
