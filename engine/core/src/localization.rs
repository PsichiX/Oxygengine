use crate::{
    app::AppBuilder,
    assets::{
        asset::AssetId, database::AssetsDatabase, protocols::localization::LocalizationAsset,
    },
    ecs::{
        pipeline::{PipelineBuilder, PipelineBuilderError},
        Universe,
    },
};
use pest::{iterators::Pair, Parser};
use std::{collections::HashMap, fmt::Write};

#[allow(clippy::upper_case_acronyms)]
mod parser {
    #[derive(Parser)]
    #[grammar = "localization.pest"]
    pub(super) struct SentenceParser;
}

#[derive(Default)]
pub struct Localization {
    default_language: Option<String>,
    current_language: Option<String>,
    /// { text id: { language: text format } }
    map: HashMap<String, HashMap<String, String>>,
}

impl Localization {
    pub fn default_language(&self) -> Option<&str> {
        self.default_language.as_deref()
    }

    pub fn set_default_language(&mut self, value: Option<String>) {
        self.default_language = value;
    }

    pub fn current_language(&self) -> Option<&str> {
        self.current_language.as_deref()
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
            match parser::SentenceParser::parse(parser::Rule::sentence, text_format) {
                Ok(mut ast) => {
                    let pair = ast.next().unwrap();
                    match pair.as_rule() {
                        parser::Rule::sentence => Ok(Self::parse_sentence_inner(pair, params)),
                        _ => unreachable!(),
                    }
                }
                Err(error) => Err(error.to_string()),
            }
        } else {
            Err(format!("There is no text format for id: {}", id))
        }
    }

    fn parse_sentence_inner(pair: Pair<parser::Rule>, params: &[(&str, &str)]) -> String {
        let mut result = String::new();
        for p in pair.into_inner() {
            match p.as_rule() {
                parser::Rule::text => result.push_str(&p.as_str().replace("\\|", "|")),
                parser::Rule::identifier => {
                    let ident = p.as_str();
                    if let Some((_, v)) = params.iter().find(|(id, _)| id == &ident) {
                        result.push_str(v);
                    } else {
                        write!(result, "{{@{}}}", ident).unwrap();
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
    ($res:expr, $text:expr, $( $id:ident => $value:expr ),*) => {
        $crate::localization::Localization::format_text(
            &$res,
            $text,
            &[ $( (stringify!($id), &$value.to_string()) ),* ]
        )
    }
}

#[derive(Default)]
pub struct LocalizationSystemCache {
    language_table: HashMap<AssetId, String>,
}

pub type LocalizationSystemResources<'a> = (
    &'a AssetsDatabase,
    &'a mut Localization,
    &'a mut LocalizationSystemCache,
);

pub fn localization_system(universe: &mut Universe) {
    let (assets, mut localization, mut cache) =
        universe.query_resources::<LocalizationSystemResources>();

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
        cache.language_table.insert(id, asset.language.clone());
    }
    for id in assets.lately_unloaded_protocol("locals") {
        if let Some(name) = cache.language_table.remove(id) {
            localization.remove_language(&name);
        }
    }
}

pub fn bundle_installer<PB, PMS>(
    builder: &mut AppBuilder<PB>,
    _: (),
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(Localization::default());
    builder.install_resource(LocalizationSystemCache::default());
    builder.install_system::<LocalizationSystemResources>(
        "localization",
        localization_system,
        &[],
    )?;
    Ok(())
}
