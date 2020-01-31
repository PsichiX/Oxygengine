use crate::assets::{
    asset::AssetID, database::AssetsDatabase, protocols::localization::LocalizationAsset,
};
use pest::{iterators::Pair, Parser};
use specs::{ReadExpect, System, Write};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "localization.pest"]
struct SentenceParser;

// TODO: swap text id with language.
/// { text id: { language: text format } }
#[derive(Default)]
pub struct Localization {
    default_language: Option<String>,
    current_language: Option<String>,
    map: HashMap<String, HashMap<String, String>>,
}

impl Localization {
    pub fn default_language(&self) -> Option<&str> {
        self.default_language.as_ref().map(|v| v.as_str())
    }

    pub fn set_default_language(&mut self, value: Option<String>) {
        self.default_language = value;
    }

    pub fn current_language(&self) -> Option<&str> {
        self.current_language.as_ref().map(|v| v.as_str())
    }

    pub fn set_current_language(&mut self, value: Option<String>) {
        self.current_language = value.clone();
        if self.default_language.is_none() && value.is_some() {
            self.default_language = value;
        }
    }

    pub fn add_text(&mut self, id: &str, language: &str, text_format: &str) {
        if let Some(map) = self.map.get_mut(id) {
            map.insert(language.to_owned(), text_format.to_owned());
        } else {
            let mut map = HashMap::new();
            map.insert(language.to_owned(), text_format.to_owned());
            self.map.insert(id.to_owned(), map);
        }
    }

    pub fn remove_text(&mut self, id: &str, language: &str) -> bool {
        let (empty, removed) = if let Some(map) = self.map.get_mut(id) {
            let removed = map.remove(language).is_some();
            let empty = map.is_empty();
            (empty, removed)
        } else {
            (false, false)
        };
        if empty {
            self.map.remove(id);
        }
        removed
    }

    pub fn remove_text_all(&mut self, id: &str) -> bool {
        self.map.remove(id).is_some()
    }

    pub fn remove_language(&mut self, lang: &str) {
        for map in self.map.values_mut() {
            map.remove(lang);
        }
    }

    pub fn find_text_format(&self, id: &str) -> Option<&str> {
        if let Some(current) = &self.current_language {
            if let Some(default) = &self.default_language {
                if let Some(map) = self.map.get(id) {
                    return map
                        .get(current)
                        .or_else(|| map.get(default))
                        .or(None)
                        .as_ref()
                        .map(|v| v.as_str());
                }
            }
        }
        None
    }

    pub fn format_text(&self, id: &str, params: &[(&str, &str)]) -> Result<String, String> {
        if let Some(text_format) = self.find_text_format(id) {
            match SentenceParser::parse(Rule::sentence, &text_format) {
                Ok(mut ast) => {
                    let pair = ast.next().unwrap();
                    match pair.as_rule() {
                        Rule::sentence => Ok(Self::parse_sentence_inner(pair, params)),
                        _ => unreachable!(),
                    }
                }
                Err(error) => Err(error.to_string()),
            }
        } else {
            Err(format!("There is no text format for id: {}", id))
        }
    }

    fn parse_sentence_inner(pair: Pair<Rule>, params: &[(&str, &str)]) -> String {
        let mut result = String::new();
        for p in pair.into_inner() {
            match p.as_rule() {
                Rule::text => result.push_str(&p.as_str().replace("\\|", "|")),
                Rule::identifier => {
                    let ident = p.as_str();
                    if let Some((_, v)) = params.iter().find(|(id, _)| id == &ident) {
                        result.push_str(v);
                    } else {
                        result.push_str(&format!("{{@{}}}", ident));
                    }
                }
                _ => {}
            }
        }
        result
    }
}

#[macro_export]
macro_rules! localization_format_text {
    ($res:expr, $text:expr, $( $id:expr => $value:expr ),*) => {
        $crate::localization::Localization::format_text(&$res, $text, &[ $( ($id, &$value.to_string()) ),* ])
    }
}

#[derive(Default)]
pub struct LocalizationSystem {
    language_table: HashMap<AssetID, String>,
}

impl<'s> System<'s> for LocalizationSystem {
    type SystemData = (ReadExpect<'s, AssetsDatabase>, Write<'s, Localization>);

    fn run(&mut self, (assets, mut localization): Self::SystemData) {
        for id in assets.lately_loaded_protocol("locals") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded localization asset");
            let asset = asset
                .get::<LocalizationAsset>()
                .expect("trying to use non-localization asset");
            for (k, v) in &asset.dictionary {
                localization.add_text(k, &asset.language, v);
            }
            self.language_table.insert(id, asset.language.clone());
        }
        for id in assets.lately_unloaded_protocol("locals") {
            if let Some(name) = self.language_table.remove(id) {
                localization.remove_language(&name);
            }
        }
    }
}
