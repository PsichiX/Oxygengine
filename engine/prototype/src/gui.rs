use crate::resources::{camera::*, renderables::*};
use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;

pub trait GuiWidget<T>: Sized {
    fn execute(self, gui: Gui<T>) -> Rect;
}

pub struct Gui<'a, T> {
    layout: Rect,
    pointer: Vec2,
    input_consumed: &'a mut bool,
    camera: &'a Camera,
    renderables: &'a mut Renderables,
    pub context: &'a mut T,
}

macro_rules! impl_make {
    ( $owner:expr, $layout:expr ) => {
        Gui {
            layout: $layout,
            pointer: $owner.pointer,
            input_consumed: $owner.input_consumed,
            camera: $owner.camera,
            renderables: $owner.renderables,
            context: $owner.context,
        }
    };
}

impl<'a, T> Gui<'a, T> {
    pub fn layout(&self) -> Rect {
        self.layout
    }

    pub fn pointer(&self) -> Vec2 {
        self.pointer
    }

    pub fn input_consumed(&self) -> bool {
        *self.input_consumed
    }

    pub fn hovers(&self) -> bool {
        !*self.input_consumed && self.layout.contains_point(self.pointer)
    }

    pub fn widget<W>(&mut self, widget: W) -> Rect
    where
        W: GuiWidget<T>,
    {
        widget.execute(self.gui())
    }

    pub fn scope<F>(&mut self, mut f: F)
    where
        F: FnMut(Gui<T>),
    {
        f(self.gui());
    }

    pub fn clip<F>(&mut self, mut f: F)
    where
        F: FnMut(Gui<T>),
    {
        let dim = self.camera.world_size();
        let half_dim = dim * 0.5;
        let pos = self
            .camera
            .camera_to_screen_point(vec2(self.layout.x + half_dim.x, self.layout.y + half_dim.y));
        let size = self
            .camera
            .camera_to_screen_point(vec2(self.layout.w, self.layout.h));
        let dim = self.camera.viewport_size();
        let rect = Rect {
            x: pos.x / dim.x,
            y: pos.y / dim.y,
            w: size.x / dim.x,
            h: size.y / dim.y,
        };
        self.renderables.draw(Renderable::PushScissor(rect));
        f(self.gui());
        self.renderables.draw(Renderable::PopScissor);
    }

    pub fn freeform_aligned<F>(
        &mut self,
        size: impl Into<Vec2>,
        alignment: impl Into<Vec2>,
        pivot: impl Into<Vec2>,
        mut f: F,
    ) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        let size = size.into();
        let alignment = alignment.into();
        let pivot = pivot.into();
        let layout = Rect {
            x: self.layout.x + self.layout.w * alignment.x - size.x * pivot.x,
            y: self.layout.y + self.layout.h * alignment.y - size.y * pivot.y,
            w: size.x,
            h: size.y,
        };
        f(impl_make!(self, layout));
        layout
    }

    pub fn freeform_at<F>(
        &mut self,
        size: impl Into<Vec2>,
        position: impl Into<Vec2>,
        pivot: impl Into<Vec2>,
        relative: bool,
        mut f: F,
    ) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        let size = size.into();
        let position = position.into();
        let pivot = pivot.into();
        let mut layout = Rect {
            x: position.x - size.x * pivot.x,
            y: position.y - size.y * pivot.y,
            w: size.x,
            h: size.y,
        };
        if relative {
            layout.x += self.layout.x;
            layout.y += self.layout.y;
        }
        f(impl_make!(self, layout));
        layout
    }

    pub fn growable<F>(&mut self, alignment: impl Into<Vec2>, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>) -> Rect,
    {
        let alignment = alignment.into();
        self.renderables.begin();
        let layout = f(self.gui());
        let offset = vec2(self.layout.w - layout.w, self.layout.h - layout.h) * alignment
            - layout.position();
        for mut renderable in self.renderables.consume().unwrap() {
            match &mut renderable {
                Renderable::PushTransform(transform) => transform.position += offset,
                Renderable::PushScissor(rect) => {
                    let dim = self.camera.world_size();
                    let offset = self.camera.camera_to_screen_point(offset);
                    rect.x += offset.x / dim.x;
                    rect.y += offset.y / dim.y;
                }
                Renderable::Advanced(renderable) => renderable.transform.position += offset,
                Renderable::Sprite(renderable) => renderable.transform.position += offset,
                Renderable::Text(renderable) => renderable.transform.position += offset,
                _ => {}
            }
            self.renderables.draw(renderable);
        }
        layout
    }

    pub fn horizontal_overflow<F>(&mut self, alignment: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>) -> Rect,
    {
        let layout = Rect {
            x: self.layout.x,
            y: self.layout.y + self.layout.h * alignment,
            w: self.layout.w,
            h: 0.0,
        };
        f(impl_make!(self, layout))
    }

    pub fn vertical_overflow<F>(&mut self, alignment: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>) -> Rect,
    {
        let layout = Rect {
            x: self.layout.x + self.layout.w * alignment,
            y: self.layout.y,
            w: 0.0,
            h: self.layout.h,
        };
        f(impl_make!(self, layout))
    }

    pub fn scale<F>(&mut self, value: Scalar, alignment: impl Into<Vec2>, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        let alignment = alignment.into();
        let width = self.layout.w * value;
        let height = self.layout.h * value;
        let horizontal_space = self.layout.w - width;
        let vertical_space = self.layout.h - height;
        let layout = Rect {
            x: self.layout.x + horizontal_space * alignment.x,
            y: self.layout.y + vertical_space * alignment.y,
            w: width,
            h: height,
        };
        f(impl_make!(self, layout));
        layout
    }

    pub fn margin<F>(
        &mut self,
        mut left: Scalar,
        right: Scalar,
        mut top: Scalar,
        bottom: Scalar,
        mut f: F,
    ) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        let mut horizontal = left + right;
        if horizontal > self.layout.w {
            left = self.layout.w * 0.5;
            horizontal = self.layout.w;
        }
        let mut vertical = top + bottom;
        if vertical > self.layout.h {
            top = self.layout.h * 0.5;
            vertical = self.layout.h;
        }
        let layout = Rect {
            x: self.layout.x + left,
            y: self.layout.y + top,
            w: self.layout.w - horizontal,
            h: self.layout.h - vertical,
        };
        f(impl_make!(self, layout));
        layout
    }

    pub fn absolute<F>(&mut self, layout: Rect, mut f: F)
    where
        F: FnMut(Gui<T>),
    {
        f(impl_make!(self, layout));
    }

    pub fn section<F>(&mut self, relative: Rect, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        let layout = Rect {
            x: self.layout.x + relative.x,
            y: self.layout.y + relative.y,
            w: relative.w,
            h: relative.h,
        };
        f(impl_make!(self, layout));
        layout
    }

    pub fn section_left<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.clamp(0.0, self.layout.w);
        let [low, _] = self.layout.split_at_x(self.layout.x + value);
        f(impl_make!(self, low));
        low
    }

    pub fn section_right<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.clamp(0.0, self.layout.w);
        let [_, high] = self
            .layout
            .split_at_x(self.layout.x + self.layout.w - value);
        f(impl_make!(self, high));
        high
    }

    pub fn section_top<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.clamp(0.0, self.layout.h);
        let [low, _] = self.layout.split_at_y(self.layout.y + value);
        f(impl_make!(self, low));
        low
    }

    pub fn section_bottom<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.clamp(0.0, self.layout.h);
        let [_, high] = self
            .layout
            .split_at_y(self.layout.y + self.layout.h - value);
        f(impl_make!(self, high));
        high
    }

    pub fn cut_left<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.clamp(0.0, self.layout.w);
        let [low, high] = self.layout.split_at_x(self.layout.x + value);
        f(impl_make!(self, low));
        self.layout = high;
        low
    }

    pub fn cut_right<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.clamp(0.0, self.layout.w);
        let [low, high] = self
            .layout
            .split_at_x(self.layout.x + self.layout.w - value);
        f(impl_make!(self, high));
        self.layout = low;
        high
    }

    pub fn cut_top<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.clamp(0.0, self.layout.h);
        let [low, high] = self.layout.split_at_y(self.layout.y + value);
        f(impl_make!(self, low));
        self.layout = high;
        low
    }

    pub fn cut_bottom<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.clamp(0.0, self.layout.h);
        let [low, high] = self
            .layout
            .split_at_y(self.layout.y + self.layout.h - value);
        f(impl_make!(self, high));
        self.layout = low;
        high
    }

    pub fn grow_left<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.max(0.0);
        let layout = Rect {
            x: self.layout.x - value,
            y: self.layout.y,
            w: value,
            h: self.layout.h,
        };
        f(impl_make!(self, layout));
        self.layout.x -= value;
        self.layout.w += value;
        layout
    }

    pub fn grow_right<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.max(0.0);
        let layout = Rect {
            x: self.layout.x + self.layout.w,
            y: self.layout.y,
            w: value,
            h: self.layout.h,
        };
        f(impl_make!(self, layout));
        self.layout.w += value;
        layout
    }

    pub fn grow_top<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.max(0.0);
        let layout = Rect {
            x: self.layout.x,
            y: self.layout.y - value,
            w: self.layout.w,
            h: value,
        };
        f(impl_make!(self, layout));
        self.layout.y -= value;
        self.layout.h += value;
        layout
    }

    pub fn grow_bottom<F>(&mut self, mut value: Scalar, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        value = value.max(0.0);
        let layout = Rect {
            x: self.layout.x,
            y: self.layout.y + self.layout.h,
            w: self.layout.w,
            h: value,
        };
        f(impl_make!(self, layout));
        self.layout.h += value;
        layout
    }

    pub fn offset<F>(&mut self, value: impl Into<Vec2>, mut f: F) -> Rect
    where
        F: FnMut(Gui<T>),
    {
        let value = value.into();
        let layout = Rect {
            x: self.layout.x + value.x,
            y: self.layout.y + value.y,
            w: self.layout.w,
            h: self.layout.h,
        };
        f(impl_make!(self, layout));
        layout
    }

    pub fn horizontal_number<F>(&mut self, mut count: usize, mut f: F)
    where
        F: FnMut(Gui<T>, usize),
    {
        count = count.max(1);
        let size = self.layout.w / count as Scalar;
        for index in 0..count {
            let layout = Rect {
                x: self.layout.x + size * index as Scalar,
                y: self.layout.y,
                w: size,
                h: self.layout.h,
            };
            f(impl_make!(self, layout), index);
        }
    }

    pub fn horizontal_separation<F>(&mut self, mut value: Scalar, f: F)
    where
        F: FnMut(Gui<T>, usize),
    {
        value = value.clamp(0.0, self.layout.w);
        let count = (self.layout.w / value) as usize;
        self.horizontal_number(count, f);
    }

    pub fn vertical_number<F>(&mut self, mut count: usize, mut f: F)
    where
        F: FnMut(Gui<T>, usize),
    {
        count = count.max(1);
        let size = self.layout.h / count as Scalar;
        for index in 0..count {
            let layout = Rect {
                x: self.layout.x,
                y: self.layout.y + size * index as Scalar,
                w: self.layout.w,
                h: size,
            };
            f(impl_make!(self, layout), index);
        }
    }

    pub fn vertical_separation<F>(&mut self, mut value: Scalar, f: F)
    where
        F: FnMut(Gui<T>, usize),
    {
        value = value.clamp(0.0, self.layout.h);
        let count = (self.layout.h / value) as usize;
        self.vertical_number(count, f);
    }

    pub fn grid_number<F>(&mut self, mut cols: usize, mut rows: usize, mut f: F)
    where
        F: FnMut(Gui<T>, usize, usize),
    {
        cols = cols.max(1);
        rows = rows.max(1);
        let width = self.layout.w / cols as Scalar;
        let height = self.layout.h / rows as Scalar;
        for row in 0..rows {
            for col in 0..cols {
                let layout = Rect {
                    x: self.layout.x + width * col as Scalar,
                    y: self.layout.y + height * row as Scalar,
                    w: width,
                    h: height,
                };
                f(impl_make!(self, layout), col, row);
            }
        }
    }

    pub fn grid_separation<F>(&mut self, mut cell_width: Scalar, mut cell_height: Scalar, f: F)
    where
        F: FnMut(Gui<T>, usize, usize),
    {
        cell_width = cell_width.clamp(0.0, self.layout.w);
        cell_height = cell_height.clamp(0.0, self.layout.h);
        let cols = (self.layout.w / cell_width) as usize;
        let rows = (self.layout.h / cell_height) as usize;
        self.grid_number(cols, rows, f);
    }

    pub fn sprite(&mut self, name: impl ToString, tint: Rgba) {
        self.renderables.draw(
            SpriteRenderable::new(name)
                .position(self.layout.center())
                .size([self.layout.w, self.layout.h])
                .tint(tint),
        );
    }

    pub fn sprite_region(&mut self, name: impl ToString, tint: Rgba, region: Rect) {
        self.renderables.draw(
            SpriteRenderable::new(name)
                .position(self.layout.center())
                .size([self.layout.w, self.layout.h])
                .tint(tint)
                .region(region),
        );
    }

    pub fn sprite_sliced(
        &mut self,
        name: impl ToString,
        tint: Rgba,
        image_region: Rect,
        (mut left, mut right, mut top, mut bottom): (Scalar, Scalar, Scalar, Scalar),
        frame: bool,
    ) {
        // TODO: handle zero hor and ver.
        let horizontal = left + right;
        if horizontal > self.layout.w {
            left /= horizontal;
            right /= horizontal;
        }
        let vertical = top + bottom;
        if vertical > self.layout.h {
            top /= vertical;
            bottom /= vertical;
        }
        let name = name.to_string();
        self.cut_left(left, |mut gui| {
            gui.cut_top(top, |mut gui| {
                gui.sprite_region(
                    &name,
                    tint,
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        w: image_region.x,
                        h: image_region.y,
                    },
                )
            });
            gui.cut_bottom(bottom, |mut gui| {
                gui.sprite_region(
                    &name,
                    tint,
                    Rect {
                        x: 0.0,
                        y: image_region.y + image_region.h,
                        w: image_region.x,
                        h: 1.0 - (image_region.y + image_region.h),
                    },
                )
            });
            gui.sprite_region(
                &name,
                tint,
                Rect {
                    x: 0.0,
                    y: image_region.y,
                    w: image_region.x,
                    h: image_region.h,
                },
            );
        });
        self.cut_right(right, |mut gui| {
            gui.cut_top(top, |mut gui| {
                gui.sprite_region(
                    &name,
                    tint,
                    Rect {
                        x: image_region.x + image_region.w,
                        y: 0.0,
                        w: 1.0 - (image_region.x + image_region.w),
                        h: image_region.y,
                    },
                )
            });
            gui.cut_bottom(bottom, |mut gui| {
                gui.sprite_region(
                    &name,
                    tint,
                    Rect {
                        x: image_region.x + image_region.w,
                        y: image_region.y + image_region.h,
                        w: 1.0 - (image_region.x + image_region.w),
                        h: 1.0 - (image_region.y + image_region.h),
                    },
                )
            });
            gui.sprite_region(
                &name,
                tint,
                Rect {
                    x: image_region.x + image_region.w,
                    y: image_region.y,
                    w: 1.0 - (image_region.x + image_region.w),
                    h: image_region.h,
                },
            );
        });
        self.cut_top(top, |mut gui| {
            gui.sprite_region(
                &name,
                tint,
                Rect {
                    x: image_region.x,
                    y: 0.0,
                    w: image_region.w,
                    h: image_region.y,
                },
            )
        });
        self.cut_bottom(bottom, |mut gui| {
            gui.sprite_region(
                &name,
                tint,
                Rect {
                    x: image_region.x,
                    y: image_region.y + image_region.h,
                    w: image_region.w,
                    h: 1.0 - (image_region.y + image_region.h),
                },
            )
        });
        if !frame {
            self.sprite_region(
                &name,
                tint,
                Rect {
                    x: image_region.x,
                    y: image_region.y,
                    w: image_region.w,
                    h: image_region.h,
                },
            );
        }
    }

    pub fn text(
        &mut self,
        font: impl ToString,
        content: impl Into<HaTextContent>,
        size: Scalar,
        color: Rgba,
        alignment: impl Into<Vec2>,
    ) {
        let alignment = alignment.into();
        self.renderables.draw(
            TextRenderable::new(font, content)
                .position(Vec2::lerp(
                    vec2(self.layout.x, self.layout.y),
                    vec2(self.layout.x + self.layout.w, self.layout.y + self.layout.h),
                    alignment,
                ))
                .size(size)
                .color(color)
                .alignment(alignment)
                .bounds_width(Some(self.layout.w))
                .bounds_height(Some(self.layout.h)),
        );
    }

    pub fn button<F>(&mut self, mut f: F) -> bool
    where
        F: FnMut(Gui<T>, bool) -> bool,
    {
        let hovers = self.hovers();
        let gui = self.gui();
        if f(gui, hovers) && hovers {
            *self.input_consumed = true;
            true
        } else {
            false
        }
    }

    pub fn gui(&mut self) -> Gui<T> {
        Gui {
            layout: self.layout,
            pointer: self.pointer,
            input_consumed: self.input_consumed,
            camera: self.camera,
            renderables: self.renderables,
            context: self.context,
        }
    }
}

pub fn gui<T, F>(
    screen_pointer: impl Into<Vec2>,
    camera: &Camera,
    renderables: &mut Renderables,
    context: &mut T,
    mut f: F,
) -> bool
where
    F: FnMut(Gui<T>),
{
    let point = camera.screen_to_camera_point(screen_pointer.into());
    let dim = camera.world_size();
    if !point.x.is_finite()
        || !point.y.is_finite()
        || !dim.x.is_finite()
        || !dim.y.is_finite()
        || dim.x.abs() < 1.0e-6
        || dim.y.abs() < 1.0e-6
    {
        return false;
    }
    let half_dim = dim * 0.5;
    let layout = Rect {
        x: -half_dim.x,
        y: -half_dim.y,
        w: dim.x,
        h: dim.y,
    };
    let mut input_consumed = false;
    renderables.draw(Renderable::PushTransform(camera.transform()));
    let gui = Gui::<T> {
        layout,
        pointer: point - half_dim,
        input_consumed: &mut input_consumed,
        camera,
        renderables,
        context,
    };
    f(gui);
    renderables.draw(Renderable::PopTransform);
    input_consumed
}
