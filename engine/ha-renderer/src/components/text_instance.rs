use crate::{components::HaChangeFrequency, math::*};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use pest::{iterators::Pair, Parser};
use serde::{Deserialize, Serialize};
use std::str::Chars;

#[allow(clippy::upper_case_acronyms)]
mod parser {
    #[derive(Parser)]
    #[grammar = "components/rich_text.pest"]
    pub(crate) struct SentenceParser;
}

pub enum HaTextElementIter<'a> {
    NewLine {
        instance: &'a HaTextInstance,
        index: usize,
    },
    Text {
        instance: &'a HaTextInstance,
        index: usize,
        characters: Chars<'a>,
    },
    Done,
}

impl<'a> HaTextElementIter<'a> {
    pub fn new(instance: &'a HaTextInstance) -> Self {
        if let Some(fragment) = instance.content.fragments().get(0) {
            match fragment {
                HaTextFragment::NewLine => Self::NewLine { instance, index: 0 },
                HaTextFragment::Text { text, .. } => Self::Text {
                    instance,
                    index: 0,
                    characters: text.chars(),
                },
            }
        } else {
            Self::Done
        }
    }
}

impl<'a> Iterator for HaTextElementIter<'a> {
    type Item = HaTextElement<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::NewLine { instance, index } => {
                if let Some(fragment) = instance.content.fragments().get(*index + 1) {
                    match fragment {
                        HaTextFragment::NewLine => {
                            *self = Self::NewLine {
                                instance,
                                index: *index + 1,
                            };
                        }
                        HaTextFragment::Text { text, .. } => {
                            *self = Self::Text {
                                instance,
                                index: *index + 1,
                                characters: text.chars(),
                            };
                        }
                    }
                } else {
                    *self = Self::Done;
                }
                Some(HaTextElement::NewLine)
            }
            Self::Text {
                instance,
                index,
                characters,
            } => {
                if let Some(character) = characters.next() {
                    if let Some(HaTextFragment::Text {
                        size,
                        color,
                        outline,
                        thickness,
                        cursive,
                        ..
                    }) = instance.content.fragments().get(*index)
                    {
                        Some(HaTextElement::Glyph {
                            character,
                            size: size.unwrap_or_else(|| instance.size),
                            color: color.unwrap_or_else(|| instance.color),
                            outline: outline.unwrap_or_else(|| instance.outline),
                            thickness: thickness.unwrap_or_else(|| instance.thickness),
                            cursive: cursive.unwrap_or_else(|| instance.cursive),
                            wrapping: &instance.wrapping,
                        })
                    } else {
                        *self = Self::Done;
                        None
                    }
                } else if let Some(fragment) = instance.content.fragments().get(*index + 1) {
                    match fragment {
                        HaTextFragment::NewLine => {
                            *self = Self::NewLine {
                                instance,
                                index: *index + 1,
                            };
                        }
                        HaTextFragment::Text { text, .. } => {
                            *self = Self::Text {
                                instance,
                                index: *index + 1,
                                characters: text.chars(),
                            };
                        }
                    }
                    self.next()
                } else {
                    *self = Self::Done;
                    None
                }
            }
            Self::Done => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HaTextElement<'a> {
    Invalid,
    NewLine,
    Glyph {
        character: char,
        size: Scalar,
        color: Rgba,
        outline: Rgba,
        thickness: Scalar,
        cursive: Scalar,
        wrapping: &'a HaTextWrapping,
    },
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum HaTextFragment {
    NewLine,
    Text {
        #[serde(default)]
        text: String,
        #[serde(default)]
        size: Option<Scalar>,
        #[serde(default)]
        color: Option<Rgba>,
        #[serde(default)]
        outline: Option<Rgba>,
        #[serde(default)]
        thickness: Option<Scalar>,
        #[serde(default)]
        cursive: Option<Scalar>,
        #[serde(default)]
        wrapping: Option<HaTextWrapping>,
    },
}

impl Default for HaTextFragment {
    fn default() -> Self {
        Self::NewLine
    }
}

impl HaTextFragment {
    pub fn text(text: &str) -> Self {
        Self::Text {
            text: text.to_owned(),
            size: None,
            color: None,
            outline: None,
            thickness: None,
            cursive: None,
            wrapping: None,
        }
    }

    pub fn glyphs_count(&self) -> usize {
        match self {
            Self::NewLine => 0,
            Self::Text { text, .. } => text.len(),
        }
    }

    pub fn elements_count(&self) -> usize {
        match self {
            Self::NewLine => 1,
            Self::Text { text, .. } => text.len(),
        }
    }
}

#[derive(Ignite, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaTextWrapping {
    Character,
    Word,
    Set(String),
}

impl Default for HaTextWrapping {
    fn default() -> Self {
        Self::Character
    }
}

impl HaTextWrapping {
    pub fn can_wrap(&self, character: char) -> bool {
        match self {
            Self::Character => true,
            Self::Word => character.is_whitespace(),
            Self::Set(characters) => characters.chars().any(|c| c == character),
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum HaTextContent {
    Fragments(Vec<HaTextFragment>),
    RichTextTree(HaRichTextContent),
    RichText(String),
}

impl Default for HaTextContent {
    fn default() -> Self {
        Self::Fragments(vec![])
    }
}

impl HaTextContent {
    const EMPTY_FRAGMENTS: &'static [HaTextFragment] = &[];

    pub fn text(text: &str) -> Self {
        let count = text.lines().count();
        if count == 1 {
            return Self::Fragments(vec![HaTextFragment::text(text)]);
        }
        let mut result = Vec::with_capacity(count * 2 - 1);
        for (i, line) in text.lines().enumerate() {
            if i > 0 {
                result.push(HaTextFragment::NewLine);
            }
            result.push(HaTextFragment::text(line));
        }
        Self::Fragments(result)
    }

    pub fn normalize(self) -> Result<Self, String> {
        match self {
            Self::Fragments(_) => Ok(self),
            Self::RichTextTree(content) => Ok(Self::Fragments(content.build_fragments())),
            Self::RichText(text) => Ok(Self::Fragments(
                HaRichTextContent::new(&text)?.build_fragments(),
            )),
        }
    }

    pub fn normalize_lossy(self) -> Self {
        match self {
            Self::Fragments(_) => self,
            Self::RichTextTree(content) => Self::Fragments(content.build_fragments()),
            Self::RichText(text) => Self::Fragments(
                HaRichTextContent::new(&text)
                    .map(|content| content.build_fragments())
                    .unwrap_or_else(|error| vec![HaTextFragment::text(&error)]),
            ),
        }
    }

    pub fn fragments(&self) -> &[HaTextFragment] {
        match self {
            Self::Fragments(f) => f.as_slice(),
            _ => Self::EMPTY_FRAGMENTS,
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct HaTextInstance {
    font: String,
    content: HaTextContent,
    #[serde(default)]
    change_frequency: HaChangeFrequency,
    #[serde(default = "HaTextInstance::default_size")]
    size: Scalar,
    #[serde(default = "HaTextInstance::default_color")]
    color: Rgba,
    #[serde(default)]
    outline: Rgba,
    #[serde(default)]
    thickness: Scalar,
    #[serde(default)]
    cursive: Scalar,
    #[serde(default)]
    alignment: Vec2,
    #[serde(default)]
    pivot: Vec2,
    #[serde(default)]
    bounds_width: Option<Scalar>,
    #[serde(default)]
    bounds_height: Option<Scalar>,
    #[serde(default)]
    wrapping: HaTextWrapping,
    #[serde(default)]
    lines_extra_space: Scalar,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl Default for HaTextInstance {
    fn default() -> Self {
        Self {
            font: Default::default(),
            content: Default::default(),
            change_frequency: Default::default(),
            color: Self::default_color(),
            outline: Default::default(),
            thickness: Default::default(),
            cursive: Default::default(),
            size: Self::default_size(),
            alignment: Default::default(),
            pivot: Default::default(),
            bounds_width: None,
            bounds_height: None,
            wrapping: Default::default(),
            lines_extra_space: 0.0,
            dirty: true,
        }
    }
}

impl HaTextInstance {
    fn default_color() -> Rgba {
        Rgba::white()
    }

    fn default_size() -> Scalar {
        32.0
    }

    pub fn lines_count(&self) -> usize {
        1 + self
            .content
            .fragments()
            .iter()
            .filter(|fragment| matches!(fragment, HaTextFragment::NewLine))
            .count()
    }

    pub fn glyphs_count(&self) -> usize {
        self.content
            .fragments()
            .iter()
            .fold(0, |a, v| a + v.glyphs_count())
    }

    pub fn elements_count(&self) -> usize {
        self.content
            .fragments()
            .iter()
            .fold(0, |a, v| a + v.elements_count())
    }

    pub fn iter(&self) -> HaTextElementIter {
        HaTextElementIter::new(self)
    }

    pub fn font(&self) -> &str {
        &self.font
    }

    pub fn set_font(&mut self, font: impl ToString) {
        self.font = font.to_string();
        self.dirty = true;
    }

    pub fn content(&self) -> &HaTextContent {
        &self.content
    }

    pub fn set_content(&mut self, content: HaTextContent) -> Result<(), String> {
        self.content = content.normalize()?;
        self.dirty = true;
        Ok(())
    }

    pub fn set_content_lossy(&mut self, content: HaTextContent) {
        self.content = content.normalize_lossy();
        self.dirty = true;
    }

    pub fn change_frequency(&self) -> HaChangeFrequency {
        self.change_frequency
    }

    pub fn set_change_frequency(&mut self, frequency: HaChangeFrequency) {
        self.change_frequency = frequency;
        self.dirty = true;
    }

    pub fn color(&self) -> Rgba {
        self.color
    }

    pub fn set_color(&mut self, color: Rgba) {
        self.color = color;
        self.dirty = true;
    }

    pub fn thickness(&self) -> Scalar {
        self.thickness
    }

    pub fn set_thickness(&mut self, thickness: Scalar) {
        self.thickness = thickness;
        self.dirty = true;
    }

    pub fn cursive(&self) -> Scalar {
        self.cursive
    }

    pub fn set_cursive(&mut self, cursive: Scalar) {
        self.cursive = cursive;
        self.dirty = true;
    }

    pub fn size(&self) -> Scalar {
        self.size
    }

    pub fn set_size(&mut self, size: Scalar) {
        self.size = size;
        self.dirty = true;
    }

    pub fn alignment(&self) -> Vec2 {
        self.alignment
    }

    pub fn set_alignment(&mut self, alignment: Vec2) {
        self.alignment = Vec2::partial_max(Vec2::partial_min(alignment, 1.0), 0.0);
        self.dirty = true;
    }

    pub fn pivot(&self) -> Vec2 {
        self.pivot
    }

    pub fn set_pivot(&mut self, pivot: Vec2) {
        self.pivot = Vec2::partial_max(Vec2::partial_min(pivot, 1.0), 0.0);
        self.dirty = true;
    }

    pub fn bounds_width(&self) -> Option<Scalar> {
        self.bounds_width
    }

    pub fn set_bounds_width(&mut self, bounds_width: Option<Scalar>) {
        self.bounds_width = bounds_width;
        self.dirty = true;
    }

    pub fn bounds_height(&self) -> Option<Scalar> {
        self.bounds_height
    }

    pub fn set_bounds_height(&mut self, bounds_height: Option<Scalar>) {
        self.bounds_height = bounds_height;
        self.dirty = true;
    }

    pub fn wrapping(&self) -> &HaTextWrapping {
        &self.wrapping
    }

    pub fn set_wrapping(&mut self, wrapping: HaTextWrapping) {
        self.wrapping = wrapping;
        self.dirty = true;
    }

    pub fn lines_extra_space(&self) -> Scalar {
        self.lines_extra_space
    }

    pub fn set_lines_extra_space(&mut self, lines_extra_space: Scalar) {
        self.lines_extra_space = lines_extra_space;
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Prefab for HaTextInstance {
    fn post_from_prefab(&mut self) {
        self.content = std::mem::take(&mut self.content).normalize_lossy();
        self.alignment = Vec2::partial_max(Vec2::partial_min(self.alignment, 1.0), 0.0);
        self.pivot = Vec2::partial_max(Vec2::partial_min(self.pivot, 1.0), 0.0);
        self.dirty = true;
    }
}

impl PrefabComponent for HaTextInstance {}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HaRichTextOption {
    Size(Scalar),
    Color(Rgba),
    Outline(Rgba),
    Thickness(Scalar),
    Cursive(Scalar),
    Wrapping(HaTextWrapping),
}

impl HaRichTextOption {
    fn parse(pair: Pair<parser::Rule>) -> Self {
        match pair.as_rule() {
            parser::Rule::opt_size => {
                Self::Size(Self::parse_number(pair.into_inner().next().unwrap()))
            }
            parser::Rule::opt_color => {
                Self::Color(Self::parse_color(pair.into_inner().next().unwrap()))
            }
            parser::Rule::opt_outline => {
                Self::Outline(Self::parse_color(pair.into_inner().next().unwrap()))
            }
            parser::Rule::opt_thickness => {
                Self::Thickness(Self::parse_number(pair.into_inner().next().unwrap()))
            }
            parser::Rule::opt_cursive => {
                Self::Cursive(Self::parse_number(pair.into_inner().next().unwrap()))
            }
            parser::Rule::opt_wrapping => {
                Self::Wrapping(Self::parse_wrapping(pair.into_inner().next().unwrap()))
            }
            _ => unreachable!(),
        }
    }

    fn parse_number(pair: Pair<parser::Rule>) -> Scalar {
        pair.as_str().parse().unwrap()
    }

    fn parse_color(pair: Pair<parser::Rule>) -> Rgba {
        let mut pairs = pair.into_inner();
        let r = Self::parse_number(pairs.next().unwrap());
        let g = Self::parse_number(pairs.next().unwrap());
        let b = Self::parse_number(pairs.next().unwrap());
        let a = Self::parse_number(pairs.next().unwrap());
        Rgba::new(r, g, b, a)
    }

    fn parse_wrapping(pair: Pair<parser::Rule>) -> HaTextWrapping {
        match pair.as_rule() {
            parser::Rule::opt_wrapping_char => HaTextWrapping::Character,
            parser::Rule::opt_wrapping_word => HaTextWrapping::Word,
            parser::Rule::opt_wrapping_set => HaTextWrapping::Set(HaRichTextItem::parse_string(
                pair.into_inner().next().unwrap(),
            )),
            _ => unreachable!(),
        }
    }
}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HaRichTextItem {
    Group {
        #[serde(default)]
        options: Vec<HaRichTextOption>,
        content: HaRichTextContent,
    },
    Text(String),
    NewLine,
}

impl HaRichTextItem {
    fn collect_fragments(&self, opts: &RichTextOptions, output: &mut Vec<HaTextFragment>) {
        match self {
            Self::Group { options, content } => {
                let mut opts = opts.to_owned();
                for option in options {
                    match option {
                        HaRichTextOption::Size(v) => opts.size = Some(*v),
                        HaRichTextOption::Color(v) => opts.color = Some(*v),
                        HaRichTextOption::Outline(v) => opts.outline = Some(*v),
                        HaRichTextOption::Thickness(v) => opts.thickness = Some(*v),
                        HaRichTextOption::Cursive(v) => opts.cursive = Some(*v),
                        HaRichTextOption::Wrapping(v) => opts.wrapping = Some(v.to_owned()),
                    }
                }
                content.collect_fragments(&opts, output);
            }
            Self::Text(text) => output.push(HaTextFragment::Text {
                text: text.to_owned(),
                size: opts.size,
                color: opts.color,
                outline: opts.outline,
                thickness: opts.thickness,
                cursive: opts.cursive,
                wrapping: opts.wrapping.to_owned(),
            }),
            Self::NewLine => output.push(HaTextFragment::NewLine),
        }
    }

    fn fragments_count(&self) -> usize {
        match self {
            Self::Group { content, .. } => content.fragments_count(),
            Self::Text(_) | Self::NewLine => 1,
        }
    }

    fn parse(pair: Pair<parser::Rule>) -> Self {
        match pair.as_rule() {
            parser::Rule::group => {
                let mut pairs = pair.into_inner();
                let options = Self::parse_options(pairs.next().unwrap());
                let content = HaRichTextContent::parse(pairs.next().unwrap());
                Self::Group { options, content }
            }
            parser::Rule::string => Self::Text(Self::parse_string(pair)),
            parser::Rule::new_line => Self::NewLine,
            _ => unreachable!(),
        }
    }

    fn parse_options(pair: Pair<parser::Rule>) -> Vec<HaRichTextOption> {
        pair.into_inner()
            .map(|p| HaRichTextOption::parse(p.into_inner().next().unwrap()))
            .collect()
    }

    fn parse_string(pair: Pair<parser::Rule>) -> String {
        pair.into_inner().next().unwrap().as_str().to_owned()
    }
}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HaRichTextContent(pub Vec<HaRichTextItem>);

impl HaRichTextContent {
    pub fn new(text: &str) -> Result<Self, String> {
        match parser::SentenceParser::parse(parser::Rule::main, text) {
            Ok(mut ast) => {
                let pair = ast.next().unwrap();
                match pair.as_rule() {
                    parser::Rule::main => Ok(Self::parse(pair.into_inner().next().unwrap())),
                    _ => unreachable!(),
                }
            }
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn build_fragments(&self) -> Vec<HaTextFragment> {
        let mut result = Vec::with_capacity(self.fragments_count());
        self.collect_fragments(&RichTextOptions::default(), &mut result);
        result
    }

    fn collect_fragments(&self, options: &RichTextOptions, output: &mut Vec<HaTextFragment>) {
        for item in &self.0 {
            item.collect_fragments(options, output);
        }
    }

    fn fragments_count(&self) -> usize {
        self.0.iter().fold(0, |a, v| a + v.fragments_count())
    }

    fn parse(pair: Pair<parser::Rule>) -> Self {
        Self(
            pair.into_inner()
                .map(HaRichTextItem::parse)
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Debug, Default, Clone)]
struct RichTextOptions {
    size: Option<Scalar>,
    color: Option<Rgba>,
    outline: Option<Rgba>,
    thickness: Option<Scalar>,
    cursive: Option<Scalar>,
    wrapping: Option<HaTextWrapping>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rich_text() {
        let rich =
            HaRichTextContent::new("[s:1|c:(0,1,2,3)|`webgl2`[w:`_`|`text`#`renderer`]]").unwrap();
        println!("* Rich text: {:#?}", rich);
        assert_eq!(rich.fragments_count(), 4);
        let fragments = rich.build_fragments();
        println!("* Rich text fragments: {:#?}", fragments);
    }
}
