use crate::{components::HaChangeFrequency, math::*};
use core::{
    prefab::{Prefab, PrefabComponent},
    Scalar,
};
use pest::{iterators::Pair, Parser};
use serde::{Deserialize, Serialize};
use std::str::Chars;

#[allow(clippy::upper_case_acronyms)]
mod parser {
    #[derive(pest_derive::Parser)]
    #[grammar = "components/text_instance.pest"]
    pub(super) struct ContentParser;
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
        if let Some(fragment) = instance.content.0.get(0) {
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
                if let Some(fragment) = instance.content.0.get(*index + 1) {
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
                        params:
                            HaTextFragmentParams {
                                size,
                                color,
                                outline,
                                thickness,
                                cursive,
                                wrapping,
                            },
                        ..
                    }) = instance.content.0.get(*index)
                    {
                        Some(HaTextElement::Glyph {
                            character,
                            size: size.unwrap_or_else(|| instance.size),
                            color: color.unwrap_or_else(|| instance.color),
                            outline: outline.unwrap_or_else(|| instance.outline),
                            thickness: thickness.unwrap_or_else(|| instance.thickness),
                            cursive: cursive.unwrap_or_else(|| instance.cursive),
                            wrapping: wrapping.as_ref().unwrap_or(&instance.wrapping),
                        })
                    } else {
                        *self = Self::Done;
                        None
                    }
                } else if let Some(fragment) = instance.content.0.get(*index + 1) {
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HaTextFragmentParams {
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HaTextFragment {
    NewLine,
    Text {
        #[serde(default)]
        text: String,
        #[serde(default)]
        params: HaTextFragmentParams,
    },
}

impl Default for HaTextFragment {
    fn default() -> Self {
        Self::NewLine
    }
}

impl HaTextFragment {
    pub fn text(text: impl ToString) -> Self {
        Self::Text {
            text: text.to_string(),
            params: Default::default(),
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HaTextContent(pub Vec<HaTextFragment>);

impl HaTextContent {
    pub fn parse(text: &str) -> Result<Self, String> {
        match parser::ContentParser::parse(parser::Rule::main, text) {
            Ok(mut pairs) => {
                let mut stack = vec![];
                let mut result = vec![];
                for pair in pairs.next().unwrap().into_inner() {
                    match pair.as_rule() {
                        parser::Rule::text_string => {
                            let content = snailquote::unescape(pair.as_str()).unwrap();
                            for (index, part) in content.split(&['\r', '\n']).enumerate() {
                                if index > 0 {
                                    result.push(HaTextFragment::NewLine);
                                }
                                result.push(HaTextFragment::Text {
                                    text: part.to_owned(),
                                    params: stack.last().cloned().unwrap_or_default(),
                                });
                            }
                        }
                        parser::Rule::params_start => {
                            let mut result = stack.last().cloned().unwrap_or_default();
                            for pair in pair.into_inner() {
                                let pair = pair.into_inner().next().unwrap();
                                match pair.as_rule() {
                                    parser::Rule::size => {
                                        result.size = Some(Self::parse_number(
                                            pair.into_inner().next().unwrap(),
                                        ));
                                    }
                                    parser::Rule::color => {
                                        result.color = Some(Self::parse_color(
                                            pair.into_inner().next().unwrap(),
                                        ));
                                    }
                                    parser::Rule::outline => {
                                        result.outline = Some(Self::parse_color(
                                            pair.into_inner().next().unwrap(),
                                        ));
                                    }
                                    parser::Rule::thickness => {
                                        result.thickness = Some(Self::parse_number(
                                            pair.into_inner().next().unwrap(),
                                        ));
                                    }
                                    parser::Rule::cursive => {
                                        result.cursive = Some(Self::parse_number(
                                            pair.into_inner().next().unwrap(),
                                        ));
                                    }
                                    parser::Rule::wrapping => {
                                        let pair = pair.into_inner().next().unwrap();
                                        match pair.as_rule() {
                                            parser::Rule::wrapping_character => {
                                                result.wrapping = Some(HaTextWrapping::Character);
                                            }
                                            parser::Rule::wrapping_word => {
                                                result.wrapping = Some(HaTextWrapping::Word);
                                            }
                                            parser::Rule::wrapping_set => {
                                                let content = snailquote::unescape(
                                                    pair.into_inner().next().unwrap().as_str(),
                                                )
                                                .unwrap();
                                                result.wrapping =
                                                    Some(HaTextWrapping::Set(content));
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            stack.push(result);
                        }
                        parser::Rule::params_end => {
                            stack.pop();
                        }
                        _ => {}
                    }
                }
                Ok(Self(result))
            }
            Err(error) => Err(format!("{}", error)),
        }
    }

    fn parse_number(pair: Pair<parser::Rule>) -> Scalar {
        pair.as_str().parse::<Scalar>().unwrap()
    }

    fn parse_color(pair: Pair<parser::Rule>) -> Rgba {
        let mut result = Rgba::white();
        let mut pairs = pair.into_inner();
        if let Some(pair) = pairs.next() {
            result.r = Self::parse_number(pair);
        }
        if let Some(pair) = pairs.next() {
            result.g = Self::parse_number(pair);
        }
        if let Some(pair) = pairs.next() {
            result.b = Self::parse_number(pair);
        }
        if let Some(pair) = pairs.next() {
            result.a = Self::parse_number(pair);
        }
        result
    }
}

impl From<HaRichTextBuilder> for HaTextContent {
    fn from(other: HaRichTextBuilder) -> Self {
        other.build()
    }
}

impl<T: AsRef<str>> From<T> for HaTextContent {
    fn from(value: T) -> Self {
        Self::parse(value.as_ref()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HaTextWrapping {
    Character,
    Word,
    Set(String),
}

impl Default for HaTextWrapping {
    fn default() -> Self {
        Self::Word
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            .0
            .iter()
            .filter(|fragment| matches!(fragment, HaTextFragment::NewLine))
            .count()
    }

    pub fn glyphs_count(&self) -> usize {
        self.content.0.iter().fold(0, |a, v| a + v.glyphs_count())
    }

    pub fn elements_count(&self) -> usize {
        self.content.0.iter().fold(0, |a, v| a + v.elements_count())
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

    pub fn fragments(&self) -> &[HaTextFragment] {
        &self.content.0
    }

    pub fn set_content(&mut self, content: impl Into<HaTextContent>) {
        self.content = content.into();
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
        self.alignment = Vec2::partial_max(Vec2::partial_min(self.alignment, 1.0), 0.0);
        self.pivot = Vec2::partial_max(Vec2::partial_min(self.pivot, 1.0), 0.0);
        self.dirty = true;
    }
}
impl PrefabComponent for HaTextInstance {}

#[derive(Debug, Default, Clone)]
pub struct HaRichTextBuilder {
    size: Vec<Scalar>,
    color: Vec<Rgba>,
    outline: Vec<Rgba>,
    thickness: Vec<Scalar>,
    cursive: Vec<Scalar>,
    wrapping: Vec<HaTextWrapping>,
    fragments: Vec<HaTextFragment>,
}

macro_rules! impl_rich_text_group {
    ($name:ident : $type:ty) => {
        pub fn $name<F>(mut self, value: $type, f: F) -> Self
        where
            F: FnOnce(Self) -> Self,
        {
            self.$name.push(value);
            self = f(self);
            self.$name.pop();
            self
        }
    };
}

impl HaRichTextBuilder {
    pub fn with_capacity(depth: usize, count: usize) -> Self {
        Self {
            size: Vec::with_capacity(depth),
            color: Vec::with_capacity(depth),
            outline: Vec::with_capacity(depth),
            thickness: Vec::with_capacity(depth),
            cursive: Vec::with_capacity(depth),
            wrapping: Vec::with_capacity(depth),
            fragments: Vec::with_capacity(count),
        }
    }

    pub fn build(self) -> HaTextContent {
        HaTextContent(self.fragments)
    }

    impl_rich_text_group!(size: Scalar);
    impl_rich_text_group!(color: Rgba);
    impl_rich_text_group!(outline: Rgba);
    impl_rich_text_group!(thickness: Scalar);
    impl_rich_text_group!(cursive: Scalar);
    impl_rich_text_group!(wrapping: HaTextWrapping);

    pub fn new_line(mut self) -> Self {
        self.fragments.push(HaTextFragment::NewLine);
        self
    }

    pub fn text(mut self, value: impl ToString) -> Self {
        let mut first = true;
        for line in value.to_string().split(|c| c == '\r' || c == '\n') {
            if first {
                first = false;
            } else {
                self.fragments.push(HaTextFragment::NewLine);
            }
            self.fragments.push(HaTextFragment::Text {
                text: line.to_owned(),
                params: HaTextFragmentParams {
                    size: self.size.last().copied(),
                    color: self.color.last().copied(),
                    outline: self.outline.last().copied(),
                    thickness: self.thickness.last().copied(),
                    cursive: self.cursive.last().copied(),
                    wrapping: self.wrapping.last().cloned(),
                },
            });
        }
        self
    }
}

#[macro_export]
macro_rules! rich_text {
    ( @item($builder:expr) => [size ( $value:expr ) $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, size, $value) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [color = $value:ident $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, color, $crate::math::Rgba::$value()) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [color ( $value:expr ) $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, color, $value) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [outline ( $value:expr ) $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, outline, $value) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [thickness ( $value:expr ) $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, thickness, $value) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [cursive ( $value:expr ) $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, cursive, $value) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [wrapping ( $value:expr ) $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, wrapping, $value) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [wrapping = character $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, wrapping, $crate::components::text_instance::HaTextWrapping::Character) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [wrapping = word $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, wrapping, $crate::components::text_instance::HaTextWrapping::Word) => ( $($item)* ) }
    };
    ( @item($builder:expr) => [wrapping = $value:literal $( $item:tt )* ] ) => {
        $crate::rich_text! { @item($builder, wrapping, $crate::components::text_instance::HaTextWrapping::Set($value.to_string())) => ( $($item)* ) }
    };
    ( @item($builder:expr, $param:ident, $value:expr ) => ( $( $item:tt )* ) ) => {
        $builder.$param($value.into(), |mut builder| {
            $(
                builder = $crate::rich_text! { @item(builder) => $item };
            )*
            builder
        })
    };
    ( @item($builder:expr) => newline ) => ( $builder.new_line() );
    ( @item($builder:expr) => $value:literal ) => ( $builder.text($value) );
    ( @item($builder:expr) => { $value:expr } ) => ( $builder.text($value) );
    (
        $( $item:tt )+
    ) => {
        {
            let mut builder = $crate::components::text_instance::HaRichTextBuilder::default();
            $(
                builder = $crate::rich_text! { @item(builder) => $item };
            )+
            builder.build()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rich_text() {
        let a = HaRichTextBuilder::default()
            .size(64.0, |b| {
                b.text("webgl2").color(Rgba::red(), |b| {
                    b.wrapping(HaTextWrapping::Character, |b| {
                        b.text("text").new_line().text("renderer").text(42)
                    })
                })
            })
            .build();
        let b = rich_text! {
            [size(64.0)
                "webgl2"
                [color=red
                    [wrapping=character
                        "text"
                        newline
                        "renderer"
                        {42}
                    ]
                ]
            ]
        };
        let c =
            HaTextContent::parse("[s=64]webgl2[c=(1,0,0) w=c]text\nrenderer[|]42[/][/]").unwrap();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
