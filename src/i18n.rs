use std::collections::HashMap;

use serde_with::{DisplayFromStr, serde_as};

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::game::scripting::tsc::text_script::TextScriptEncoding;
use crate::game::shared_game_state::FontData;

#[derive(Debug, Clone)]
pub struct Locale {
    pub code: String,
    pub name: String,
    pub font: FontData,
    #[deprecated]
    pub encoding: Option<TextScriptEncoding>,
    #[deprecated]
    pub stage_encoding: Option<TextScriptEncoding>,
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
    pub fn new(ctx: &mut Context, base_paths: &Vec<String>, code: &str) -> Locale {
        let file = filesystem::open_find(ctx, base_paths, &format!("locale/{code}.json")).unwrap();
        let json: serde_json::Value = serde_json::from_reader(file).unwrap();

        let strings = Locale::flatten(&json);

        let name = strings["name"].clone();

        let font_name = strings["font"].clone();
        let font_scale = strings["font_scale"].parse::<f32>().unwrap_or(1.0);
        let font = FontData::new(font_name, font_scale, 0.0);

        let encoding = if let Some(enc) = strings.get("encoding").clone() {
            Some(TextScriptEncoding::from(enc.as_str()))
        } else {
            None
        };
        let stage_encoding = if let Some(enc) = strings.get("stage_encoding").clone() {
            Some(TextScriptEncoding::from(enc.as_str()))
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
pub struct Translation {
    pub locale: String,
    pub code: Option<String>,
    pub name: String,
    #[serde_as(as = "DisplayFromStr")]
    pub encoding: TextScriptEncoding,
    #[serde_as(as = "DisplayFromStr")]
    pub stage_encoding: TextScriptEncoding,

    #[serde(default, skip)]
    pub is_present: bool,
}

impl Translation {
    pub fn new(ctx: &mut Context, base_paths: &Vec<String>, code: &str) -> GameResult<Self> {
        let file = filesystem::open_find(ctx, base_paths, &format!("translation/{code}.json"))?;
        Ok(serde_json::from_reader::<_, Self>(file)?)
    }

    pub fn full_code(&self) -> String {
        let mut full_code = self.locale.clone();

        if let Some(code) = &self.code {
            full_code.push('_');
            full_code.push_str(code);
        }

        full_code
    }
}
