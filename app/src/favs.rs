//! Favourite files: INI sections and keys holding the criteria, filters and copy/move location.

use crate::engine::Criteria;
use crate::files::Filters;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Location {
    pub on: bool,
    pub path: String,
    pub copy_not_move: bool,
    pub keep_structure: bool,
}

type Ini = BTreeMap<String, BTreeMap<String, String>>;

fn b(v: bool) -> String {
    if v { "1".into() } else { "0".into() }
}

pub fn save(c: &Criteria, f: &Filters, loc: &Location) -> String {
    let mut ini: Vec<(&str, Vec<(&str, String)>)> = Vec::new();
    ini.push(("RegEx", vec![
        ("Match", c.regex.matcher.clone()), ("Replace", c.regex.replace.clone()),
        ("Include Extension", b(c.regex.inc_ext)), ("Simple", b(c.regex.simple)), ("V2", b(c.regex.v2)),
    ]));
    ini.push(("Filename", vec![("Mode", c.name.mode.to_string()), ("Fixed", c.name.text.clone())]));
    ini.push(("Replacements", vec![
        ("Replace", c.replace.find.clone()), ("With", c.replace.with.clone()),
        ("Match Case", b(c.replace.match_case)), ("First Only", b(c.replace.first)),
    ]));
    ini.push(("Case", vec![("Change", c.case.mode.to_string()), ("Exceptions", c.case.except.clone())]));
    let r = &c.remove;
    ini.push(("Removals", vec![
        ("First N", r.first_n.clone()), ("Last N", r.last_n.clone()), ("From", r.from.clone()), ("To", r.to.clone()),
        ("Chars", r.chars.clone()), ("Words", r.words.clone()), ("All Digits", b(r.digits)), ("All Chars", b(r.alpha)),
        ("All Symbols", b(r.sym)), ("All High", b(r.high)), ("Trim", b(r.trim)), ("Double Space", b(r.ds)),
        ("Accents", b(r.accents)), ("Crop Mode", r.crop.to_string()), ("Crop Marker", r.crop_marker.clone()),
        ("Lead Dots", r.lead_dots.to_string()),
    ]));
    let m = &c.movecopy;
    ini.push(("MoveCopy", vec![
        ("Source", m.source.to_string()), ("Number", m.count.clone()), ("Range", m.range_to.clone()),
        ("Target", m.target.to_string()), ("Target Pos", m.pos.clone()), ("Separator", m.sep.clone()),
    ]));
    let a = &c.add;
    ini.push(("Additions", vec![
        ("Prefix", a.prefix.clone()), ("Text", a.insert.clone()), ("Text Pos", a.at.clone()),
        ("Suffix", a.suffix.clone()), ("WordSpace", b(a.word_space)),
    ]));
    let d = &c.date;
    ini.push(("Auto Date", vec![
        ("Mode", d.mode.to_string()), ("Type", d.kind.to_string()), ("Format", d.fmt.to_string()),
        ("Separator", d.sep.clone()), ("Segments", d.seg.clone()), ("Custom", d.custom.clone()),
        ("Century", b(d.century)), ("Offset", d.offset.clone()),
    ]));
    ini.push(("Folder Name", vec![("Mode", c.folder.mode.to_string()), ("Separator", c.folder.sep.clone()), ("Levels", c.folder.levels.clone())]));
    let n = &c.number;
    ini.push(("Auto-Number", vec![
        ("Mode", n.mode.to_string()), ("Start", n.start.clone()), ("Insert", n.at.clone()), ("Increment", n.incr.clone()),
        ("Pad", n.pad.clone()), ("Break", n.brk.clone()), ("Separator", n.sep.clone()), ("Type", n.kind.to_string()),
        ("Case", n.case.to_string()), ("Break Folder", b(n.folder)),
    ]));
    ini.push(("Extension", vec![("Mode", c.ext.mode.to_string()), ("Fixed", c.ext.text.clone())]));
    ini.push(("Selections", vec![
        ("Filter", f.mask.clone()), ("Filter Match Case", b(f.match_case)), ("Filter Use Regex", b(f.regex)),
        ("Files", b(f.files)), ("Folders", b(f.folders)), ("Subfolders", b(f.subfolders)), ("Hidden", b(f.hidden)),
        ("Subfolders Level", f.level.clone()), ("Min Name Length", f.name_min.clone()), ("Min Path Length", f.path_min.clone()),
        ("Max Name Length", f.name_max.clone()), ("Max Path Length", f.path_max.clone()), ("Condition", f.condition.clone()),
    ]));
    ini.push(("New Location", vec![
        ("Enabled", b(loc.on)), ("Path", loc.path.clone()), ("Copy Not Move", b(loc.copy_not_move)), ("Keep Folder Structure", b(loc.keep_structure)),
    ]));
    ini.push(("Name Segment", vec![("Enabled", b(c.segment.on)), ("From", c.segment.from.clone()), ("To", c.segment.to.clone())]));
    ini.push(("Criteria", vec![
        ("RegEx", b(c.regex.on)), ("File", b(c.name.on)), ("Replace", b(c.replace.on)), ("Case", b(c.case.on)),
        ("Removals", b(c.remove.on)), ("MoveCopy", b(c.movecopy.on)), ("Additions", b(c.add.on)), ("AutoDate", b(c.date.on)),
        ("Folder", b(c.folder.on)), ("AutoNumber", b(c.number.on)), ("Extension", b(c.ext.on)),
    ]));
    let order: Vec<String> = c.order.iter().map(|i| i.to_string()).collect();
    ini.push(("Renaming Criteria Order", vec![("Criteria Order", order.join(","))]));
    let mut out = String::new();
    for (sec, keys) in ini {
        out.push_str(&format!("[{sec}]\r\n"));
        for (k, v) in keys {
            out.push_str(&format!("{k}={v}\r\n"));
        }
        out.push_str("\r\n");
    }
    out.push_str("[Character Translation]\r\n");
    out.push_str(&format!("Enabled={}\r\n", b(c.translations_on)));
    for (i, (a, t)) in c.translations.iter().enumerate() {
        out.push_str(&format!("Pair {i}={a}={t}\r\n"));
    }
    out
}

fn parse(text: &str) -> Ini {
    let mut ini = Ini::new();
    let mut sec = String::new();
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if line.starts_with('[') && line.ends_with(']') {
            sec = line[1..line.len() - 1].to_string();
        } else if let Some(i) = line.find('=') {
            ini.entry(sec.clone()).or_default().insert(line[..i].trim().to_string(), line[i + 1..].to_string());
        }
    }
    ini
}

pub fn load(text: &str, c: &mut Criteria, f: &mut Filters, loc: &mut Location) {
    let ini = parse(text);
    let s = |sec: &str, key: &str, cur: &mut String| {
        if let Some(v) = ini.get(sec).and_then(|m| m.get(key)) {
            *cur = v.clone();
        }
    };
    let bo = |sec: &str, key: &str, cur: &mut bool| {
        if let Some(v) = ini.get(sec).and_then(|m| m.get(key)) {
            *cur = v.trim() == "1";
        }
    };
    let us = |sec: &str, key: &str, cur: &mut usize| {
        if let Some(v) = ini.get(sec).and_then(|m| m.get(key)) {
            *cur = v.trim().parse().unwrap_or(0);
        }
    };
    s("RegEx", "Match", &mut c.regex.matcher);
    s("RegEx", "Replace", &mut c.regex.replace);
    bo("RegEx", "Include Extension", &mut c.regex.inc_ext);
    bo("RegEx", "Simple", &mut c.regex.simple);
    bo("RegEx", "V2", &mut c.regex.v2);
    us("Filename", "Mode", &mut c.name.mode);
    s("Filename", "Fixed", &mut c.name.text);
    s("Replacements", "Replace", &mut c.replace.find);
    s("Replacements", "With", &mut c.replace.with);
    bo("Replacements", "Match Case", &mut c.replace.match_case);
    bo("Replacements", "First Only", &mut c.replace.first);
    us("Case", "Change", &mut c.case.mode);
    s("Case", "Exceptions", &mut c.case.except);
    let r = &mut c.remove;
    s("Removals", "First N", &mut r.first_n);
    s("Removals", "Last N", &mut r.last_n);
    s("Removals", "From", &mut r.from);
    s("Removals", "To", &mut r.to);
    s("Removals", "Chars", &mut r.chars);
    s("Removals", "Words", &mut r.words);
    bo("Removals", "All Digits", &mut r.digits);
    bo("Removals", "All Chars", &mut r.alpha);
    bo("Removals", "All Symbols", &mut r.sym);
    bo("Removals", "All High", &mut r.high);
    bo("Removals", "Trim", &mut r.trim);
    bo("Removals", "Double Space", &mut r.ds);
    bo("Removals", "Accents", &mut r.accents);
    us("Removals", "Crop Mode", &mut r.crop);
    s("Removals", "Crop Marker", &mut r.crop_marker);
    us("Removals", "Lead Dots", &mut r.lead_dots);
    let m = &mut c.movecopy;
    us("MoveCopy", "Source", &mut m.source);
    s("MoveCopy", "Number", &mut m.count);
    s("MoveCopy", "Range", &mut m.range_to);
    us("MoveCopy", "Target", &mut m.target);
    s("MoveCopy", "Target Pos", &mut m.pos);
    s("MoveCopy", "Separator", &mut m.sep);
    let a = &mut c.add;
    s("Additions", "Prefix", &mut a.prefix);
    s("Additions", "Text", &mut a.insert);
    s("Additions", "Text Pos", &mut a.at);
    s("Additions", "Suffix", &mut a.suffix);
    bo("Additions", "WordSpace", &mut a.word_space);
    let d = &mut c.date;
    us("Auto Date", "Mode", &mut d.mode);
    us("Auto Date", "Type", &mut d.kind);
    us("Auto Date", "Format", &mut d.fmt);
    s("Auto Date", "Separator", &mut d.sep);
    s("Auto Date", "Segments", &mut d.seg);
    s("Auto Date", "Custom", &mut d.custom);
    bo("Auto Date", "Century", &mut d.century);
    s("Auto Date", "Offset", &mut d.offset);
    us("Folder Name", "Mode", &mut c.folder.mode);
    s("Folder Name", "Separator", &mut c.folder.sep);
    s("Folder Name", "Levels", &mut c.folder.levels);
    let n = &mut c.number;
    us("Auto-Number", "Mode", &mut n.mode);
    s("Auto-Number", "Start", &mut n.start);
    s("Auto-Number", "Insert", &mut n.at);
    s("Auto-Number", "Increment", &mut n.incr);
    s("Auto-Number", "Pad", &mut n.pad);
    s("Auto-Number", "Break", &mut n.brk);
    s("Auto-Number", "Separator", &mut n.sep);
    us("Auto-Number", "Type", &mut n.kind);
    us("Auto-Number", "Case", &mut n.case);
    bo("Auto-Number", "Break Folder", &mut n.folder);
    us("Extension", "Mode", &mut c.ext.mode);
    s("Extension", "Fixed", &mut c.ext.text);
    s("Selections", "Filter", &mut f.mask);
    bo("Selections", "Filter Match Case", &mut f.match_case);
    bo("Selections", "Filter Use Regex", &mut f.regex);
    bo("Selections", "Files", &mut f.files);
    bo("Selections", "Folders", &mut f.folders);
    bo("Selections", "Subfolders", &mut f.subfolders);
    bo("Selections", "Hidden", &mut f.hidden);
    s("Selections", "Subfolders Level", &mut f.level);
    s("Selections", "Min Name Length", &mut f.name_min);
    s("Selections", "Min Path Length", &mut f.path_min);
    s("Selections", "Max Name Length", &mut f.name_max);
    s("Selections", "Max Path Length", &mut f.path_max);
    s("Selections", "Condition", &mut f.condition);
    bo("New Location", "Enabled", &mut loc.on);
    s("New Location", "Path", &mut loc.path);
    bo("New Location", "Copy Not Move", &mut loc.copy_not_move);
    bo("New Location", "Keep Folder Structure", &mut loc.keep_structure);
    bo("Name Segment", "Enabled", &mut c.segment.on);
    s("Name Segment", "From", &mut c.segment.from);
    s("Name Segment", "To", &mut c.segment.to);
    bo("Criteria", "RegEx", &mut c.regex.on);
    bo("Criteria", "File", &mut c.name.on);
    bo("Criteria", "Replace", &mut c.replace.on);
    bo("Criteria", "Case", &mut c.case.on);
    bo("Criteria", "Removals", &mut c.remove.on);
    bo("Criteria", "MoveCopy", &mut c.movecopy.on);
    bo("Criteria", "Additions", &mut c.add.on);
    bo("Criteria", "AutoDate", &mut c.date.on);
    bo("Criteria", "Folder", &mut c.folder.on);
    bo("Criteria", "AutoNumber", &mut c.number.on);
    bo("Criteria", "Extension", &mut c.ext.on);
    if let Some(o) = ini.get("Renaming Criteria Order").and_then(|m| m.get("Criteria Order")) {
        let v: Vec<u8> = o.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        let mut sorted = v.clone();
        sorted.sort();
        if sorted == (1..=11).collect::<Vec<u8>>() {
            c.order = v;
        }
    }
    if let Some(tr) = ini.get("Character Translation") {
        c.translations_on = tr.get("Enabled").map(|v| v.trim() == "1").unwrap_or(false);
        let mut pairs: Vec<(usize, (String, String))> = tr
            .iter()
            .filter_map(|(k, v)| {
                let i: usize = k.strip_prefix("Pair ")?.trim().parse().ok()?;
                let eq = v.find('=')?;
                Some((i, (v[..eq].to_string(), v[eq + 1..].to_string())))
            })
            .collect();
        pairs.sort_by_key(|p| p.0);
        c.translations = pairs.into_iter().map(|p| p.1).collect();
    }
}
