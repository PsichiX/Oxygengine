extern crate oxygengine_composite_renderer as renderer;
extern crate oxygengine_core as core;

use core::{
    assets::{asset::AssetID, database::AssetsDatabase},
    error::*,
};
use futures::{future, Future};
use renderer::{composite_renderer::*, math::*, png_image_asset_protocol::PngImageAsset};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::*, Clamped, JsCast};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::*;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub struct WebCompositeRenderer {
    state: RenderState,
    viewport: Rect,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    images_cache: Rc<RefCell<HashMap<String, ImageBitmap>>>,
    images_table: Rc<RefCell<HashMap<AssetID, String>>>,
}

unsafe impl Send for WebCompositeRenderer {}
unsafe impl Sync for WebCompositeRenderer {}

impl WebCompositeRenderer {
    pub fn new(canvas_id: &str) -> Self {
        let document = window().document().expect("no `window.document` exists");
        let canvas = document
            .get_element_by_id(canvas_id)
            .expect(&format!("no `{}` canvas in document", canvas_id));
        let canvas: HtmlCanvasElement = canvas
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();
        Self {
            state: RenderState::default(),
            viewport: Rect::default(),
            canvas,
            context,
            images_cache: Default::default(),
            images_table: Default::default(),
        }
    }

    pub fn with_state(canvas_id: &str, state: RenderState) -> Self {
        let mut result = Self::new(canvas_id);
        *result.state_mut() = state;
        result
    }
}

impl CompositeRenderer for WebCompositeRenderer {
    fn execute<'a, I>(&mut self, commands: I) -> Result<()>
    where
        I: IntoIterator<Item = Command<'a>>,
    {
        let mut stats = Stats::default();
        for command in commands {
            match command {
                Command::Draw(renderable) => match renderable {
                    Renderable::Rectangle(rectangle) => {
                        self.context
                            .set_fill_style(&rectangle.color.to_string().into());
                        self.context.fill_rect(
                            rectangle.rect.x.into(),
                            rectangle.rect.y.into(),
                            rectangle.rect.w.into(),
                            rectangle.rect.h.into(),
                        );
                        stats.render_ops += 2;
                        stats.renderables += 1;
                    }
                    Renderable::Text(text) => {
                        self.context.set_fill_style(&text.color.to_string().into());
                        self.context
                            .set_font(&format!("{}px {}", text.size, &text.font));
                        self.context.set_text_align(match text.align {
                            TextAlign::Left => "left",
                            TextAlign::Center => "center",
                            TextAlign::Right => "right",
                        });
                        drop(self.context.fill_text(
                            &text.text,
                            text.position.x.into(),
                            text.position.y.into(),
                        ));
                        stats.render_ops += 4;
                        stats.renderables += 1;
                    }
                    Renderable::Path(path) => {
                        let mut ops = 0;
                        self.context.begin_path();
                        for element in &path.elements {
                            match element {
                                PathElement::MoveTo(pos) => {
                                    self.context.move_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::LineTo(pos) => {
                                    self.context.line_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::BezierCurveTo(cpa, cpb, pos) => {
                                    self.context.bezier_curve_to(
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
                                    self.context.quadratic_curve_to(
                                        cp.x.into(),
                                        cp.y.into(),
                                        pos.x.into(),
                                        pos.y.into(),
                                    );
                                    ops += 1;
                                }
                                PathElement::Arc(pos, r, a) => {
                                    drop(self.context.arc(
                                        pos.x.into(),
                                        pos.y.into(),
                                        (*r).into(),
                                        a.start.into(),
                                        a.end.into(),
                                    ));
                                    ops += 1;
                                }
                                PathElement::Ellipse(pos, r, rot, a) => {
                                    drop(self.context.ellipse(
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
                                    self.context.rect(
                                        rect.x.into(),
                                        rect.y.into(),
                                        rect.w.into(),
                                        rect.h.into(),
                                    );
                                    ops += 1;
                                }
                            }
                        }
                        self.context.set_fill_style(&path.color.to_string().into());
                        self.context.fill();
                        stats.render_ops += 3 + ops;
                        stats.renderables += 1;
                    }
                    Renderable::Image(image) => {
                        let path: &str = &image.image;
                        if let Some(bitmap) = self.images_cache.borrow().get(path) {
                            let src = if let Some(src) = image.source {
                                src
                            } else {
                                Rect {
                                    x: 0.0,
                                    y: 0.0,
                                    w: bitmap.width() as Scalar,
                                    h: bitmap.height() as Scalar,
                                }
                            };
                            let dst = if let Some(dst) = image.destination {
                                dst
                            } else {
                                Rect {
                                    x: 0.0,
                                    y: 0.0,
                                    w: src.w,
                                    h: src.h,
                                }
                            };
                            drop(self
                                .context
                                .draw_image_with_image_bitmap_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                                    bitmap,
                                    src.x.into(),
                                    src.y.into(),
                                    src.w.into(),
                                    src.h.into(),
                                    dst.x.into(),
                                    dst.y.into(),
                                    dst.w.into(),
                                    dst.h.into(),
                                ));
                            stats.render_ops += 1;
                            stats.renderables += 1;
                        }
                    }
                },
                Command::Stroke(line_width, renderable) => match renderable {
                    Renderable::Rectangle(rectangle) => {
                        self.context
                            .set_stroke_style(&rectangle.color.to_string().into());
                        self.context.set_line_width(line_width.into());
                        self.context.stroke_rect(
                            rectangle.rect.x.into(),
                            rectangle.rect.y.into(),
                            rectangle.rect.w.into(),
                            rectangle.rect.h.into(),
                        );
                        stats.render_ops += 3;
                        stats.renderables += 1;
                    }
                    Renderable::Text(text) => {
                        self.context
                            .set_stroke_style(&text.color.to_string().into());
                        self.context.set_line_width(line_width.into());
                        self.context
                            .set_font(&format!("{}px {}", text.size, &text.font));
                        self.context.set_text_align(match text.align {
                            TextAlign::Left => "left",
                            TextAlign::Center => "center",
                            TextAlign::Right => "right",
                        });
                        drop(self.context.stroke_text(
                            &text.text,
                            text.position.x.into(),
                            text.position.y.into(),
                        ));
                        stats.render_ops += 5;
                        stats.renderables += 1;
                    }
                    Renderable::Path(path) => {
                        let mut ops = 0;
                        self.context.begin_path();
                        for element in &path.elements {
                            match element {
                                PathElement::MoveTo(pos) => {
                                    self.context.move_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::LineTo(pos) => {
                                    self.context.line_to(pos.x.into(), pos.y.into());
                                    ops += 1;
                                }
                                PathElement::BezierCurveTo(cpa, cpb, pos) => {
                                    self.context.bezier_curve_to(
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
                                    self.context.quadratic_curve_to(
                                        cp.x.into(),
                                        cp.y.into(),
                                        pos.x.into(),
                                        pos.y.into(),
                                    );
                                    ops += 1;
                                }
                                PathElement::Arc(pos, r, a) => {
                                    drop(self.context.arc(
                                        pos.x.into(),
                                        pos.y.into(),
                                        (*r).into(),
                                        a.start.into(),
                                        a.end.into(),
                                    ));
                                    ops += 1;
                                }
                                PathElement::Ellipse(pos, r, rot, a) => {
                                    drop(self.context.ellipse(
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
                                    self.context.rect(
                                        rect.x.into(),
                                        rect.y.into(),
                                        rect.w.into(),
                                        rect.h.into(),
                                    );
                                    ops += 1;
                                }
                            }
                        }
                        self.context
                            .set_stroke_style(&path.color.to_string().into());
                        self.context.set_line_width(line_width.into());
                        self.context.stroke();
                        stats.render_ops += 4 + ops;
                        stats.renderables += 1;
                    }
                    Renderable::Image(image) => {
                        panic!(
                            "[Oxygengine] Trying to render image as stroke: {}",
                            image.image
                        );
                    }
                },
                Command::Transform(transform) => match transform {
                    Transformation::Translate(pos) => {
                        drop(self.context.translate(pos.x.into(), pos.y.into()));
                        stats.render_ops += 1;
                    }
                    Transformation::Rotate(rot) => {
                        drop(self.context.rotate(rot.into()));
                        stats.render_ops += 1;
                    }
                    Transformation::Scale(scl) => {
                        drop(self.context.scale(scl.x.into(), scl.y.into()));
                        stats.render_ops += 1;
                    }
                    Transformation::Transform(a, b, c, d, e, f) => {
                        drop(self.context.transform(
                            a.into(),
                            b.into(),
                            c.into(),
                            d.into(),
                            e.into(),
                            f.into(),
                        ));
                        stats.render_ops += 1;
                    }
                },
                Command::Store => {
                    self.context.save();
                    stats.render_ops += 1;
                }
                Command::Restore => {
                    self.context.restore();
                    stats.render_ops += 1;
                }
                _ => {}
            }
        }
        self.state.set_stats(stats);
        Ok(())
    }

    fn state(&self) -> &RenderState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut RenderState {
        &mut self.state
    }

    fn viewport(&self) -> Rect {
        self.viewport
    }

    fn update_state(&mut self) {
        let w = self.canvas.client_width();
        let h = self.canvas.client_height();
        if (self.viewport.w - w as f32).abs() > 1.0 || (self.viewport.h - h as f32).abs() > 1.0 {
            self.canvas.set_width(w as u32);
            self.canvas.set_height(h as u32);
            self.viewport = Rect {
                x: 0.0,
                y: 0.0,
                w: w as Scalar,
                h: h as Scalar,
            };
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
            let pixels = asset.pixels();
            let width = asset.width() as u32;
            let height = asset.height() as u32;
            #[allow(mutable_transmutes)]
            let pixels = unsafe { std::mem::transmute::<&[u8], &mut [u8]>(pixels) };
            let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(pixels), width, height)
                .unwrap();
            let promise = window().create_image_bitmap_with_image_data(&data).unwrap();
            let ic = self.images_cache.clone();
            let it = self.images_table.clone();
            let future = JsFuture::from(promise).and_then(move |data| {
                assert!(data.is_instance_of::<ImageBitmap>());
                let data: ImageBitmap = data.dyn_into().unwrap();
                ic.borrow_mut().insert(path.clone(), data);
                it.borrow_mut().insert(id, path);
                future::ok(JsValue::null())
            });
            future_to_promise(future);
        }
        for id in assets.lately_unloaded_protocol("png") {
            if let Some(path) = self.images_table.borrow_mut().remove(id) {
                self.images_cache.borrow_mut().remove(&path);
            }
        }
    }
}
