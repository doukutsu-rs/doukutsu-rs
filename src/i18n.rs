use std::collections::HashMap;

use crate::framework::context::Context;
use crate::framework::filesystem;
use crate::game::shared_game_state::FontData;

#[derive(Debug, Clone)]
pub struct Locale {
    pub code: String,
    pub name: String,
    pub font: FontData,
    strings: HashMap<String, String>,
}

impl Locale {
    pub fn new(ctx: &mut Context, base_paths: &Vec<String>, code: &str) -> Locale {
        let file = filesystem::open_find(ctx, base_paths, &format!("locale/{}.json", code)).unwrap();
        let json: serde_json::Value = serde_json::from_reader(file).unwrap();

        let strings = Locale::flatten(&json);

        let name = strings["name"].clone();

        let font_name = strings["font"].clone();
        let font_scale = strings["font_scale"].parse::<f32>().unwrap_or(1.0);
        let font = FontData::new(font_name, font_scale, 0.0);

        Locale { code: code.to_string(), name, font, strings }
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

    pub fn t(&self, key: &str) -> String {
        self.strings.get(key).unwrap_or(&key.to_owned()).to_owned()
    }

    pub fn tt(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut string = self.t(key);

        for (key, value) in args.iter() {
            string = string.replace(&format!("{{{}}}", key), &value);
        }

        string
    }

    pub fn set_font(&mut self, font: FontData) {
        self.font = font;
    }
}
