use std::path::{Path, PathBuf};
use std::str::FromStr;

use image::{DynamicImage, Rgb, RgbImage};
use itertools::Itertools;

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::parse::csv::{parse_csv, read_csv};
use crate::pdxfile::PdxFile;
use crate::report::{ErrorKey, Severity, err, fatal, report, untidy, warn};
use crate::token::{Loc, Token};

pub type ProvId = u32;

#[derive(Clone, Debug, Default)]
pub struct ImperatorProvinces {
    /// Colors in the provinces.png
    colors: TigerHashSet<Rgb<u8>>,

    /// Kept for adjacency coordinate validation.
    provinces_png: Option<RgbImage>,

    /// Provinces defined in definition.csv.
    /// Imperator requires uninterrupted indices starting at 0, but we want to be able to warn
    /// and continue if they're not, so it's a hashmap.
    provinces: TigerHashMap<ProvId, Province>,

    /// Kept and used for error reporting.
    definition_csv: Option<FileEntry>,

    adjacencies: Vec<Adjacency>,

    impassable: TigerHashSet<ProvId>,

    sea_or_river: TigerHashSet<ProvId>,

    default_map_files: Option<MapFileNames>,

    pending_map_files: Vec<FileEntry>,
}

#[derive(Clone, Debug, Default)]
struct MapFileNames {
    definitions: Option<String>,
    provinces: Option<String>,
    #[allow(dead_code)]
    positions: Option<String>, // set to "positions.txt" in vanilla, but the file is actually missing
    #[allow(dead_code)]
    rivers: Option<String>, // processed in rivers.rs
    #[allow(dead_code)]
    topology: Option<String>, // not processed yet
    adjacencies: Option<String>,
    #[allow(dead_code)]
    areas: Option<String>, // processed in areas.rs
    #[allow(dead_code)]
    regions: Option<String>, // processed in regions.rs
    #[allow(dead_code)]
    ports: Option<String>, // not processed yet
    #[allow(dead_code)]
    climate: Option<String>, // not processed yet
}

impl MapFileNames {
    fn from_default_map(block: &Block) -> Self {
        Self {
            definitions: map_filename(block.get_field_value("definitions")),
            provinces: map_filename(block.get_field_value("provinces")),
            positions: map_filename(block.get_field_value("positions")),
            rivers: map_filename(block.get_field_value("rivers")),
            topology: map_filename(block.get_field_value("topology")),
            adjacencies: map_filename(block.get_field_value("adjacencies")),
            areas: map_filename(block.get_field_value("areas")),
            regions: map_filename(block.get_field_value("regions")),
            ports: map_filename(block.get_field_value("ports")),
            climate: map_filename(block.get_field_value("climate")),
        }
    }

    fn matches_entry(entry: &FileEntry, expected: Option<&String>) -> bool {
        let Some(expected) = expected else { return false };
        let expected = expected.trim();
        if expected.is_empty() {
            return false;
        }

        let expected_path = Path::new(expected);
        if expected_path == entry.path() {
            return true;
        }

        expected_path.file_name().is_some_and(|filename| filename == entry.filename())
    }

    fn is_map_key(key: &Token) -> bool {
        key.lowercase_is("definitions")
            || key.lowercase_is("provinces")
            || key.lowercase_is("positions")
            || key.lowercase_is("rivers")
            || key.lowercase_is("topology")
            || key.lowercase_is("adjacencies")
            || key.lowercase_is("areas")
            || key.lowercase_is("regions")
            || key.lowercase_is("ports")
            || key.lowercase_is("climate")
    }
}

fn map_filename(token: Option<&Token>) -> Option<String> {
    token
        .map(|value| value.as_str().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
}

impl ImperatorProvinces {
    fn province_color(&self, provid: ProvId) -> Option<Rgb<u8>> {
        self.provinces.get(&provid).map(|p| p.color)
    }

    fn provinces_png_pixel(&self, coords: Coords) -> Option<Rgb<u8>> {
        let img = self.provinces_png.as_ref()?;

        let x = u32::try_from(coords.x).ok()?;
        let y = u32::try_from(coords.y).ok()?;

        // Map pixels are addressed as x,y from the top-left corner.
        if x >= img.width() || y >= img.height() {
            return None;
        }

        Some(*img.get_pixel(x, y))
    }

    fn parse_definition(&mut self, csv: &[Token]) {
        if let Some(province) = Province::parse(csv) {
            if self.provinces.contains_key(&province.id) {
                err(ErrorKey::DuplicateItem)
                    .msg("duplicate entry for this province id")
                    .loc(&province.comment)
                    .push();
            }
            self.provinces.insert(province.id, province);
        }
    }

    pub fn load_impassable(&mut self, block: &Block) {
        enum Expecting<'a> {
            Range(&'a Token),
            List(&'a Token),
            Nothing,
        }

        let mut expecting = Expecting::Nothing;
        for item in block.iter_items() {
            match expecting {
                Expecting::Nothing => {
                    if let Some((key, token)) = item.expect_assignment() {
                        if key.lowercase_is("sea_zones")
                            || key.lowercase_is("river_provinces")
                            || key.lowercase_is("impassable_terrain")
                            || key.lowercase_is("uninhabitable")
                            || key.lowercase_is("wasteland")
                            || key.lowercase_is("lakes")
                        {
                            if token.is("LIST") {
                                expecting = Expecting::List(key);
                            } else if token.is("RANGE") {
                                expecting = Expecting::Range(key);
                            } else {
                                expecting = Expecting::Nothing;
                            }
                        } else if !MapFileNames::is_map_key(key) {
                            let msg = format!("unexpected key `{key}`");
                            warn(ErrorKey::UnknownField).msg(msg).loc(key).push();
                        }
                    }
                }
                Expecting::Range(key) => {
                    if let Some(block) = item.expect_block() {
                        let vec: Vec<&Token> = block.iter_values().collect();
                        if vec.len() != 2 {
                            err(ErrorKey::Validation).msg("invalid RANGE").loc(block).push();
                            expecting = Expecting::Nothing;
                            continue;
                        }
                        let from = vec[0].as_str().parse::<ProvId>();
                        let to = vec[1].as_str().parse::<ProvId>();
                        if from.is_err() || to.is_err() {
                            err(ErrorKey::Validation).msg("invalid RANGE").loc(block).push();
                            expecting = Expecting::Nothing;
                            continue;
                        }
                        for provid in from.unwrap()..=to.unwrap() {
                            self.impassable.insert(provid);
                            if key.is("sea_zones") || key.is("river_provinces") {
                                self.sea_or_river.insert(provid);
                            }
                        }
                    }
                    expecting = Expecting::Nothing;
                }
                Expecting::List(key) => {
                    if let Some(block) = item.expect_block() {
                        for token in block.iter_values() {
                            let provid = token.as_str().parse::<ProvId>();
                            if let Ok(provid) = provid {
                                self.impassable.insert(provid);
                                if key.is("sea_zones") || key.is("river_provinces") {
                                    self.sea_or_river.insert(provid);
                                }
                            } else {
                                err(ErrorKey::Validation)
                                    .msg("invalid LIST item")
                                    .loc(token)
                                    .push();
                                break;
                            }
                        }
                    }
                    expecting = Expecting::Nothing;
                }
            }
        }
    }

    pub fn verify_exists_implied(&self, key: &str, item: &Token, max_sev: Severity) {
        if let Ok(provid) = key.parse::<ProvId>() {
            if !self.provinces.contains_key(&provid) {
                let msg = format!("province {provid} not defined in map_data/definition.csv");
                report(ErrorKey::MissingItem, Item::Province.severity()).msg(msg).loc(item).push();
            }
        } else {
            let msg = "province id should be numeric";
            let sev = Item::Province.severity().at_most(max_sev);
            report(ErrorKey::Validation, sev).msg(msg).loc(item).push();
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        if let Ok(provid) = key.parse::<ProvId>() {
            self.provinces.contains_key(&provid)
        } else {
            false
        }
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.provinces.values().map(|item| &item.key)
    }

    pub fn validate(&self, _data: &Everything) {
        for item in &self.adjacencies {
            item.validate(self);
        }
    }

    fn handle_adjacencies_content(&mut self, entry: &FileEntry, content: &str) {
        let mut seen_terminator = false;
        for csv in parse_csv(entry, 1, content) {
            if csv[0].is("-1") {
                seen_terminator = true;
            } else if seen_terminator {
                let msg = "the line with all `-1;` should be the last line in the file";
                warn(ErrorKey::ParseError).msg(msg).loc(&csv[0]).push();
                break;
            } else {
                self.adjacencies.extend(Adjacency::parse(&csv));
            }
        }
        if !seen_terminator {
            let msg = "CK3 needs a line with all `-1;` at the end of this file";
            err(ErrorKey::ParseError).msg(msg).loc(entry).push();
        }
    }

    fn handle_definitions_content(&mut self, entry: &FileEntry, content: &str) {
        self.definition_csv = Some(entry.clone());
        for csv in parse_csv(entry, 0, content) {
            self.parse_definition(&csv);
        }
    }

    fn handle_provinces_image(&mut self, img: DynamicImage, entry: &FileEntry) {
        match img {
            DynamicImage::ImageRgb8(img) => {
                for pixel in img.pixels().dedup() {
                    self.colors.insert(*pixel);
                }

                // Keep the full image for validating adjacency coordinates.
                self.provinces_png = Some(img);
            }
            other => {
                let msg = format!(
                    "`{}` has wrong color format `{:?}`, should be Rgb8",
                    entry.path().display(),
                    other.color()
                );
                err(ErrorKey::ImageFormat).msg(msg).loc(entry).push();
            }
        }
    }

    fn process_map_entry(&mut self, entry: &FileEntry) {
        let Some(map_files) = &self.default_map_files else { return };

        if MapFileNames::matches_entry(entry, map_files.adjacencies.as_ref()) {
            let content = match read_csv(entry.fullpath()) {
                Ok(content) => content,
                Err(e) => {
                    err(ErrorKey::ReadError)
                        .msg(format!("could not read file: {e:#}"))
                        .loc(entry)
                        .push();
                    return;
                }
            };
            self.handle_adjacencies_content(entry, &content);
            return;
        }

        if MapFileNames::matches_entry(entry, map_files.definitions.as_ref()) {
            let content = match read_csv(entry.fullpath()) {
                Ok(content) => content,
                Err(e) => {
                    let msg = format!("could not read `{}`: {:#}", entry.path().display(), e);
                    err(ErrorKey::ReadError).msg(msg).loc(entry).push();
                    return;
                }
            };
            self.handle_definitions_content(entry, &content);
            return;
        }

        if MapFileNames::matches_entry(entry, map_files.provinces.as_ref()) {
            let img = match image::open(entry.fullpath()) {
                Ok(img) => img,
                Err(e) => {
                    let msg = format!("could not read `{}`: {e:#}", entry.path().display());
                    err(ErrorKey::ReadError).msg(msg).loc(entry).push();
                    return;
                }
            };
            self.handle_provinces_image(img, entry);
        }
    }

    fn process_pending_map_entries(&mut self) {
        let pending = std::mem::take(&mut self.pending_map_files);
        for entry in pending {
            self.process_map_entry(&entry);
        }
    }
}

#[derive(Debug)]
pub enum FileContent {
    DefaultMap(Block),
    Deferred,
}

impl FileHandler<FileContent> for ImperatorProvinces {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("map_data")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<FileContent> {
        if entry.path().components().count() == 2 {
            if &*entry.filename().to_string_lossy() == "default.map" {
                return PdxFile::read_optional_bom(entry, parser).map(FileContent::DefaultMap);
            }
            return Some(FileContent::Deferred);
        }
        None
    }

    fn handle_file(&mut self, entry: &FileEntry, content: FileContent) {
        match content {
            FileContent::DefaultMap(block) => {
                let map_files = MapFileNames::from_default_map(&block);
                self.default_map_files = Some(map_files);
                self.load_impassable(&block);
                self.process_pending_map_entries();
            }
            FileContent::Deferred => {
                if self.default_map_files.is_some() {
                    self.process_map_entry(entry);
                } else {
                    self.pending_map_files.push(entry.clone());
                }
            }
        }
    }

    fn finalize(&mut self) {
        if self.default_map_files.is_none() {
            self.default_map_files = Some(MapFileNames {
                definitions: Some("definition.csv".to_string()),
                provinces: Some("provinces.png".to_string()),
                positions: Some("positions.txt".to_string()),
                rivers: Some("rivers.png".to_string()),
                topology: Some("heightmap.heightmap".to_string()),
                adjacencies: Some("adjacencies.csv".to_string()),
                areas: Some("areas.txt".to_string()),
                regions: Some("regions.txt".to_string()),
                ports: Some("ports.csv".to_string()),
                climate: Some("climate.txt".to_string()),
            });
            self.process_pending_map_entries();
        }

        if self.definition_csv.is_none() {
            // Shouldn't happen, it should come from vanilla if not from the mod
            eprintln!("map_data/definition.csv is missing?!?");
            return;
        }
        let definition_csv = self.definition_csv.as_ref().unwrap();

        let mut seen_colors = TigerHashMap::default();
        #[allow(clippy::cast_possible_truncation)]
        for i in 1..self.provinces.len() as u32 {
            if let Some(province) = self.provinces.get(&i) {
                if let Some(k) = seen_colors.get(&province.color) {
                    let msg = format!("color was already used for id {k}");
                    warn(ErrorKey::Colors).msg(msg).loc(&province.comment).push();
                } else {
                    seen_colors.insert(province.color, i);
                }
            } else {
                let msg = format!("province ids must be sequential, but {i} is missing");
                err(ErrorKey::Validation).msg(msg).loc(definition_csv).push();
                return;
            }
        }
        for color in &self.colors {
            if !seen_colors.contains_key(color) {
                let Rgb(rgb) = color;
                let msg = format!(
                    "definitions.csv lacks entry for color ({}, {}, {})",
                    rgb[0], rgb[1], rgb[2]
                );
                untidy(ErrorKey::Colors).msg(msg).loc(definition_csv).push();
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Coords {
    x: i32,
    y: i32,
}

impl Coords {
    fn is_sentinel(self) -> bool {
        self.x == -1 && self.y == -1
    }
}

#[allow(dead_code)] // TODO
#[derive(Clone, Debug)]
pub struct Adjacency {
    line: Loc,
    from: ProvId,
    to: ProvId,
    /// Adjacency kind, should be `sea` or `river_large`.
    kind: Token,
    through: ProvId,
    /// start and stop are map coordinates (should be within provinces.png bounds) and should have the right color on provinces.png
    /// They can be -1 -1 though.
    start: Coords,
    stop: Coords,
    comment: Token,
}

fn verify<T: FromStr>(v: &Token, msg: &str) -> Option<T> {
    let r = v.as_str().parse().ok();
    if r.is_none() {
        err(ErrorKey::ParseError).msg(msg).loc(v).push();
    }
    r
}

impl Adjacency {
    pub fn parse(csv: &[Token]) -> Option<Self> {
        if csv.is_empty() {
            return None;
        }

        let line = csv[0].loc;

        if csv.len() != 9 {
            let msg = "wrong number of fields for this line, expected 9";
            err(ErrorKey::ParseError).msg(msg).loc(&csv[0]).push();
            return None;
        }

        let from = verify(&csv[0], "expected province id");
        let to = verify(&csv[1], "expected province id");
        let through = verify(&csv[3], "expected province id");
        let start_x = verify(&csv[4], "expected x coordinate");
        let start_y = verify(&csv[5], "expected y coordinate");
        let stop_x = verify(&csv[6], "expected x coordinate");
        let stop_y = verify(&csv[7], "expected y coordinate");

        Some(Adjacency {
            line,
            from: from?,
            to: to?,
            kind: csv[2].clone(),
            through: through?,
            start: Coords { x: start_x?, y: start_y? },
            stop: Coords { x: stop_x?, y: stop_y? },
            comment: csv[8].clone(),
        })
    }

    fn validate(&self, provinces: &ImperatorProvinces) {
        for prov in &[self.from, self.to, self.through] {
            if !provinces.provinces.contains_key(prov) {
                let msg = format!("province id {prov} not defined in definitions.csv");
                fatal(ErrorKey::Crash).msg(msg).loc(self.line).push();
            }
        }

        if !self.kind.lowercase_is("sea") && !self.kind.lowercase_is("river_large") {
            let msg = format!(
                "adjacency type `{}` is invalid; expected `sea` or `river_large`",
                self.kind.as_str()
            );
            err(ErrorKey::Validation).msg(msg).loc(&self.kind).push();
        }

        if self.start.is_sentinel() && self.stop.is_sentinel() {
            return;
        }

        let Some(img) = provinces.provinces_png.as_ref() else {
            // Can't validate coordinates without provinces.png.
            return;
        };

        let (w, h) = (img.width(), img.height());
        for (label, coords) in [("start", &self.start), ("stop", &self.stop)] {
            if coords.is_sentinel() {
                continue;
            }

            let x = u32::try_from(coords.x);
            let y = u32::try_from(coords.y);
            if x.is_err() || y.is_err() {
                let msg = format!(
                    "{label} coordinate ({}, {}) is out of bounds (image size {}x{})",
                    coords.x, coords.y, w, h
                );
                err(ErrorKey::Validation).msg(msg).loc(&self.comment).push();
                continue;
            }
            let (x, y) = (x.unwrap(), y.unwrap());
            if x >= w || y >= h {
                let msg = format!(
                    "{label} coordinate ({}, {}) is out of bounds (image size {}x{})",
                    coords.x, coords.y, w, h
                );
                err(ErrorKey::Validation).msg(msg).loc(&self.comment).push();
            }
        }

        if !self.start.is_sentinel() {
            let Some(expected_start) = provinces.province_color(self.from) else {
                return;
            };
            if let Some(actual) = provinces.provinces_png_pixel(self.start) {
                if actual != expected_start {
                    let Rgb([er, eg, eb]) = expected_start;
                    let Rgb([ar, ag, ab]) = actual;
                    let msg = format!(
                        "start coordinate is in the wrong province color: expected ({er}, {eg}, {eb}), got ({ar}, {ag}, {ab})"
                    );
                    err(ErrorKey::Validation).msg(msg).loc(&self.comment).push();
                }
            }
        }

        if !self.stop.is_sentinel() {
            let Some(expected_stop) = provinces.province_color(self.to) else {
                return;
            };
            if let Some(actual) = provinces.provinces_png_pixel(self.stop) {
                if actual != expected_stop {
                    let Rgb([er, eg, eb]) = expected_stop;
                    let Rgb([ar, ag, ab]) = actual;
                    let msg = format!(
                        "stop coordinate is in the wrong province color: expected ({er}, {eg}, {eb}), got ({ar}, {ag}, {ab})"
                    );
                    err(ErrorKey::Validation).msg(msg).loc(&self.comment).push();
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Province {
    key: Token,
    id: ProvId,
    color: Rgb<u8>,
    comment: Token,
}

impl Province {
    fn parse(csv: &[Token]) -> Option<Self> {
        if csv.is_empty() {
            return None;
        }

        if csv.len() < 5 {
            let msg = "too few fields for this line, expected 5";
            err(ErrorKey::ParseError).msg(msg).loc(&csv[0]).push();
            return None;
        }

        let id = verify(&csv[0], "expected province id")?;
        let r = verify(&csv[1], "expected red value")?;
        let g = verify(&csv[2], "expected green value")?;
        let b = verify(&csv[3], "expected blue value")?;
        let color = Rgb::from([r, g, b]);
        Some(Province { key: csv[0].clone(), id, color, comment: csv[4].clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use crate::fileset::FileKind;
    use crate::report::take_reports;

    fn loc(line: u32, column: u32) -> Loc {
        let mut loc = Loc::for_file(
            PathBuf::from("map_data/adjacencies.csv"),
            FileKind::Mod,
            PathBuf::from("C:/test/map_data/adjacencies.csv"),
        );
        loc.line = line;
        loc.column = column;
        loc
    }

    fn tok(s: &str, line: u32, column: u32) -> Token {
        Token::new(s, loc(line, column))
    }

    fn base_provinces(img: RgbImage, from_color: Rgb<u8>, to_color: Rgb<u8>) -> ImperatorProvinces {
        let _ = take_reports();

        let mut provinces = ImperatorProvinces::default();
        provinces.provinces_png = Some(img);
        provinces.provinces.insert(
            1,
            Province { key: tok("1", 1, 1), id: 1, color: from_color, comment: tok("c", 1, 1) },
        );
        provinces.provinces.insert(
            2,
            Province { key: tok("2", 1, 1), id: 2, color: to_color, comment: tok("c", 1, 1) },
        );
        provinces
    }

    fn adjacency(start: Coords, stop: Coords) -> Adjacency {
        Adjacency {
            line: loc(1, 1),
            from: 1,
            to: 2,
            kind: tok("sea", 1, 5),
            through: 1,
            start,
            stop,
            comment: tok("comment", 1, 10),
        }
    }

    fn adjacency_with_kind(kind: &str, start: Coords, stop: Coords) -> Adjacency {
        Adjacency { kind: tok(kind, 1, 5), ..adjacency(start, stop) }
    }

    fn take_msgs() -> Vec<String> {
        take_reports().into_iter().map(|(meta, _)| meta.msg).collect()
    }

    #[test]
    fn adjacency_start_out_of_bounds_errors() {
        let img = RgbImage::from_pixel(2, 2, Rgb([1, 2, 3]));
        let provinces = base_provinces(img, Rgb([1, 2, 3]), Rgb([9, 9, 9]));

        let adj = adjacency(Coords { x: 5, y: 0 }, Coords { x: -1, y: -1 });
        adj.validate(&provinces);

        let msgs = take_msgs();
        assert!(
            msgs.iter().any(|m| m.contains("start coordinate (5, 0) is out of bounds")),
            "reports were: {msgs:?}"
        );
    }

    #[test]
    fn adjacency_start_wrong_color_errors() {
        let mut img = RgbImage::from_pixel(2, 2, Rgb([0, 0, 0]));
        img.put_pixel(0, 0, Rgb([9, 9, 9]));
        let provinces = base_provinces(img, Rgb([1, 2, 3]), Rgb([7, 8, 9]));

        let adj = adjacency(Coords { x: 0, y: 0 }, Coords { x: -1, y: -1 });
        adj.validate(&provinces);

        let msgs = take_msgs();
        assert!(
            msgs.iter().any(|m| m.contains("start coordinate is in the wrong province color")),
            "reports were: {msgs:?}"
        );
    }

    #[test]
    fn adjacency_stop_wrong_color_errors_when_start_sentinel() {
        let mut img = RgbImage::from_pixel(2, 2, Rgb([0, 0, 0]));
        img.put_pixel(1, 1, Rgb([9, 9, 9]));
        let provinces = base_provinces(img, Rgb([1, 2, 3]), Rgb([7, 8, 9]));

        let adj = adjacency(Coords { x: -1, y: -1 }, Coords { x: 1, y: 1 });
        adj.validate(&provinces);

        let msgs = take_msgs();
        assert!(
            msgs.iter().any(|m| m.contains("stop coordinate is in the wrong province color")),
            "reports were: {msgs:?}"
        );
    }

    #[test]
    fn adjacency_one_endpoint_sentinel_can_still_pass() {
        let mut img = RgbImage::from_pixel(2, 2, Rgb([0, 0, 0]));
        img.put_pixel(1, 0, Rgb([7, 8, 9]));
        let provinces = base_provinces(img, Rgb([1, 2, 3]), Rgb([7, 8, 9]));

        let adj = adjacency(Coords { x: -1, y: -1 }, Coords { x: 1, y: 0 });
        adj.validate(&provinces);

        let msgs = take_msgs();
        assert!(msgs.is_empty(), "reports were: {msgs:?}");
    }

    #[test]
    fn adjacency_kind_invalid_errors_even_if_coords_sentinel() {
        let img = RgbImage::from_pixel(2, 2, Rgb([1, 2, 3]));
        let provinces = base_provinces(img, Rgb([1, 2, 3]), Rgb([9, 9, 9]));

        let adj = adjacency_with_kind("land", Coords { x: -1, y: -1 }, Coords { x: -1, y: -1 });
        adj.validate(&provinces);

        let msgs = take_msgs();
        assert!(
            msgs.iter().any(|m| m.contains("adjacency type `land` is invalid")),
            "reports were: {msgs:?}"
        );
    }

    #[test]
    fn adjacency_kind_sea_and_river_large_are_allowed() {
        let mut img = RgbImage::from_pixel(2, 2, Rgb([0, 0, 0]));
        img.put_pixel(0, 0, Rgb([1, 2, 3]));
        img.put_pixel(1, 0, Rgb([7, 8, 9]));
        let provinces = base_provinces(img, Rgb([1, 2, 3]), Rgb([7, 8, 9]));

        for kind in ["sea", "river_large"] {
            let adj = adjacency_with_kind(kind, Coords { x: 0, y: 0 }, Coords { x: 1, y: 0 });
            adj.validate(&provinces);

            let msgs = take_msgs();
            assert!(msgs.is_empty(), "kind {kind} reports were: {msgs:?}");
        }
    }
}
