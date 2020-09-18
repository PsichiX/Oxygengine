#![allow(clippy::many_single_char_names)]

extern crate oxygengine_composite_renderer as renderer;
#[macro_use]
extern crate oxygengine_core as core;

use core::{
    assets::{asset::AssetID, database::AssetsDatabase},
    error::*,
    Scalar,
};
use js_sys::{Array, Uint8Array};
use renderer::{
    composite_renderer::*, font_asset_protocol::FontAsset, font_face_asset_protocol::FontFaceAsset,
    math::*, png_image_asset_protocol::PngImageAsset,
};
use std::collections::HashMap;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

pub mod prelude {
    pub use crate::*;
}

pub fn get_canvas_by_id(id: &str) -> HtmlCanvasElement {
    let document = window().document().expect("Could not get window document");
    let canvas = document
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("no `{}` canvas in document", id));
    canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap()
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub struct WebCompositeRenderer {
    state: RenderState,
    view_size: Vec2,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    images_cache: HashMap<String, HtmlImageElement>,
    images_table: HashMap<AssetID, String>,
    fontfaces_cache: HashMap<String, FontFace>,
    fontfaces_table: HashMap<AssetID, String>,
    cached_image_smoothing: Option<bool>,
    surfaces_cache: HashMap<String, (HtmlCanvasElement, CanvasRenderingContext2d)>,
}

unsafe impl Send for WebCompositeRenderer {}
unsafe impl Sync for WebCompositeRenderer {}

impl WebCompositeRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Self {
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        Self {
            state: RenderState::default(),
            view_size: Vec2::zero(),
            canvas,
            context,
            images_cache: Default::default(),
            images_table: Default::default(),
            fontfaces_cache: Default::default(),
            fontfaces_table: Default::default(),
            cached_image_smoothing: None,
            surfaces_cache: Default::default(),
        }
    }

    pub fn with_state(canvas: HtmlCanvasElement, state: RenderState) -> Self {
        let mut result = Self::new(canvas);
        *result.state_mut() = state;
        result
    }

    fn execute_with<'a, I>(
        &self,
        context: &CanvasRenderingContext2d,
        mut current_alpha: Scalar,
        commands: I,
    ) -> Result<(usize, usize)>
    where
        I: IntoIterator<Item = Command<'a>>,
    {
        let mut render_ops = 0;
        let mut renderables = 0;
        let mut alpha_stack = vec![current_alpha];
        for command in commands {
            match command {
                Command::Draw(renderable) => match renderable {
                    Renderable::None => {}
                    Renderable::Rectangle(rectangle) => {
                        context.set_fill_style(&rectangle.color.to_string().into());
                        context.fill_rect(
                            rectangle.rect.x.into(),
                            rectangle.rect.y.into(),
                            rectangle.rect.w.into(),
                            rectangle.rect.h.into(),
                        );
                        render_ops += 2;
                        renderables += 1;
                    }
                    Renderable::FullscreenRectangle(color) => {
                        context.save();
                        drop(context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0));
                        context.set_fill_style(&color.to_string().into());
                        context.fill_rect(
                            0.0,
                            0.0,
                            self.view_size.x.into(),
                            self.view_size.y.into(),
                        );
                        context.restore();
                        render_ops += 2;
                        renderables += 1;
                    }
                    Renderable::Text(text) => {
                        context.set_fill_style(&text.color.to_string().into());
                        context.set_font(&format!("{}px \"{}\"", text.size, &text.font));
                        context.set_text_align(match text.align {
                            TextAlign::Left => "left",
                            TextAlign::Center => "center",
                            TextAlign::Right => "right",
                        });
                        context.set_text_baseline(match text.baseline {
                            TextBaseLine::Top => "top",
                            TextBaseLine::Middle => "middle",
                            TextBaseLine::Bottom => "bottom",
                            TextBaseLine::Alphabetic => "alphabetic",
                            TextBaseLine::Hanging => "hanging",
                        });
                        for (i, line) in text.text.lines().enumerate() {
                            if let Some(max_width) = text.max_width {
                                drop(context.fill_text_with_max_width(
                                    line,
                                    text.position.x.into(),
                                    (text.position.y + text.size * i as Scalar).into(),
                                    max_width.into(),
                                ));
                            } else {
                                drop(context.fill_text(
                                    line,
                                    text.position.x.into(),
                                    (text.position.y + text.size * i as Scalar).into(),
                                ));
                            }
                            render_ops += 1;
                        }
                        render_ops += 3;
                        renderables += 1;
                    }
                    Renderable::Path(path) => {
                        let mut ops = 0;
                        context.begin_path();
                        for element in &path.elements {
                            match element {
                                PathElement::MoveTo(pos) => {
                                    context.move_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::LineTo(pos) => {
                                    context.line_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::BezierCurveTo(cpa, cpb, pos) => {
                                    context.bezier_curve_to(
                                        cpa.x.into(),
                                        cpa.y.into(),
                                        cpb.x.into(),
                                        cpb.y.into(),
                                        pos.x.into(),
                                        pos.y.into(),
                                    );
                                    ops += 1;
                                }
                                PathElement::QuadraticCurveTo(cp, pos) => {
                                    context.quadratic_curve_to(
                                        cp.x.into(),
                                        cp.y.into(),
                                        pos.x.into(),
                                        pos.y.into(),
                                    );
                                    ops += 1;
                                }
                                PathElement::Arc(pos, r, a) => {
                                    drop(context.arc(
                                        pos.x.into(),
                                        pos.y.into(),
                                        (*r).into(),
                                        a.start.into(),
                                        a.end.into(),
                                    ));
                                    ops += 1;
                                }
                                PathElement::Ellipse(pos, r, rot, a) => {
                                    drop(context.ellipse(
                                        pos.x.into(),
                                        pos.y.into(),
                                        r.x.into(),
                                        r.y.into(),
                                        (*rot).into(),
                                        a.start.into(),
                                        a.end.into(),
                                    ));
                                    ops += 1;
                                }
                                PathElement::Rectangle(rect) => {
                                    context.rect(
                                        rect.x.into(),
                                        rect.y.into(),
                                        rect.w.into(),
                                        rect.h.into(),
                                    );
                                    ops += 1;
                                }
                            }
                        }
                        context.set_fill_style(&path.color.to_string().into());
                        context.close_path();
                        context.fill();
                        render_ops += 4 + ops;
                        renderables += 1;
                    }
                    Renderable::Mask(mask) => {
                        let mut ops = 0;
                        context.begin_path();
                        for element in &mask.elements {
                            match element {
                                PathElement::MoveTo(pos) => {
                                    context.move_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::LineTo(pos) => {
                                    context.line_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::BezierCurveTo(cpa, cpb, pos) => {
                                    context.bezier_curve_to(
                                        cpa.x.into(),
                                        cpa.y.into(),
                                        cpb.x.into(),
                                        cpb.y.into(),
                                        pos.x.into(),
                                        pos.y.into(),
                                    );
                                    ops += 1;
                                }
                                PathElement::QuadraticCurveTo(cp, pos) => {
                                    context.quadratic_curve_to(
                                        cp.x.into(),
                                        cp.y.into(),
                                        pos.x.into(),
                                        pos.y.into(),
                                    );
                                    ops += 1;
                                }
                                PathElement::Arc(pos, r, a) => {
                                    drop(context.arc(
                                        pos.x.into(),
                                        pos.y.into(),
                                        (*r).into(),
                                        a.start.into(),
                                        a.end.into(),
                                    ));
                                    ops += 1;
                                }
                                PathElement::Ellipse(pos, r, rot, a) => {
                                    drop(context.ellipse(
                                        pos.x.into(),
                                        pos.y.into(),
                                        r.x.into(),
                                        r.y.into(),
                                        (*rot).into(),
                                        a.start.into(),
                                        a.end.into(),
                                    ));
                                    ops += 1;
                                }
                                PathElement::Rectangle(rect) => {
                                    context.rect(
                                        rect.x.into(),
                                        rect.y.into(),
                                        rect.w.into(),
                                        rect.h.into(),
                                    );
                                    ops += 1;
                                }
                            }
                        }
                        context.close_path();
                        context.clip();
                        render_ops += 3 + ops;
                    }
                    Renderable::Image(image) => {
                        let path: &str = &image.image;
                        if let Some(elm) = self.images_cache.get(path) {
                            let mut src = if let Some(src) = image.source {
                                src
                            } else {
                                Rect {
                                    x: 0.0,
                                    y: 0.0,
                                    w: elm.width() as Scalar,
                                    h: elm.height() as Scalar,
                                }
                            };
                            src.x += self.state.image_source_inner_margin;
                            src.y += self.state.image_source_inner_margin;
                            src.w -= self.state.image_source_inner_margin * 2.0;
                            src.h -= self.state.image_source_inner_margin * 2.0;
                            let dst = if let Some(dst) = image.destination {
                                dst
                            } else {
                                Rect {
                                    x: 0.0,
                                    y: 0.0,
                                    w: src.w,
                                    h: src.h,
                                }
                            }
                            .align(image.alignment);
                            drop(context
                                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                                    elm,
                                    src.x.into(),
                                    src.y.into(),
                                    src.w.into(),
                                    src.h.into(),
                                    dst.x.into(),
                                    dst.y.into(),
                                    dst.w.into(),
                                    dst.h.into(),
                                ));
                            render_ops += 1;
                            renderables += 1;
                        } else if let Some((elm, _)) = self.surfaces_cache.get(path) {
                            let mut src = if let Some(src) = image.source {
                                src
                            } else {
                                Rect {
                                    x: 0.0,
                                    y: 0.0,
                                    w: elm.width() as Scalar,
                                    h: elm.height() as Scalar,
                                }
                            };
                            src.x += self.state.image_source_inner_margin;
                            src.y += self.state.image_source_inner_margin;
                            src.w -= self.state.image_source_inner_margin * 2.0;
                            src.h -= self.state.image_source_inner_margin * 2.0;
                            let dst = if let Some(dst) = image.destination {
                                dst
                            } else {
                                Rect {
                                    x: 0.0,
                                    y: 0.0,
                                    w: src.w,
                                    h: src.h,
                                }
                            }
                            .align(image.alignment);
                            drop(context
                                .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                                    elm,
                                    src.x.into(),
                                    src.y.into(),
                                    src.w.into(),
                                    src.h.into(),
                                    dst.x.into(),
                                    dst.y.into(),
                                    dst.w.into(),
                                    dst.h.into(),
                                ));
                            render_ops += 1;
                            renderables += 1;
                        }
                    }
                    Renderable::Triangles(triangles) => {
                        let path: &str = &triangles.image;
                        if let Some(elm) = self.images_cache.get(path) {
                            let w = elm.width() as Scalar;
                            let h = elm.height() as Scalar;
                            let s = Vec2::new(w, h);
                            for triangle in triangles.indices.chunks(3) {
                                if triangle.len() == 3 {
                                    context.save();
                                    let a = triangles.vertices[triangle[0]];
                                    let b = triangles.vertices[triangle[1]];
                                    let c = triangles.vertices[triangle[2]];
                                    context.begin_path();
                                    context.move_to(a.0.x.into(), a.0.y.into());
                                    context.line_to(b.0.x.into(), b.0.y.into());
                                    context.line_to(c.0.x.into(), c.0.y.into());
                                    context.close_path();
                                    context.clip();
                                    let s0 = a.1 * s;
                                    let s1 = b.1 * s;
                                    let s2 = c.1 * s;
                                    let denom = s0.x * (s2.y - s1.y) - s1.x * s2.y
                                        + s2.x * s1.y
                                        + (s1.x - s2.x) * s0.y;
                                    if denom.abs() > 0.0 {
                                        let m11 = -(s0.y * (c.0.x - b.0.x) - s1.y * c.0.x
                                            + s2.y * b.0.x
                                            + (s1.y - s2.y) * a.0.x)
                                            / denom;
                                        let m12 = (s1.y * c.0.y + s0.y * (b.0.y - c.0.y)
                                            - s2.y * b.0.y
                                            + (s2.y - s1.y) * a.0.y)
                                            / denom;
                                        let m21 = (s0.x * (c.0.x - b.0.x) - s1.x * c.0.x
                                            + s2.x * b.0.x
                                            + (s1.x - s2.x) * a.0.x)
                                            / denom;
                                        let m22 = -(s1.x * c.0.y + s0.x * (b.0.y - c.0.y)
                                            - s2.x * b.0.y
                                            + (s2.x - s1.x) * a.0.y)
                                            / denom;
                                        let dx = (s0.x * (s2.y * b.0.x - s1.y * c.0.x)
                                            + s0.y * (s1.x * c.0.x - s2.x * b.0.x)
                                            + (s2.x * s1.y - s1.x * s2.y) * a.0.x)
                                            / denom;
                                        let dy = (s0.x * (s2.y * b.0.y - s1.y * c.0.y)
                                            + s0.y * (s1.x * c.0.y - s2.x * b.0.y)
                                            + (s2.x * s1.y - s1.x * s2.y) * a.0.y)
                                            / denom;
                                        drop(context.transform(
                                            m11.into(),
                                            m12.into(),
                                            m21.into(),
                                            m22.into(),
                                            dx.into(),
                                            dy.into(),
                                        ));
                                        drop(
                                            context
                                                .draw_image_with_html_image_element(elm, 0.0, 0.0),
                                        );
                                        render_ops += 2;
                                        renderables += 1;
                                    }
                                    context.restore();
                                    render_ops += 8;
                                }
                            }
                        } else if let Some((elm, _)) = self.surfaces_cache.get(path) {
                            let w = elm.width() as Scalar;
                            let h = elm.height() as Scalar;
                            let s = Vec2::new(w, h);
                            for triangle in triangles.indices.chunks(3) {
                                if triangle.len() == 3 {
                                    context.save();
                                    let a = triangles.vertices[triangle[0]];
                                    let b = triangles.vertices[triangle[1]];
                                    let c = triangles.vertices[triangle[2]];
                                    context.begin_path();
                                    context.move_to(a.0.x.into(), a.0.y.into());
                                    context.line_to(b.0.x.into(), b.0.y.into());
                                    context.line_to(c.0.x.into(), c.0.y.into());
                                    context.clip();
                                    let s0 = a.1 * s;
                                    let s1 = b.1 * s;
                                    let s2 = c.1 * s;
                                    let denom = s0.x * (s2.y - s1.y) - s1.x * s2.y
                                        + s2.x * s1.y
                                        + (s1.x - s2.x) * s0.y;
                                    if denom.abs() > 0.0 {
                                        let m11 = -(s0.y * (c.0.x - b.0.x) - s1.y * c.0.x
                                            + s2.y * b.0.x
                                            + (s1.y - s2.y) * a.0.x)
                                            / denom;
                                        let m12 = (s1.y * c.0.y + s0.y * (b.0.y - c.0.y)
                                            - s2.y * b.0.y
                                            + (s2.y - s1.y) * a.0.y)
                                            / denom;
                                        let m21 = (s0.x * (c.0.x - b.0.x) - s1.x * c.0.x
                                            + s2.x * b.0.x
                                            + (s1.x - s2.x) * a.0.x)
                                            / denom;
                                        let m22 = -(s1.x * c.0.y + s0.x * (b.0.y - c.0.y)
                                            - s2.x * b.0.y
                                            + (s2.x - s1.x) * a.0.y)
                                            / denom;
                                        let dx = (s0.x * (s2.y * b.0.x - s1.y * c.0.x)
                                            + s0.y * (s1.x * c.0.x - s2.x * b.0.x)
                                            + (s2.x * s1.y - s1.x * s2.y) * a.0.x)
                                            / denom;
                                        let dy = (s0.x * (s2.y * b.0.y - s1.y * c.0.y)
                                            + s0.y * (s1.x * c.0.y - s2.x * b.0.y)
                                            + (s2.x * s1.y - s1.x * s2.y) * a.0.y)
                                            / denom;
                                        drop(context.transform(
                                            m11.into(),
                                            m12.into(),
                                            m21.into(),
                                            m22.into(),
                                            dx.into(),
                                            dy.into(),
                                        ));
                                        drop(
                                            context
                                                .draw_image_with_html_canvas_element(elm, 0.0, 0.0),
                                        );
                                        render_ops += 2;
                                        renderables += 1;
                                    }
                                    context.restore();
                                    render_ops += 8;
                                }
                            }
                        }
                    }
                    Renderable::Commands(commands) => {
                        let (o, r) =
                            self.execute_with(context, current_alpha, commands.into_iter())?;
                        render_ops += o;
                        renderables += r;
                    }
                },
                Command::Stroke(line_width, renderable) => match renderable {
                    Renderable::None => {}
                    Renderable::Rectangle(rectangle) => {
                        context.set_stroke_style(&rectangle.color.to_string().into());
                        context.set_line_width(line_width.into());
                        context.stroke_rect(
                            rectangle.rect.x.into(),
                            rectangle.rect.y.into(),
                            rectangle.rect.w.into(),
                            rectangle.rect.h.into(),
                        );
                        render_ops += 3;
                        renderables += 1;
                    }
                    Renderable::FullscreenRectangle(color) => {
                        context.save();
                        drop(context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0));
                        context.set_stroke_style(&color.to_string().into());
                        context.set_line_width(line_width.into());
                        context.fill_rect(
                            0.0,
                            0.0,
                            self.view_size.x.into(),
                            self.view_size.y.into(),
                        );
                        context.restore();
                        render_ops += 2;
                        renderables += 1;
                    }
                    Renderable::Text(text) => {
                        context.set_stroke_style(&text.color.to_string().into());
                        context.set_line_width(line_width.into());
                        context.set_font(&format!("{}px \"{}\"", text.size, &text.font));
                        context.set_text_align(match text.align {
                            TextAlign::Left => "left",
                            TextAlign::Center => "center",
                            TextAlign::Right => "right",
                        });
                        context.set_text_baseline(match text.baseline {
                            TextBaseLine::Top => "top",
                            TextBaseLine::Middle => "middle",
                            TextBaseLine::Bottom => "bottom",
                            TextBaseLine::Alphabetic => "alphabetic",
                            TextBaseLine::Hanging => "hanging",
                        });
                        for (i, line) in text.text.lines().enumerate() {
                            if let Some(max_width) = text.max_width {
                                drop(context.stroke_text_with_max_width(
                                    line,
                                    text.position.x.into(),
                                    (text.position.y + text.size * i as Scalar).into(),
                                    max_width.into(),
                                ));
                            } else {
                                drop(context.stroke_text(
                                    line,
                                    text.position.x.into(),
                                    (text.position.y + text.size * i as Scalar).into(),
                                ));
                            }
                            render_ops += 1;
                        }
                        render_ops += 4;
                        renderables += 1;
                    }
                    Renderable::Path(path) => {
                        let mut ops = 0;
                        context.begin_path();
                        for element in &path.elements {
                            match element {
                                PathElement::MoveTo(pos) => {
                                    context.move_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::LineTo(pos) => {
                                    context.line_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::BezierCurveTo(cpa, cpb, pos) => {
                                    context.bezier_curve_to(
                                        cpa.x.into(),
                                        cpa.y.into(),
                                        cpb.x.into(),
                                        cpb.y.into(),
                                        pos.x.into(),
                                        pos.y.into(),
                                    );
                                    ops += 1;
                                }
                                PathElement::QuadraticCurveTo(cp, pos) => {
                                    context.quadratic_curve_to(
                                        cp.x.into(),
                                        cp.y.into(),
                                        pos.x.into(),
                                        pos.y.into(),
                                    );
                                    ops += 1;
                                }
                                PathElement::Arc(pos, r, a) => {
                                    drop(context.arc(
                                        pos.x.into(),
                                        pos.y.into(),
                                        (*r).into(),
                                        a.start.into(),
                                        a.end.into(),
                                    ));
                                    ops += 1;
                                }
                                PathElement::Ellipse(pos, r, rot, a) => {
                                    drop(context.ellipse(
                                        pos.x.into(),
                                        pos.y.into(),
                                        r.x.into(),
                                        r.y.into(),
                                        (*rot).into(),
                                        a.start.into(),
                                        a.end.into(),
                                    ));
                                    ops += 1;
                                }
                                PathElement::Rectangle(rect) => {
                                    context.rect(
                                        rect.x.into(),
                                        rect.y.into(),
                                        rect.w.into(),
                                        rect.h.into(),
                                    );
                                    ops += 1;
                                }
                            }
                        }
                        context.set_stroke_style(&path.color.to_string().into());
                        context.set_line_width(line_width.into());
                        context.close_path();
                        context.stroke();
                        render_ops += 5 + ops;
                        renderables += 1;
                    }
                    Renderable::Mask(_) => error!("Trying to make stroked mask"),
                    Renderable::Image(image) => {
                        error!("Trying to render stroked image: {}", image.image)
                    }
                    Renderable::Triangles(triangles) => {
                        context.save();
                        context.set_stroke_style(&triangles.color.to_string().into());
                        context.set_line_width(line_width.into());
                        for triangle in triangles.indices.chunks(3) {
                            if triangle.len() == 3 {
                                let a = triangles.vertices[triangle[0]];
                                let b = triangles.vertices[triangle[1]];
                                let c = triangles.vertices[triangle[2]];
                                context.begin_path();
                                context.move_to(a.0.x.into(), a.0.y.into());
                                context.line_to(b.0.x.into(), b.0.y.into());
                                context.line_to(c.0.x.into(), c.0.y.into());
                                context.close_path();
                                context.stroke();
                                render_ops += 6;
                                renderables += 1;
                            }
                        }
                        context.restore();
                        render_ops += 4;
                    }
                    Renderable::Commands(commands) => {
                        error!("Trying to render stroked subcommands: {:#?}", commands)
                    }
                },
                Command::Transform(a, b, c, d, e, f) => {
                    drop(context.transform(
                        a.into(),
                        b.into(),
                        c.into(),
                        d.into(),
                        e.into(),
                        f.into(),
                    ));
                    render_ops += 1;
                }
                Command::Effect(effect) => {
                    drop(context.set_global_composite_operation(&effect.to_string()));
                    render_ops += 1;
                }
                Command::Alpha(alpha) => {
                    current_alpha = alpha_stack.last().copied().unwrap_or(1.0) * alpha;
                    context.set_global_alpha(current_alpha.max(0.0).min(1.0).into());
                    render_ops += 1;
                }
                Command::Store => {
                    alpha_stack.push(current_alpha);
                    context.save();
                    render_ops += 1;
                }
                Command::Restore => {
                    current_alpha = alpha_stack.pop().unwrap_or(1.0);
                    context.restore();
                    render_ops += 1;
                }
                _ => {}
            }
        }
        Ok((render_ops, renderables))
    }
}

impl CompositeRenderer for WebCompositeRenderer {
    fn execute<'a, I>(&mut self, commands: I) -> Result<(usize, usize)>
    where
        I: IntoIterator<Item = Command<'a>>,
    {
        self.execute_with(&self.context, 1.0, commands)
    }

    fn images_count(&self) -> usize {
        self.images_cache.len()
    }

    fn fontfaces_count(&self) -> usize {
        self.fontfaces_cache.len()
    }

    fn surfaces_count(&self) -> usize {
        self.surfaces_cache.len()
    }

    fn state(&self) -> &RenderState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut RenderState {
        &mut self.state
    }

    fn view_size(&self) -> Vec2 {
        self.view_size
    }

    fn update_state(&mut self) {
        let w = self.canvas.client_width();
        let h = self.canvas.client_height();
        if (self.view_size.x - w as Scalar).abs() > 1.0
            || (self.view_size.y - h as Scalar).abs() > 1.0
        {
            self.canvas.set_width(w as u32);
            self.canvas.set_height(h as u32);
            self.view_size = Vec2::new(w as Scalar, h as Scalar);
            self.cached_image_smoothing = None;
        }
        if self.cached_image_smoothing.is_none()
            || self.cached_image_smoothing.unwrap() != self.state.image_smoothing
        {
            self.context
                .set_image_smoothing_enabled(self.state.image_smoothing);
            self.cached_image_smoothing = Some(self.state.image_smoothing);
        }
    }

    fn update_cache(&mut self, assets: &AssetsDatabase) {
        for id in assets.lately_loaded_protocol("png") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded png asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<PngImageAsset>()
                .expect("trying to use non-png asset");
            let width = asset.width() as u32;
            let height = asset.height() as u32;
            let buffer = Uint8Array::from(asset.bytes());
            let buffer_val: &JsValue = buffer.as_ref();
            let parts = Array::new_with_length(1);
            parts.set(0, buffer_val.clone());
            let blob = Blob::new_with_u8_array_sequence(parts.as_ref()).unwrap();
            let elm = HtmlImageElement::new_with_width_and_height(width, height).unwrap();
            elm.set_src(&Url::create_object_url_with_blob(&blob).unwrap());
            self.images_cache.insert(path.clone(), elm);
            self.images_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("png") {
            if let Some(path) = self.images_table.remove(id) {
                self.images_cache.remove(&path);
            }
        }
        for id in assets.lately_loaded_protocol("fontface") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded font face asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<FontFaceAsset>()
                .expect("trying to use non-font-face asset");
            let font_asset = assets
                .asset_by_id(asset.font_asset())
                .expect("trying to use not loaded font asset");
            let font_asset = font_asset
                .get::<FontAsset>()
                .expect("trying to use non-font asset");
            let mut descriptors = FontFaceDescriptors::new();
            if let Some(style) = &asset.face().style {
                descriptors.style(style);
            }
            descriptors.weight(&asset.face().weight.to_string());
            descriptors.stretch(&format!("{}%", asset.face().stretch));
            if let Some(variant) = &asset.face().variant {
                descriptors.variant(variant);
            }
            let elm = FontFace::new_with_u8_array_and_descriptors(
                &path,
                #[allow(mutable_transmutes, clippy::transmute_ptr_to_ptr)]
                unsafe {
                    std::mem::transmute(font_asset.bytes())
                },
                &descriptors,
            )
            .unwrap();
            drop(
                elm.load()
                    .unwrap_or_else(|_| panic!("Could not load font: {}", path)),
            );
            window()
                .document()
                .expect("Could not get window document")
                .fonts()
                .add(&elm)
                .unwrap_or_else(|_| panic!("Could not add font: {}", path));
            self.fontfaces_cache.insert(path.clone(), elm);
            self.fontfaces_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("fontface") {
            if let Some(path) = self.fontfaces_table.remove(id) {
                self.fontfaces_cache.remove(&path);
            }
        }
    }

    fn create_surface(&mut self, name: &str, width: usize, height: usize) -> bool {
        let document = window().document().expect("Could not get window document");
        let canvas = document
            .create_element("canvas")
            .expect("Could not create canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        canvas.set_width(width as u32);
        canvas.set_height(height as u32);
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        self.surfaces_cache
            .insert(name.to_owned(), (canvas, context));
        true
    }

    fn destroy_surface(&mut self, name: &str) -> bool {
        if let Some((canvas, _)) = self.surfaces_cache.remove(name) {
            canvas.remove();
            true
        } else {
            false
        }
    }

    fn has_surface(&mut self, name: &str) -> bool {
        self.surfaces_cache.contains_key(name)
    }

    fn get_surface_size(&self, name: &str) -> Option<(usize, usize)> {
        self.surfaces_cache
            .get(name)
            .map(|(canvas, _)| (canvas.width() as usize, canvas.height() as usize))
    }

    fn update_surface<'a, I>(&mut self, name: &str, commands: I) -> Result<(usize, usize)>
    where
        I: IntoIterator<Item = Command<'a>>,
    {
        if let Some((canvas, context)) = self.surfaces_cache.get(name) {
            context.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
            self.execute_with(context, 1.0, commands)
        } else {
            Err(Error::Message(format!("There is no surface: {}", name)))
        }
    }
}

impl CompositeRendererResources<HtmlImageElement> for WebCompositeRenderer {
    fn add_resource(&mut self, id: String, resource: HtmlImageElement) -> Result<AssetID> {
        let asset_id = AssetID::new();
        self.images_cache.insert(id.clone(), resource);
        self.images_table.insert(asset_id, id);
        Ok(asset_id)
    }

    fn remove_resource(&mut self, id: AssetID) -> Result<HtmlImageElement> {
        if let Some(id) = self.images_table.remove(&id) {
            if let Some(resource) = self.images_cache.remove(&id) {
                Ok(resource)
            } else {
                Err(Error::Message(format!(
                    "Image resource does not exists in cache: {:?}",
                    id
                )))
            }
        } else {
            Err(Error::Message(format!(
                "Image resource does not exists in table: {:?}",
                id
            )))
        }
    }
}

impl CompositeRendererResources<FontFace> for WebCompositeRenderer {
    fn add_resource(&mut self, id: String, resource: FontFace) -> Result<AssetID> {
        let asset_id = AssetID::new();
        self.fontfaces_cache.insert(id.clone(), resource);
        self.fontfaces_table.insert(asset_id, id);
        Ok(asset_id)
    }

    fn remove_resource(&mut self, id: AssetID) -> Result<FontFace> {
        if let Some(id) = self.fontfaces_table.remove(&id) {
            if let Some(resource) = self.fontfaces_cache.remove(&id) {
                Ok(resource)
            } else {
                Err(Error::Message(format!(
                    "Font face resource does not exists in cache: {:?}",
                    id
                )))
            }
        } else {
            Err(Error::Message(format!(
                "Font face esource does not exists in table: {:?}",
                id
            )))
        }
    }
}
