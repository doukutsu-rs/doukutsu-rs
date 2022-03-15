use crate::framework::context::Context;
use crate::framework::filesystem;
use crate::shared_game_state::FontData;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Locale {
    strings: HashMap<String, String>,
    pub font: FontData,
}

impl Locale {
    pub fn new(ctx: &mut Context, code: &str, font: FontData) -> Locale {
        let mut filename = "en.json".to_owned();

        if code != "en" && filesystem::exists(ctx, &format!("/builtin/locale/{}.json", code)) {
            filename = format!("{}.json", code);
        }

        let file = filesystem::open(ctx, &format!("/builtin/locale/{}", filename)).unwrap();
        let json: serde_json::Value = serde_json::from_reader(file).unwrap();

        let strings = Locale::flatten(&json);

        Locale { strings, font }
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

    pub fn tt(&self, key: &str, args: HashMap<String, String>) -> String {
        let mut string = self.t(key);

        for (key, value) in args.iter() {
            string = string.replace(&format!("{{{}}}", key), &value);
        }

        string
    }
}
