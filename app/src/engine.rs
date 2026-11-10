//! The renaming rules. Each criterion turns (name, extension) into a new (name, extension).
//! Criteria are applied in a fixed default order with their own defaults.

use chrono::{DateTime, Datelike, Local, Timelike};
use std::path::PathBuf;

pub const INVALID_CHARS: &str = "/:*\\?\"<>|";

pub const NAME_MODES: [&str; 6] = ["Keep", "Remove", "Fixed", "Reverse", "Pad Numbers", "Reformat Date"];
pub const CASE_MODES: [&str; 6] = ["Same", "Lower", "Upper", "Title", "Sentence", "Title Enhanced"];
pub const CROP_MODES: [&str; 3] = ["Before", "After", "Special"];
pub const LEAD_DOTS: [&str; 4] = ["None", ".", "..", "Both"];
pub const MC_SOURCE: [&str; 7] = ["None", "Copy first n", "Copy last n", "Move first n", "Move last n", "Copy range", "Move range"];
pub const MC_TARGET: [&str; 4] = ["None", "To start", "To end", "To pos."];
pub const DATE_MODES: [&str; 3] = ["None", "Prefix", "Suffix"];
pub const DATE_TYPES: [&str; 12] = [
    "Creation (Curr.)", "Creation (New)", "Modified (Curr.)", "Modified (New)", "Accessed (Curr.)", "Accessed (New)",
    "Taken (Original)", "Taken (Digitized)", "Taken (Modified)", "Taken (Recent)", "Current", "Item Date",
];
pub const DATE_FORMATS: [&str; 10] = ["DMY", "DMY HM", "DMY HMS", "MDY", "MDY HM", "MDY HMS", "YMD", "YMD HM", "YMD HMS", "Custom"];
pub const FOLDER_MODES: [&str; 3] = ["None", "Prefix", "Suffix"];
pub const NUM_MODES: [&str; 5] = ["None", "Prefix", "Suffix", "Pre.+Suff.", "Insert"];
pub const NUM_CASE: [&str; 3] = [" ", "Lower", "Upper"];
pub const EXT_MODES: [&str; 7] = ["Same", "Lower", "Upper", "Title", "Fixed", "Extra", "Remove"];

pub fn num_types() -> Vec<String> {
    let mut v = Vec::new();
    for b in 2..=36 {
        v.push(match b {
            2 => "Base 2 (Binary)".to_string(),
            8 => "Base 8 (Octal)".to_string(),
            10 => "Base 10 (Decimal)".to_string(),
            16 => "Base 16 (Hex)".to_string(),
            _ => format!("Base {b}"),
        });
    }
    v.extend(["A-Z", "a-z", "Roman Numerals", "Hebrew Numerals"].map(String::from));
    v
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegExC { pub on: bool, pub matcher: String, pub replace: String, pub inc_ext: bool, pub simple: bool, pub v2: bool }
#[derive(Clone, Debug, PartialEq)]
pub struct NameC { pub on: bool, pub mode: usize, pub text: String }
#[derive(Clone, Debug, PartialEq)]
pub struct ReplaceC { pub on: bool, pub find: String, pub with: String, pub match_case: bool, pub first: bool }
#[derive(Clone, Debug, PartialEq)]
pub struct CaseC { pub on: bool, pub mode: usize, pub except: String }
#[derive(Clone, Debug, PartialEq)]
pub struct RemoveC {
    pub on: bool, pub first_n: String, pub last_n: String, pub from: String, pub to: String,
    pub chars: String, pub words: String, pub crop: usize, pub crop_marker: String,
    pub digits: bool, pub high: bool, pub trim: bool, pub ds: bool, pub accents: bool, pub alpha: bool, pub sym: bool,
    pub lead_dots: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MoveCopyC { pub on: bool, pub source: usize, pub count: String, pub range_to: String, pub target: usize, pub pos: String, pub sep: String }
#[derive(Clone, Debug, PartialEq)]
pub struct AddC { pub on: bool, pub prefix: String, pub insert: String, pub at: String, pub suffix: String, pub word_space: bool }
#[derive(Clone, Debug, PartialEq)]
pub struct AutoDateC { pub on: bool, pub mode: usize, pub kind: usize, pub fmt: usize, pub sep: String, pub seg: String, pub custom: String, pub century: bool, pub offset: String }
#[derive(Clone, Debug, PartialEq)]
pub struct FolderC { pub on: bool, pub mode: usize, pub sep: String, pub levels: String }
#[derive(Clone, Debug, PartialEq)]
pub struct NumberC { pub on: bool, pub mode: usize, pub at: String, pub start: String, pub incr: String, pub pad: String, pub sep: String, pub brk: String, pub folder: bool, pub kind: usize, pub case: usize }
#[derive(Clone, Debug, PartialEq)]
pub struct ExtC { pub on: bool, pub mode: usize, pub text: String }
#[derive(Clone, Debug, PartialEq)]
pub struct SegmentC { pub on: bool, pub from: String, pub to: String }

#[derive(Clone, Debug, PartialEq)]
pub struct Criteria {
    pub regex: RegExC,
    pub name: NameC,
    pub replace: ReplaceC,
    pub case: CaseC,
    pub remove: RemoveC,
    pub movecopy: MoveCopyC,
    pub add: AddC,
    pub date: AutoDateC,
    pub folder: FolderC,
    pub number: NumberC,
    pub ext: ExtC,
    pub segment: SegmentC,
    /// Criteria ids 1..=11 in the order they are applied.
    pub order: Vec<u8>,
    /// Special (14): character translations, "from=to" pairs.
    pub translations: Vec<(String, String)>,
    pub translations_on: bool,
}

impl Default for Criteria {
    fn default() -> Self {
        Criteria {
            regex: RegExC { on: true, matcher: String::new(), replace: String::new(), inc_ext: false, simple: false, v2: true },
            name: NameC { on: true, mode: 0, text: String::new() },
            replace: ReplaceC { on: true, find: String::new(), with: String::new(), match_case: false, first: false },
            case: CaseC { on: true, mode: 0, except: String::new() },
            remove: RemoveC {
                on: true, first_n: "0".into(), last_n: "0".into(), from: "0".into(), to: "0".into(),
                chars: String::new(), words: String::new(), crop: 0, crop_marker: String::new(),
                digits: false, high: false, trim: false, ds: false, accents: false, alpha: false, sym: false, lead_dots: 0,
            },
            movecopy: MoveCopyC { on: true, source: 0, count: "1".into(), range_to: "0".into(), target: 0, pos: "1".into(), sep: String::new() },
            add: AddC { on: true, prefix: String::new(), insert: String::new(), at: "0".into(), suffix: String::new(), word_space: false },
            date: AutoDateC { on: true, mode: 0, kind: 0, fmt: 0, sep: String::new(), seg: String::new(), custom: String::new(), century: false, offset: "0".into() },
            folder: FolderC { on: true, mode: 0, sep: String::new(), levels: "1".into() },
            number: NumberC { on: true, mode: 0, at: "0".into(), start: "1".into(), incr: "1".into(), pad: "0".into(), sep: String::new(), brk: "0".into(), folder: false, kind: 8, case: 0 },
            ext: ExtC { on: true, mode: 0, text: String::new() },
            segment: SegmentC { on: true, from: "0".into(), to: "0".into() },
            order: (1..=11).collect(),
            translations: Vec::new(),
            translations_on: false,
        }
    }
}

/// What the engine needs to know about one listed item.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemInfo {
    pub path: PathBuf,
    pub is_dir: bool,
    pub created: Option<DateTime<Local>>,
    pub modified: Option<DateTime<Local>>,
    pub accessed: Option<DateTime<Local>>,
}

fn int(s: &str) -> i64 {
    s.trim().parse::<i64>().unwrap_or(0)
}

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

/// Split a file name into (stem, extension-without-dot). Folders keep their whole name.
pub fn split_name(file_name: &str, is_dir: bool) -> (String, String) {
    if is_dir {
        return (file_name.to_string(), String::new());
    }
    match file_name.rfind('.') {
        Some(i) if i > 0 => (file_name[..i].to_string(), file_name[i + 1..].to_string()),
        _ => (file_name.to_string(), String::new()),
    }
}

pub fn join_name(stem: &str, ext: &str) -> String {
    if ext.is_empty() { stem.to_string() } else { format!("{stem}.{ext}") }
}

/// Numbering state carried across the selected items, in list order:
/// the count starts at Start, goes up by Incr. per item, and restarts at Start when the item's folder
/// changes (Folder ticked) or when the first Break characters of the new name differ from the
/// previous item's new name.
pub struct Counter {
    pub n: i64,
    pub prev_name: Option<String>,
    pub prev_folder: Option<PathBuf>,
    pub started: bool,
}

impl Counter {
    pub fn new() -> Self {
        Counter { n: 0, prev_name: None, prev_folder: None, started: false }
    }
}

impl Criteria {
    /// Is any renaming criterion changing names at all?
    pub fn is_active(&self) -> bool {
        let c = self;
        (c.regex.on && !c.regex.matcher.is_empty())
            || (c.name.on && c.name.mode != 0)
            || (c.replace.on && !c.replace.find.is_empty())
            || (c.case.on && c.case.mode != 0)
            || (c.remove.on && remove_active(&c.remove))
            || (c.movecopy.on && c.movecopy.source != 0 && c.movecopy.target != 0)
            || (c.add.on && (!c.add.prefix.is_empty() || !c.add.insert.is_empty() || !c.add.suffix.is_empty() || c.add.word_space))
            || (c.date.on && c.date.mode != 0)
            || (c.folder.on && c.folder.mode != 0)
            || (c.number.on && c.number.mode != 0)
            || (c.ext.on && c.ext.mode != 0)
            || (c.translations_on && !c.translations.is_empty())
    }

    /// The new file name of one item. `counter` advances once per call.
    pub fn new_name(&self, item: &ItemInfo, counter: &mut Counter) -> String {
        let start = int(&self.number.start);
        let folder = item.path.parent().map(|p| p.to_path_buf());
        if !counter.started || (self.number.on && self.number.folder && counter.prev_folder != folder) {
            counter.n = start;
        }
        let mut name = self.build(item, counter.n);
        let brk = int(&self.number.brk).max(0) as usize;
        if self.number.on && brk > 0 {
            if let Some(prev) = &counter.prev_name {
                let a: String = name.chars().take(brk).collect();
                let b: String = prev.chars().take(brk).collect();
                if a != b && counter.n != start {
                    counter.n = start;
                    name = self.build(item, counter.n);
                }
            }
        }
        counter.prev_name = Some(name.clone());
        counter.prev_folder = folder;
        counter.started = true;
        counter.n += int(&self.number.incr);
        name
    }

    /// Apply the criteria in order (criteria 1..11).
    fn build(&self, item: &ItemInfo, number: i64) -> String {
        let file_name = item.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let (stem0, ext0) = split_name(&file_name, item.is_dir);

        // Name Segment: criteria only touch characters From..To of the name.
        let (pre, mut stem, post) = {
            let from = int(&self.segment.from);
            let to = int(&self.segment.to);
            let cs = chars(&stem0);
            if self.segment.on && from > 0 && to >= from && (from as usize) <= cs.len() {
                let a = from as usize - 1;
                let b = (to as usize).min(cs.len());
                (cs[..a].iter().collect::<String>(), cs[a..b].iter().collect::<String>(), cs[b..].iter().collect::<String>())
            } else {
                (String::new(), stem0.clone(), String::new())
            }
        };
        let mut ext = ext0.clone();

        for id in &self.order {
            match id {
                1 if self.regex.on => {
                    if self.regex.inc_ext && !ext.is_empty() {
                        let whole = apply_regex(&self.regex, &join_name(&stem, &ext));
                        let (s, e) = split_name(&whole, false);
                        stem = s;
                        ext = e;
                    } else {
                        stem = apply_regex(&self.regex, &stem);
                    }
                }
                2 if self.name.on => stem = apply_name(&self.name, &stem),
                3 if self.replace.on => stem = apply_replace(&self.replace, &stem),
                4 if self.case.on => stem = apply_case(self.case.mode, &self.case.except, &stem),
                5 if self.remove.on => stem = apply_remove(&self.remove, &stem),
                6 if self.movecopy.on => stem = apply_movecopy(&self.movecopy, &stem),
                7 if self.add.on => stem = apply_add(&self.add, &stem),
                8 if self.date.on => stem = apply_date(&self.date, item, &stem),
                9 if self.folder.on => stem = apply_folder(&self.folder, item, &stem),
                10 if self.number.on => stem = apply_number(&self.number, &stem, number),
                11 if self.ext.on => ext = apply_ext(&self.ext, &ext),
                _ => {}
            }
            // Character Translations run right after Replace (3).
            if *id == 3 && self.translations_on {
                for (a, b) in &self.translations {
                    if !a.is_empty() {
                        stem = stem.replace(a.as_str(), b);
                    }
                }
            }
        }
        join_name(&format!("{pre}{stem}{post}"), &ext)
    }
}

fn remove_active(r: &RemoveC) -> bool {
    int(&r.first_n) > 0 || int(&r.last_n) > 0 || int(&r.from) > 0 || !r.chars.is_empty() || !r.words.is_empty()
        || !r.crop_marker.is_empty() || r.digits || r.high || r.trim || r.ds || r.accents || r.alpha || r.sym || r.lead_dots != 0
}

// ---------------- (1) RegEx ----------------

/// `*` and `?` wildcards to a regular expression, used by "Simple" and by the filter mask.
pub fn wildcard_to_regex(w: &str) -> String {
    let mut out = String::new();
    for c in w.chars() {
        match c {
            '*' => out.push_str("(.*)"),
            '?' => out.push_str("(.)"),
            _ => out.push_str(&regex::escape(&c.to_string())),
        }
    }
    out
}

/// `\1` style back-references become `${1}`.
fn convert_replacement(r: &str) -> String {
    let mut out = String::new();
    let cs: Vec<char> = r.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '\\' && i + 1 < cs.len() && cs[i + 1].is_ascii_digit() {
            out.push_str(&format!("${{{}}}", cs[i + 1]));
            i += 2;
        } else if cs[i] == '$' && !(i + 1 < cs.len() && (cs[i + 1].is_ascii_digit() || cs[i + 1] == '{')) {
            out.push_str("$$");
            i += 1;
        } else {
            out.push(cs[i]);
            i += 1;
        }
    }
    out
}

/// Simple mode: regex symbols are escaped,
/// `%0`..`%9` become lazy groups (the last one used is greedy) and `%n` in the replacement is group n.
fn simple_pattern(m: &str) -> String {
    let meta = "-[]/{}()*+?.\\^$|";
    let mut p = String::new();
    for c in m.chars() {
        if meta.contains(c) {
            p.push('\\');
        }
        p.push(c);
    }
    let mut last = None;
    for d in 0..10 {
        let tok = format!("%{d}");
        if p.contains(&tok) {
            p = p.replace(&tok, &format!("(?<g{d}>.+?)"));
            last = Some(d);
        }
    }
    if let Some(d) = last {
        p = p.replace(&format!("(?<g{d}>.+?)"), &format!("(?<g{d}>.+)"));
    }
    p
}

fn apply_regex(c: &RegExC, s: &str) -> String {
    if c.matcher.is_empty() {
        return s.to_string();
    }
    let (pat, rep) = if c.simple {
        let mut r = c.replace.replace('$', "$$");
        for d in 0..10 {
            r = r.replace(&format!("%{d}"), &format!("${{g{d}}}"));
        }
        (simple_pattern(&c.matcher), r)
    } else {
        (c.matcher.clone(), convert_replacement(&c.replace))
    };
    if c.v2 && !c.simple {
        match regex::Regex::new(&pat) {
            Ok(re) => re.replace_all(s, rep.as_str()).into_owned(),
            Err(_) => s.to_string(),
        }
    } else {
        // Without v2 (and in Simple mode) the expression has to match the whole name.
        match regex::Regex::new(&format!("^(?:{pat})$")) {
            Ok(re) if re.is_match(s) => re.replace(s, rep.as_str()).into_owned(),
            _ => s.to_string(),
        }
    }
}

// ---------------- (2) Name ----------------

fn apply_name(c: &NameC, s: &str) -> String {
    match c.mode {
        1 => String::new(),
        2 => c.text.clone(),
        3 => s.chars().rev().collect(),
        4 => {
            let width = int(&c.text).max(0) as usize;
            if width == 0 {
                return s.to_string();
            }
            let mut out = String::new();
            let mut run = String::new();
            for ch in s.chars() {
                if ch.is_ascii_digit() {
                    run.push(ch);
                } else {
                    if !run.is_empty() {
                        out.push_str(&format!("{:0>width$}", run));
                        run.clear();
                    }
                    out.push(ch);
                }
            }
            if !run.is_empty() {
                out.push_str(&format!("{:0>width$}", run));
            }
            out
        }
        5 => reformat_date(s, &c.text),
        _ => s.to_string(),
    }
}

/// Finds a date written as YYYY-MM-DD, YYYYMMDD or DD-MM-YYYY and writes it again with `pattern`
/// (strftime style, e.g. "%d.%m.%Y").
fn reformat_date(s: &str, pattern: &str) -> String {
    if pattern.is_empty() {
        return s.to_string();
    }
    let pats = [
        (r"(\d{4})[-_. ]?(\d{2})[-_. ]?(\d{2})", [1usize, 2, 3]),
        (r"(\d{2})[-_. ](\d{2})[-_. ](\d{4})", [3, 2, 1]),
    ];
    for (p, idx) in pats {
        let re = regex::Regex::new(p).unwrap();
        if let Some(caps) = re.captures(s) {
            let y: i32 = caps[idx[0]].parse().unwrap_or(0);
            let m: u32 = caps[idx[1]].parse().unwrap_or(0);
            let d: u32 = caps[idx[2]].parse().unwrap_or(0);
            if let Some(date) = chrono::NaiveDate::from_ymd_opt(y, m, d) {
                let mut out = String::new();
                use std::fmt::Write;
                if write!(out, "{}", date.format(pattern)).is_err() {
                    return s.to_string();
                }
                let whole = caps.get(0).unwrap();
                return format!("{}{}{}", &s[..whole.start()], out, &s[whole.end()..]);
            }
        }
    }
    s.to_string()
}

// ---------------- (3) Replace ----------------

fn apply_replace(c: &ReplaceC, s: &str) -> String {
    if c.find.is_empty() {
        return s.to_string();
    }
    let mut pat = regex::escape(&c.find);
    if !c.match_case {
        pat = format!("(?i){pat}");
    }
    let re = match regex::Regex::new(&pat) {
        Ok(r) => r,
        Err(_) => return s.to_string(),
    };
    let with = regex::NoExpand(c.with.as_str());
    if c.first { re.replacen(s, 1, with).into_owned() } else { re.replace_all(s, with).into_owned() }
}

// ---------------- (4) Case ----------------

/// Title: every character lowered, then raised when the character before it
/// (start counts as a tab) is a tab, newline, space, ( ) . { [ - or _.
fn title_case(s: &str) -> String {
    let mut out = String::new();
    let mut prev = '\t';
    for ch in s.chars() {
        let lower: String = ch.to_lowercase().collect();
        let piece: String = if matches!(prev, '\n' | '\t' | ' ' | '(' | ')' | '.' | '{' | '[' | '-' | '_') {
            lower.to_uppercase()
        } else {
            lower
        };
        prev = piece.chars().last().unwrap_or(ch);
        out.push_str(&piece);
    }
    out
}

/// Sentence: everything lowered; the first letter at the start and after each "." is raised.
fn sentence_case(s: &str) -> String {
    let mut out = String::new();
    let mut cap = true;
    for ch in s.chars() {
        let lower: String = ch.to_lowercase().collect();
        let first = lower.chars().next().unwrap_or(ch);
        if cap && (first.is_ascii_alphabetic() || (first as u32) >= 0x80) {
            out.extend(lower.to_uppercase().chars());
            cap = false;
        } else {
            out.push_str(&lower);
        }
        if first == '.' {
            cap = true;
        }
    }
    out
}

/// Raise the first character that has an upper-case form.
fn capitalise_word(w: &str) -> String {
    let mut out = String::new();
    let mut done = false;
    for ch in w.chars() {
        if !done {
            let up: String = ch.to_uppercase().collect();
            if up != ch.to_string() {
                out.push_str(&up);
                done = true;
                continue;
            }
            if ch.is_uppercase() {
                done = true;
            }
        }
        out.push(ch);
    }
    out
}

/// Title Enhanced: words are split at ? : space , _ . ! -; a word already in capitals
/// stays; words in the always-lower list stay lower except the first word and one after . ? ! :;
/// words in the always-upper list become capitals; every other word is lowered and capitalised.
/// The two lists come from preferences and are empty here.
fn title_enhanced(s: &str) -> String {
    const SEP: &[char] = &['?', ':', ' ', ',', '_', '.', '!', '-'];
    const ALWAYS_LOWER: &[&str] = &[];
    const ALWAYS_UPPER: &[&str] = &[];
    let mut out = String::new();
    let mut word = String::new();
    let mut force = true;
    let flush = |word: &mut String, out: &mut String, force: &mut bool| {
        if word.is_empty() {
            return;
        }
        let upper = word.to_uppercase();
        if *word == upper && word.chars().any(|c| c.is_alphabetic()) {
            out.push_str(word);
        } else {
            let lower = word.to_lowercase();
            if ALWAYS_UPPER.contains(&lower.as_str()) {
                out.push_str(&upper);
            } else if !*force && ALWAYS_LOWER.contains(&lower.as_str()) {
                out.push_str(&lower);
            } else {
                out.push_str(&capitalise_word(&lower));
            }
        }
        *force = false;
        word.clear();
    };
    for ch in s.chars() {
        if SEP.contains(&ch) {
            flush(&mut word, &mut out, &mut force);
            out.push(ch);
            if matches!(ch, '.' | '?' | '!' | ':') {
                force = true;
            }
        } else {
            word.push(ch);
        }
    }
    flush(&mut word, &mut out, &mut force);
    out
}

pub fn apply_case(mode: usize, except: &str, s: &str) -> String {
    let out = match mode {
        1 => s.to_lowercase(),
        2 => s.to_uppercase(),
        3 => title_case(s),
        4 => sentence_case(s),
        5 => title_enhanced(s),
        _ => s.to_string(),
    };
    if mode == 0 || except.trim().is_empty() {
        return out;
    }
    // Exceptions: words that keep the case as typed in the box.
    let mut out = out;
    for word in except.split(':').map(str::trim).filter(|w| !w.is_empty()) {
        let re = match regex::Regex::new(&format!(r"(?i)\b{}\b", regex::escape(word))) {
            Ok(r) => r,
            Err(_) => continue,
        };
        out = re.replace_all(&out, regex::NoExpand(word)).into_owned();
    }
    out
}

// ---------------- (5) Remove ----------------

/// The symbols "Sym." removes.
pub const SYMBOLS: &str = "!£$%^&*()-_=+[]{};:'@#~,.<>/?\\|`¬¦";

/// Remove characters From..To: 1-based, inclusive; negative counts from the end,
/// To = 0 means "to the end", From = 0 does nothing, a positive To below From does nothing.
fn remove_range(s: &str, from: i64, to: i64) -> String {
    let cs = chars(s);
    let len = cs.len() as i64;
    if from == 0 || len == 0 {
        return s.to_string();
    }
    if from > 0 && to > 0 && to < from {
        return s.to_string();
    }
    let mut a = if from < 1 { (from + 1 + len).max(1) } else { from };
    let mut b = if to < 1 { (to + 1 + len).max(1) } else { to };
    if a > b {
        std::mem::swap(&mut a, &mut b);
    }
    let a = (a - 1).clamp(0, len) as usize;
    let b = (b.clamp(0, len) as usize).max(a);
    cs[..a].iter().chain(cs[b..].iter()).collect()
}

/// Crop, case-sensitive. Before: keep the marker and what follows. After: keep up to and
/// including the marker. Special: the marker is "left*right" and that whole span is removed.
fn crop(s: &str, mode: usize, marker: &str) -> String {
    match mode {
        0 => s.find(marker).map(|i| s[i..].to_string()).unwrap_or_else(|| s.to_string()),
        1 => s.find(marker).map(|i| s[..i + marker.len()].to_string()).unwrap_or_else(|| s.to_string()),
        _ => {
            let Some(star) = marker.find('*') else { return s.to_string() };
            let (left, right) = (&marker[..star], &marker[star + 1..]);
            let start = if left.is_empty() { Some(0) } else { s.find(left) };
            let Some(start) = start else { return s.to_string() };
            let after = start + left.len();
            let end = if right.is_empty() { Some(s.len()) } else { s[after..].find(right).map(|j| after + j + right.len()) };
            match end {
                Some(end) => format!("{}{}", &s[..start], &s[end..]),
                None => s.to_string(),
            }
        }
    }
}

/// Lead Dots: remove a leading marker when the character after it is not another dot.
fn lead_dot(s: &str, marker: &str) -> String {
    if s.len() > marker.len() && s.starts_with(marker) && !s[marker.len()..].starts_with('.') {
        s[marker.len()..].to_string()
    } else {
        s.to_string()
    }
}

/// Accents: Windows FoldStringW(MAP_COMPOSITE) and keep the base character.
#[cfg(windows)]
fn fold_accent(c: char) -> char {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn FoldStringW(flags: u32, src: *const u16, src_len: i32, dst: *mut u16, dst_len: i32) -> i32;
    }
    let mut buf = [0u16; 2];
    let src = c.encode_utf16(&mut buf);
    if src.len() != 1 {
        return c;
    }
    let mut dst = [0u16; 4];
    // SAFETY: both buffers are valid for the lengths given.
    let n = unsafe { FoldStringW(0x40, src.as_ptr(), 1, dst.as_mut_ptr(), 4) };
    if n >= 1 { char::from_u32(dst[0] as u32).unwrap_or(c) } else { c }
}
#[cfg(not(windows))]
fn fold_accent(c: char) -> char {
    c
}

fn apply_remove(c: &RemoveC, s: &str) -> String {
    let mut cs = chars(s);
    let first = int(&c.first_n).max(0) as usize;
    if first > 0 {
        cs.drain(..first.min(cs.len()));
    }
    let last = int(&c.last_n).max(0) as usize;
    if last > 0 {
        let keep = cs.len().saturating_sub(last);
        cs.truncate(keep);
    }
    let mut out: String = cs.into_iter().collect();
    out = remove_range(&out, int(&c.from), int(&c.to));
    if let Some(first) = c.chars.chars().next() {
        if first == '<' || first == '>' {
            // "<X" removes characters below X, ">Y" above Y, both together the ones between.
            let cc: Vec<char> = c.chars.chars().collect();
            let mut lo = None;
            let mut hi = None;
            for i in 0..cc.len().saturating_sub(1) {
                if cc[i] == '<' {
                    lo = Some(cc[i + 1]);
                }
                if cc[i] == '>' {
                    hi = Some(cc[i + 1]);
                }
            }
            out = out
                .chars()
                .filter(|&ch| match (hi, lo) {
                    (Some(h), Some(l)) => !(ch > h && ch < l),
                    (Some(h), None) => ch <= h,
                    (None, Some(l)) => ch >= l,
                    _ => true,
                })
                .collect();
        } else {
            out = out.chars().filter(|ch| !c.chars.contains(*ch)).collect();
        }
    }
    for w in c.words.split_whitespace() {
        out = out.replace(w, "");
    }
    if !c.crop_marker.is_empty() {
        out = crop(&out, c.crop, &c.crop_marker);
    }
    if c.digits {
        out.retain(|ch| !ch.is_ascii_digit());
    }
    if c.high {
        out.retain(|ch| !(0x80..=0xFF).contains(&(ch as u32)));
    }
    if c.ds {
        while out.contains("  ") {
            out = out.replace("  ", " ");
        }
    }
    if c.accents {
        out = out.chars().map(fold_accent).collect();
    }
    if c.alpha {
        out.retain(|ch| !ch.is_ascii_alphabetic());
    }
    if c.sym {
        out.retain(|ch| {
            let u = ch as u32;
            !SYMBOLS.contains(ch)
                && !(0xA0..=0xFF).contains(&u)
                && !(0x2000..=0x206F).contains(&u)
                && !(0x2190..=0x21FF).contains(&u)
                && !(0x2600..=0x27BF).contains(&u)
        });
    }
    if c.trim {
        out = out.trim().to_string();
    }
    match c.lead_dots {
        1 => out = lead_dot(&out, "."),
        2 => out = lead_dot(&out, ".."),
        3 => out = lead_dot(&lead_dot(&out, ".."), "."),
        _ => {}
    }
    out
}

// ---------------- (6) Move/Copy Parts ----------------

/// Move/Copy Parts (6). "To pos." puts the separator on both sides.
fn apply_movecopy(c: &MoveCopyC, s: &str) -> String {
    let n = int(&c.count);
    if c.source == 0 || n == 0 {
        return s.to_string();
    }
    let cs = chars(s);
    let len = cs.len();
    let in_range = n > 0 && (n as usize) <= len;
    let nn = n.max(0) as usize;
    let (part, rest): (String, Vec<char>) = match c.source {
        1 if in_range => (cs[..nn].iter().collect(), cs.clone()),
        2 if in_range => (cs[len - nn..].iter().collect(), cs.clone()),
        3 if in_range => (cs[..nn].iter().collect(), cs[nn..].to_vec()),
        4 if in_range => (cs[len - nn..].iter().collect(), cs[..len - nn].to_vec()),
        5 | 6 => {
            let from = (n.max(1) as usize - 1).min(len);
            let to = (int(&c.range_to).max(0) as usize).min(len).max(from);
            let part: String = cs[from..to].iter().collect();
            if c.source == 6 {
                (part, cs[..from].iter().chain(cs[to..].iter()).copied().collect())
            } else {
                (part, cs.clone())
            }
        }
        _ => (String::new(), cs.clone()),
    };
    let rest_s: String = rest.iter().collect();
    match c.target {
        1 => format!("{part}{}{rest_s}", c.sep),
        2 => format!("{rest_s}{}{part}", c.sep),
        3 => {
            let rl = rest.len() as i64;
            let mut pos = int(&c.pos);
            if pos < 0 {
                pos = (rl + 1 + pos).max(1);
            }
            if pos > rl {
                pos = rl + 1;
            }
            let at = (pos.max(1) - 1) as usize;
            let left: String = rest[..at].iter().collect();
            let right: String = rest[at..].iter().collect();
            format!("{left}{sep}{part}{sep}{right}", sep = c.sep)
        }
        _ => s.to_string(),
    }
}

// ---------------- (7) Add ----------------

/// Insert `text` before character index `idx` (clamped).
fn insert_at(s: &str, text: &str, idx: i64) -> String {
    let cs = chars(s);
    let p = idx.clamp(0, cs.len() as i64) as usize;
    let left: String = cs[..p].iter().collect();
    let right: String = cs[p..].iter().collect();
    format!("{left}{text}{right}")
}

/// Add (7): Prefix first, then Insert (only when "at pos." isn't 0: positive is 1-based,
/// negative counts from the end), then Suffix, then Word Space.
fn apply_add(c: &AddC, s: &str) -> String {
    let mut out = format!("{}{}", c.prefix, s);
    let at = int(&c.at);
    if !c.insert.is_empty() && at != 0 {
        let idx = if at > 0 { at - 1 } else { out.chars().count() as i64 + at };
        out = insert_at(&out, &c.insert, idx);
    }
    out.push_str(&c.suffix);
    if c.word_space {
        let mut w = String::new();
        let mut prev: Option<char> = None;
        for (i, ch) in out.chars().enumerate() {
            if i > 0 && ch.is_uppercase() && prev != Some(' ') {
                w.push(' ');
            }
            w.push(ch);
            prev = Some(ch);
        }
        out = w;
    }
    out
}

// ---------------- (8) Auto Date ----------------

pub fn date_text(c: &AutoDateC, item: &ItemInfo) -> Option<String> {
    let base = match c.kind {
        0 | 1 => item.created,
        2 | 3 | 11 => item.modified,
        4 | 5 => item.accessed,
        10 => Some(Local::now()),
        _ => None, // photo dates are not read in this version
    }?;
    let dt = base + chrono::Duration::hours(int(&c.offset));
    if c.fmt == 9 {
        if c.custom.is_empty() {
            return None;
        }
        let mut out = String::new();
        use std::fmt::Write;
        write!(out, "{}", dt.format(&c.custom)).ok()?;
        return Some(out);
    }
    let y = if c.century { format!("{:04}", dt.year()) } else { format!("{:02}", dt.year() % 100) };
    let m = format!("{:02}", dt.month());
    let d = format!("{:02}", dt.day());
    let seg = &c.seg;
    let date = match c.fmt / 3 {
        0 => format!("{d}{seg}{m}{seg}{y}"),
        1 => format!("{m}{seg}{d}{seg}{y}"),
        _ => format!("{y}{seg}{m}{seg}{d}"),
    };
    let time = match c.fmt % 3 {
        1 => format!("{seg}{:02}{seg}{:02}", dt.hour(), dt.minute()),
        2 => format!("{seg}{:02}{seg}{:02}{seg}{:02}", dt.hour(), dt.minute(), dt.second()),
        _ => String::new(),
    };
    Some(format!("{date}{time}"))
}

fn apply_date(c: &AutoDateC, item: &ItemInfo, s: &str) -> String {
    if c.mode == 0 {
        return s.to_string();
    }
    match date_text(c, item) {
        Some(d) if c.mode == 1 => format!("{d}{}{s}", c.sep),
        Some(d) => format!("{s}{}{d}", c.sep),
        None => s.to_string(),
    }
}

// ---------------- (9) Append Folder Name ----------------

/// Append Folder Name (9): walks up from the item's folder,
/// joining |Levels| names with the separator (the drive is never used); a negative Levels takes only
/// the farthest of those folders.
fn apply_folder(c: &FolderC, item: &ItemInfo, s: &str) -> String {
    if c.mode == 0 {
        return s.to_string();
    }
    let levels = int(&c.levels);
    let n = levels.unsigned_abs() as usize;
    let mut names = Vec::new();
    let mut p = item.path.parent();
    while let Some(dir) = p {
        if names.len() >= n {
            break;
        }
        match dir.file_name() {
            Some(nm) => names.push(nm.to_string_lossy().into_owned()),
            None => break,
        }
        p = dir.parent();
    }
    let folder = if levels < 0 {
        if names.len() == n { names.last().cloned().unwrap_or_default() } else { String::new() }
    } else {
        names.reverse();
        names.join(&c.sep)
    };
    if folder.is_empty() {
        return s.to_string();
    }
    if c.mode == 1 { format!("{folder}{}{s}", c.sep) } else { format!("{s}{}{folder}", c.sep) }
}

// ---------------- (10) Numbering ----------------

fn to_base(mut n: i64, base: u32) -> String {
    let neg = n < 0;
    n = n.abs();
    if n == 0 {
        return "0".into();
    }
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    while n > 0 {
        out.push(digits[(n % base as i64) as usize] as char);
        n /= base as i64;
    }
    if neg {
        out.push('-');
    }
    out.iter().rev().collect()
}

/// A-Z / a-z numbering: 1 = a, 26 = z, 27 = aa.
fn to_letters(n: i64) -> String {
    let mut n = n;
    let mut out = Vec::new();
    while n > 0 {
        let mut r = n % 26;
        let mut q = n / 26;
        if r == 0 {
            r = 26;
            q -= 1;
        }
        out.push((b'a' + r as u8 - 1) as char);
        n = q;
    }
    out.iter().rev().collect()
}

/// Roman numerals: standard table, thousands repeat "M"; below 100000 only.
fn to_roman(n: i64) -> String {
    if n <= 0 || n >= 100_000 {
        return String::new();
    }
    let table = [
        (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"), (100, "C"), (90, "XC"), (50, "L"),
        (40, "XL"), (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
    ];
    let mut n = n;
    let mut out = String::new();
    for (v, s) in table {
        while n >= v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}

fn to_hebrew(n: i64) -> String {
    if n <= 0 {
        return n.to_string();
    }
    let table = [
        (400, 'ת'), (300, 'ש'), (200, 'ר'), (100, 'ק'), (90, 'צ'), (80, 'פ'), (70, 'ע'), (60, 'ס'), (50, 'נ'),
        (40, 'מ'), (30, 'ל'), (20, 'כ'), (10, 'י'), (9, 'ט'), (8, 'ח'), (7, 'ז'), (6, 'ו'), (5, 'ה'), (4, 'ד'),
        (3, 'ג'), (2, 'ב'), (1, 'א'),
    ];
    let mut n = n % 1000;
    let mut out = String::new();
    // 15 and 16 are written 9+6 and 9+7.
    for (v, c) in table {
        while n >= v {
            if n == 15 || n == 16 {
                out.push('ט');
                out.push(if n == 15 { 'ו' } else { 'ז' });
                n = 0;
                break;
            }
            out.push(c);
            n -= v;
        }
    }
    out
}

/// Number text. `kind` is the Type index (base = index + 2), `case` 0 blank, 1 Lower,
/// 2 Upper. Digits come out lower case and are raised unless Case is Lower; A-Z is raised unless Lower,
/// a-z only when Upper; Roman is lowered only when Lower. Padding prepends "0" ("a" for letters).
pub fn format_number(n: i64, kind: usize, case: usize, pad: usize) -> String {
    let mut s = match kind {
        0..=34 => to_base(n, kind as u32 + 2),
        35 | 36 => to_letters(n),
        37 => {
            let r = to_roman(n);
            if case == 1 { r.to_lowercase() } else { r }
        }
        _ => {
            if n <= 10000 { to_hebrew(n) } else { String::new() }
        }
    };
    if pad > 0 {
        let fill = if kind == 35 || kind == 36 { "a" } else { "0" };
        while s.chars().count() < pad {
            s.insert_str(0, fill);
        }
    }
    match kind {
        0..=34 if case != 1 => s.to_uppercase(),
        35 if case != 1 => s.to_uppercase(),
        36 if case == 2 => s.to_uppercase(),
        _ => s,
    }
}

/// Numbering placement (10). A ":" in the separator is replaced by the number and that text is
/// used as the number with no separator. Insert puts sep+number+sep after `at` characters.
fn apply_number(c: &NumberC, s: &str, value: i64) -> String {
    if c.mode == 0 {
        return s.to_string();
    }
    let mut num = format_number(value, c.kind, c.case, int(&c.pad).max(0) as usize);
    let mut sep = c.sep.clone();
    if sep.contains(':') {
        num = sep.replace(':', &num);
        sep.clear();
    }
    match c.mode {
        1 => format!("{num}{sep}{s}"),
        2 => format!("{s}{sep}{num}"),
        3 => format!("{num}{sep}{s}{sep}{num}"),
        _ => {
            let at = int(&c.at);
            if at == 0 {
                return s.to_string();
            }
            let idx = if at < 0 { s.chars().count() as i64 + at } else { at };
            insert_at(s, &format!("{sep}{num}{sep}"), idx)
        }
    }
}

// ---------------- (11) Extension ----------------

/// Extension (11): Fixed sets the box text, Extra adds ".text", Remove clears it.
fn apply_ext(c: &ExtC, ext: &str) -> String {
    match c.mode {
        1 => ext.to_lowercase(),
        2 => ext.to_uppercase(),
        3 => title_case(ext),
        4 => c.text.clone(),
        5 => {
            if ext.is_empty() {
                c.text.clone()
            } else {
                format!("{ext}.{}", c.text)
            }
        }
        6 => String::new(),
        _ => ext.to_string(),
    }
}

/// Does a name contain a character Windows refuses in file names?
pub fn has_invalid(name: &str) -> bool {
    name.chars().any(|c| INVALID_CHARS.contains(c)) || name.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(p: &str) -> ItemInfo {
        ItemInfo { path: PathBuf::from(p), is_dir: false, created: None, modified: None, accessed: None }
    }

    #[test]
    fn numbering_suffix_pad() {
        let mut c = Criteria::default();
        c.number.mode = 2;
        c.number.pad = "3".into();
        c.number.sep = "_".into();
        let mut k = Counter::new();
        assert_eq!(c.new_name(&item("C:/x/photo.jpg"), &mut k), "photo_001.jpg");
        assert_eq!(c.new_name(&item("C:/x/other.jpg"), &mut k), "other_002.jpg");
    }

    #[test]
    fn numbering_separator_placeholder() {
        let mut c = Criteria::default();
        c.number.mode = 2;
        c.number.sep = " (:)".into();
        let mut k = Counter::new();
        assert_eq!(c.new_name(&item("C:/x/a.txt"), &mut k), "a (1).txt");
    }

    #[test]
    fn case_rules() {
        assert_eq!(title_case("hello wORLD-foo_bar (x)y"), "Hello World-Foo_Bar (X)Y");
        assert_eq!(title_case("o'neil 2nd"), "O'neil 2nd");
        assert_eq!(sentence_case("hELLO. wORLD"), "Hello. World");
        assert_eq!(title_enhanced("the NASA report-v2"), "The NASA Report-V2");
    }

    #[test]
    fn remove_rules() {
        assert_eq!(remove_range("abcdef", 3, 0), "ab");
        assert_eq!(remove_range("abcdef", 2, 3), "adef");
        assert_eq!(remove_range("abcdef", 4, 2), "abcdef");
        assert_eq!(remove_range("abcdef", -2, 0), "abcd");
        assert_eq!(lead_dot(".hidden", "."), "hidden");
        assert_eq!(lead_dot("..x", "."), "..x");
        assert_eq!(crop("abc-def", 2, "b*e"), "af");
        let mut r = Criteria::default().remove;
        r.sym = true;
        assert_eq!(apply_remove(&r, "a.b!c d£"), "abc d");
    }

    #[test]
    fn add_insert_positions() {
        let mut a = Criteria::default().add;
        a.insert = "X".into();
        a.at = "2".into();
        assert_eq!(apply_add(&a, "abc"), "aXbc");
        a.at = "-1".into();
        assert_eq!(apply_add(&a, "abc"), "abXc");
        a.at = "0".into();
        assert_eq!(apply_add(&a, "abc"), "abc");
    }

    #[test]
    fn regex_simple_and_v2() {
        let mut c = Criteria::default();
        c.regex.matcher = r"(\d+)".into();
        c.regex.replace = r"[\1]".into();
        c.ext.mode = 1;
        let mut k = Counter::new();
        assert_eq!(c.new_name(&item("C:/x/a12b.JPG"), &mut k), "a[12]b.jpg");
        let mut s = Criteria::default();
        s.regex.simple = true;
        s.regex.matcher = "%1 - %2".into();
        s.regex.replace = "%2 - %1".into();
        let mut k = Counter::new();
        assert_eq!(s.new_name(&item("C:/x/Artist - Song.mp3"), &mut k), "Song - Artist.mp3");
    }

    #[test]
    fn bases() {
        assert_eq!(format_number(255, 14, 0, 0), "FF");
        assert_eq!(format_number(255, 14, 1, 0), "ff");
        assert_eq!(format_number(28, 35, 0, 0), "AB");
        assert_eq!(format_number(3, 35, 0, 3), "AAC");
        assert_eq!(format_number(14, 37, 0, 0), "XIV");
        assert_eq!(format_number(4000, 37, 0, 0), "MMMM");
        assert_eq!(format_number(7, 8, 0, 3), "007");
    }
}
