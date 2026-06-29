use std::collections::HashMap;

use serde_with::{serde_as, DisplayFromStr};

use crate::common::Encoding;
use crate::engine_constants::{DataType, EngineConstants};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::game::shared_game_state::FontData;

#[derive(Debug, Clone)]
pub struct Locale {
    pub code: String,
    pub name: String,
    pub font: FontData,
    pub encoding: Option<Encoding>,
    pub stage_encoding: Option<Encoding>,
    strings: HashMap<String, String>,
}

impl Default for Locale {
    fn default() -> Self {
        Locale {
            code: "en".to_owned(),
            name: "English".to_owned(),
            font: FontData { path: String::new(), scale: 1.0, space_offset: 0.0 },
            encoding: None,
            stage_encoding: None,
            strings: HashMap::new(),
        }
    }
}

impl Locale {
    pub fn new(ctx: &Context, base_paths: &Vec<String>, code: &str) -> Locale {
        let file = filesystem::open_find(ctx, base_paths, &format!("locale/{code}.json")).unwrap();
        let json: serde_json::Value = serde_json::from_reader(file).unwrap();

        let strings = Locale::flatten(&json);

        let name = strings["name"].clone();

        let font_name = strings["font"].clone();
        let font_scale = strings["font_scale"].parse::<f32>().unwrap_or(1.0);
        let font = FontData::new(font_name, font_scale, 0.0);

        let encoding =
            if let Some(enc) = strings.get("encoding").clone() { Some(Encoding::from(enc.as_str())) } else { None };
        let stage_encoding = if let Some(enc) = strings.get("stage_encoding").clone() {
            Some(Encoding::from(enc.as_str()))
        } else {
            None
        };

        Locale { code: code.to_string(), name, font, encoding, stage_encoding, strings }
    }

    fn flatten(json: &serde_json::Value) -> HashMap<String, String> {
        let mut strings = HashMap::new();

        for (key, value) in json.as_object().unwrap() {
            match value {
                serde_json::Value::String(string) => {
                    strings.insert(key.to_owned(), string.to_owned());
                }
                serde_json::Value::Object(_) => {
                    let substrings = Locale::flatten(value);

                    for (sub_key, sub_value) in substrings.iter() {
                        strings.insert(format!("{}.{}", key, sub_key), sub_value.to_owned());
                    }
                }
                _ => {}
            }
        }

        strings
    }

    /// if the key does not exists, return the origin key instead
    pub fn t<'a: 'b, 'b>(&'a self, key: &'b str) -> &'b str {
        if let Some(str) = self.strings.get(key) {
            str
        } else {
            key
        }
    }

    pub fn tt(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut string = self.t(key).to_owned();

        for (key, value) in args.iter() {
            string = string.replace(&format!("{{{}}}", key), &value);
        }

        string
    }

    pub fn set_font(&mut self, font: FontData) {
        self.font = font;
    }
}

#[serde_as]
#[derive(Clone, Debug, serde::Deserialize)]
/// Info about a translation variant.
///
/// A translation variant contains information about a specific translation of game files
/// into a particular language (its name, code and game data parameters), while the [`Locale`] contains data
/// common to all translation variants (font parameters and UI strings).
pub struct Translation {
    /// The locale code for which the translation variant is intended.
    pub locale: String,
    /// Code of the translation variant.
    /// `None` represents the default translation variant for the specified locale.
    pub code: Option<String>,
    /// Name of the the translation variant (typically the name of the translator or translation team).
    pub name: String,

    #[serde_as(as = "Option<DisplayFromStr>")]
    /// Encoding of text scripts.
    pub encoding: Option<Encoding>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    /// Encoding of strings in the stage table.
    pub stage_encoding: Option<Encoding>,

    // Properties of the translation data "root"
    #[serde(default, skip)]
    /// Whether the translation game data directory exists.
    pub available: bool,
    #[serde(default, skip)]
    /// Whether the translation data directory is complete.
    ///
    /// Incomplete translations don't contain all necessary files,
    /// as they are intended to contain only translatable files (scripts, textures, the stage table);
    /// therefore, a type of a type of game data cannot be determined for them.
    pub is_complete: bool,
    #[serde(default, skip)]
    pub data_type: Option<DataType>,
}

impl Default for Translation {
    fn default() -> Self {
        let locale = Locale::default();
        Self {
            locale: locale.code,
            code: None,
            name: "".to_string(),
            encoding: None,
            stage_encoding: None,
            available: false,
            is_complete: false,
            data_type: None,
        }
    }
}

impl Translation {
    pub fn new(ctx: &mut Context, base_paths: &Vec<String>, code: &str) -> GameResult<Self> {
        let file = filesystem::open_find(ctx, base_paths, &format!("translation/{code}.json"))?;
        Self::from_file(file)
    }

    pub fn from_file<R: std::io::Read>(mut file: R) -> GameResult<Self> {
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(serde_json::from_str(&content)?)
    }

    pub fn full_code(&self) -> String {
        let mut full_code = self.locale.clone();

        if let Some(code) = &self.code {
            full_code.push('_');
            full_code.push_str(code);
        }

        full_code
    }

    /// Whether this translation is the default for its locale.
    pub fn is_default(&self) -> bool {
        self.code.is_none()
    }

    pub fn analyze_data(&mut self, ctx: &Context, constants: &EngineConstants) {
        if self.locale == constants.base_locale && self.is_default() {
            self.available = true;
            self.is_complete = true;
            self.data_type = constants.active_root.data_type;
        } else {
            let tran_path = constants.active_root.translation_path(ctx, self);
            self.available = filesystem::exists(ctx, &tran_path);

            if self.available {
                self.data_type = DataType::from_path(ctx, tran_path.as_str());

                if self.data_type.is_none() {
                    let translation_root = constants.roots.get(&tran_path);

                    self.is_complete = translation_root.is_some(); // incomplete translation roots aren't added to the list
                    self.data_type = translation_root.unwrap_or(&constants.active_root).data_type;
                }
            }
        }
    }
}

impl From<&Locale> for Translation {
    fn from(value: &Locale) -> Self {
        Self {
            locale: value.code.clone(),
            name: "".to_string(),
            encoding: value.encoding,
            stage_encoding: value.stage_encoding,
            ..Default::default()
        }
    }
}
