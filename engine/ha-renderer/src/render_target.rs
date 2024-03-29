use crate::{
    ha_renderer::RenderStageResources, image::ImageMipmap, pipeline::stage::ClearSettings,
    HasContextResources,
};
use core::{id::ID, Scalar};
use glow::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum RenderTargetError {
    InvalidId(String),
    DuplicateId(String),
    DepthStencilAlreadyPresent(String),
    NoResources,
    ColorTargetBufferDoesNotExists(usize),
    Internal(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetValueType {
    Color,
    FloatColor,
}

impl Default for TargetValueType {
    fn default() -> Self {
        Self::Color
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetBuffer {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub value_type: TargetValueType,
    #[serde(default)]
    pub mipmap: ImageMipmap,
}

impl TargetBuffer {
    pub fn color(id: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            value_type: TargetValueType::Color,
            mipmap: Default::default(),
        }
    }

    pub fn float_color(id: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            value_type: TargetValueType::FloatColor,
            mipmap: Default::default(),
        }
    }

    pub fn mipmap(mut self, mipmap: ImageMipmap) -> Self {
        self.mipmap = mipmap;
        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetBuffers {
    #[serde(default)]
    pub depth_stencil: Option<String>,
    #[serde(default)]
    pub colors: Vec<TargetBuffer>,
}

impl TargetBuffers {
    pub fn with_depth_stencil(mut self, id: String) -> Result<Self, RenderTargetError> {
        if let Some(ds) = &self.depth_stencil {
            if ds == &id {
                return Err(RenderTargetError::DepthStencilAlreadyPresent(id));
            }
        }
        if self.colors.iter().any(|c| c.id == id) {
            return Err(RenderTargetError::DuplicateId(id));
        }
        self.depth_stencil = Some(id);
        Ok(self)
    }

    pub fn with_color(mut self, buffer: TargetBuffer) -> Result<Self, RenderTargetError> {
        if let Some(ds) = &self.depth_stencil {
            if ds == &buffer.id {
                return Err(RenderTargetError::DepthStencilAlreadyPresent(buffer.id));
            }
        }
        if self.colors.iter().any(|c| c.id == buffer.id) {
            return Err(RenderTargetError::DuplicateId(buffer.id));
        }
        self.colors.push(buffer);
        Ok(self)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RenderTargetSizeValue {
    Screen {
        #[serde(default)]
        level: i8,
    },
    ScreenAspectWidth {
        value: usize,
        #[serde(default)]
        round_up: bool,
    },
    ScreenAspectHeight {
        value: usize,
        #[serde(default)]
        round_up: bool,
    },
    Exact {
        value: usize,
        #[serde(default)]
        level: i8,
    },
}

impl Default for RenderTargetSizeValue {
    fn default() -> Self {
        Self::Screen { level: 0 }
    }
}

impl RenderTargetSizeValue {
    pub fn width(self, width: usize, height: usize) -> usize {
        match self {
            Self::Screen { level } => width << level,
            Self::ScreenAspectWidth { value, .. } => value,
            Self::ScreenAspectHeight { value, round_up } => {
                let value = (value as Scalar * width as Scalar) / height as Scalar;
                if round_up {
                    value.ceil() as _
                } else {
                    value.floor() as _
                }
            }
            Self::Exact { value, level } => value << level,
        }
    }

    pub fn height(self, width: usize, height: usize) -> usize {
        match self {
            Self::Screen { level } => height << level,
            Self::ScreenAspectWidth { value, round_up } => {
                let value = (value as Scalar * height as Scalar) / width as Scalar;
                if round_up {
                    value.ceil() as _
                } else {
                    value.floor() as _
                }
            }
            Self::ScreenAspectHeight { value, .. } => value,
            Self::Exact { value, level } => value << level,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub enum RenderTargetClipAreaValue {
    #[default]
    Full,
    Exact(usize),
    Margin(usize),
    Anchor(Scalar),
    AspectRatio {
        width: usize,
        height: usize,
    },
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct RenderTargetClipArea {
    pub left: RenderTargetClipAreaValue,
    pub right: RenderTargetClipAreaValue,
    pub top: RenderTargetClipAreaValue,
    pub bottom: RenderTargetClipAreaValue,
}

impl RenderTargetClipArea {
    /// (x, y, width, height)
    pub fn rect(&self, width: usize, height: usize) -> (usize, usize, usize, usize) {
        let left = match self.left {
            RenderTargetClipAreaValue::Full => 0,
            RenderTargetClipAreaValue::Exact(v) => v,
            RenderTargetClipAreaValue::Margin(v) => v,
            RenderTargetClipAreaValue::Anchor(v) => (v.clamp(0.0, 1.0) * width as Scalar) as usize,
            RenderTargetClipAreaValue::AspectRatio {
                width: w,
                height: h,
            } => width.saturating_sub(w * height / h) / 2,
        }
        .clamp(0, width);
        let right = match self.right {
            RenderTargetClipAreaValue::Full => width,
            RenderTargetClipAreaValue::Exact(v) => v,
            RenderTargetClipAreaValue::Margin(v) => width.saturating_sub(v),
            RenderTargetClipAreaValue::Anchor(v) => (v.clamp(0.0, 1.0) * width as Scalar) as usize,
            RenderTargetClipAreaValue::AspectRatio {
                width: w,
                height: h,
            } => (width + w * height / h) / 2,
        }
        .clamp(0, width);
        let top = match self.top {
            RenderTargetClipAreaValue::Full => 0,
            RenderTargetClipAreaValue::Exact(v) => v,
            RenderTargetClipAreaValue::Margin(v) => v,
            RenderTargetClipAreaValue::Anchor(v) => (v.clamp(0.0, 1.0) * height as Scalar) as usize,
            RenderTargetClipAreaValue::AspectRatio {
                width: w,
                height: h,
            } => height.saturating_sub(h * width / w) / 2,
        }
        .clamp(0, height);
        let bottom = match self.bottom {
            RenderTargetClipAreaValue::Full => height,
            RenderTargetClipAreaValue::Exact(v) => v,
            RenderTargetClipAreaValue::Margin(v) => height.saturating_sub(v),
            RenderTargetClipAreaValue::Anchor(v) => (v.clamp(0.0, 1.0) * height as Scalar) as usize,
            RenderTargetClipAreaValue::AspectRatio {
                width: w,
                height: h,
            } => (height + h * width / w) / 2,
        }
        .clamp(0, height);
        let x = left.min(right);
        let y = top.min(bottom);
        let w = left.max(right) - x;
        let h = top.max(bottom) - y;
        (x, y, w, h)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderTargetDescriptor {
    Main,
    Custom {
        #[serde(default)]
        buffers: TargetBuffers,
        #[serde(default)]
        width: RenderTargetSizeValue,
        #[serde(default)]
        height: RenderTargetSizeValue,
    },
}

impl Default for RenderTargetDescriptor {
    fn default() -> Self {
        Self::Main
    }
}

impl RenderTargetDescriptor {
    pub fn simple(color_name: impl ToString) -> Result<Self, RenderTargetError> {
        Ok(Self::Custom {
            buffers: TargetBuffers::default().with_color(TargetBuffer {
                id: color_name.to_string(),
                value_type: TargetValueType::Color,
                mipmap: Default::default(),
            })?,
            width: RenderTargetSizeValue::default(),
            height: RenderTargetSizeValue::default(),
        })
    }

    pub fn simple_with_depth_stencil(
        color_name: impl ToString,
        depth_stencil_name: impl ToString,
    ) -> Result<Self, RenderTargetError> {
        Ok(Self::Custom {
            buffers: TargetBuffers::default()
                .with_depth_stencil(depth_stencil_name.to_string())?
                .with_color(TargetBuffer {
                    id: color_name.to_string(),
                    value_type: TargetValueType::Color,
                    mipmap: Default::default(),
                })?,
            width: RenderTargetSizeValue::default(),
            height: RenderTargetSizeValue::default(),
        })
    }
}

#[derive(Debug)]
pub struct RenderTargetResources {
    pub buffer_handle: <Context as HasContext>::Framebuffer,
    pub depth_stencil_handle: Option<<Context as HasContext>::Texture>,
    pub color_handles: Vec<<Context as HasContext>::Texture>,
}

pub type RenderTargetId = ID<RenderTarget>;

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderTargetDetailedInfo {
    pub buffers: TargetBuffers,
    pub cached_width: usize,
    pub cached_height: usize,
    pub preferred_width: RenderTargetSizeValue,
    pub preferred_height: RenderTargetSizeValue,
    pub backbuffer: bool,
}

#[derive(Debug)]
pub struct RenderTarget {
    buffers: TargetBuffers,
    cached_width: usize,
    cached_height: usize,
    preferred_width: RenderTargetSizeValue,
    preferred_height: RenderTargetSizeValue,
    backbuffer: bool,
    resources: Option<RenderTargetResources>,
}

impl Drop for RenderTarget {
    fn drop(&mut self) {
        if self.resources.is_some() {
            panic!(
                "Dropping {} without calling `context_release` to release resources first!",
                std::any::type_name::<Self>()
            );
        }
    }
}

impl HasContextResources<Context> for RenderTarget {
    type Error = RenderTargetError;

    fn has_resources(&self) -> bool {
        self.backbuffer || self.resources.is_some()
    }

    fn context_initialize(&mut self, context: &Context) -> Result<(), Self::Error> {
        if self.backbuffer {
            return Ok(());
        }

        self.context_release(context)?;

        let buffer_handle = match unsafe { context.create_framebuffer() } {
            Ok(handle) => handle,
            Err(error) => return Err(RenderTargetError::Internal(error)),
        };
        let depth_stencil_handle = if self.buffers.depth_stencil.is_some() {
            match unsafe { context.create_texture() } {
                Ok(handle) => Some(handle),
                Err(error) => return Err(RenderTargetError::Internal(error)),
            }
        } else {
            None
        };
        let color_handles = self
            .buffers
            .colors
            .iter()
            .map(|_| match unsafe { context.create_texture() } {
                Ok(handle) => Ok(handle),
                Err(error) => Err(RenderTargetError::Internal(error)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        unsafe {
            if let Some(handle) = &depth_stencil_handle {
                context.bind_texture(TEXTURE_2D, Some(*handle));
                context.tex_image_2d(
                    TEXTURE_2D,
                    0,
                    DEPTH24_STENCIL8 as _,
                    self.cached_width as _,
                    self.cached_height as _,
                    0,
                    DEPTH_STENCIL,
                    UNSIGNED_INT_24_8,
                    None,
                );
                context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, NEAREST as _);
                context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as _);
                context.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _);
                context.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _);
                context.bind_texture(TEXTURE_2D, None);
            }

            for (handle, buffer) in color_handles.iter().zip(self.buffers.colors.iter()) {
                context.bind_texture(TEXTURE_2D, Some(*handle));
                match buffer.value_type {
                    TargetValueType::Color => {
                        context.tex_image_2d(
                            TEXTURE_2D,
                            0,
                            RGBA as _,
                            self.cached_width as _,
                            self.cached_height as _,
                            0,
                            RGBA,
                            UNSIGNED_BYTE,
                            None,
                        );
                        if let ImageMipmap::Generate(Some(limit)) = buffer.mipmap {
                            context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAX_LEVEL, limit as i32);
                        }
                    }
                    TargetValueType::FloatColor => {
                        context.tex_image_2d(
                            TEXTURE_2D,
                            0,
                            RGBA32F as _,
                            self.cached_width as _,
                            self.cached_height as _,
                            0,
                            RGBA,
                            FLOAT,
                            None,
                        );
                    }
                }
                context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, LINEAR as _);
                context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, NEAREST as _);
                context.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _);
                context.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _);
            }
            context.bind_texture(TEXTURE_2D, None);

            context.bind_framebuffer(FRAMEBUFFER, Some(buffer_handle));
            context.framebuffer_texture_2d(
                FRAMEBUFFER,
                DEPTH_ATTACHMENT,
                TEXTURE_2D,
                depth_stencil_handle,
                0,
            );
            for (i, handle) in color_handles.iter().enumerate() {
                context.framebuffer_texture_2d(
                    FRAMEBUFFER,
                    COLOR_ATTACHMENT0 + i as u32,
                    TEXTURE_2D,
                    Some(*handle),
                    0,
                );
            }
        }

        self.resources = Some(RenderTargetResources {
            buffer_handle,
            depth_stencil_handle,
            color_handles,
        });
        Ok(())
    }

    fn context_release(&mut self, context: &Context) -> Result<(), Self::Error> {
        if self.backbuffer {
            return Ok(());
        }

        if let Some(resources) = std::mem::take(&mut self.resources) {
            unsafe {
                context.delete_framebuffer(resources.buffer_handle);
                if let Some(handle) = resources.depth_stencil_handle {
                    context.delete_texture(handle);
                }
                for handle in resources.color_handles {
                    context.delete_texture(handle);
                }
            }
        }
        Ok(())
    }
}

impl RenderTarget {
    pub fn new(
        buffers: TargetBuffers,
        preferred_width: RenderTargetSizeValue,
        preferred_height: RenderTargetSizeValue,
    ) -> Self {
        Self {
            buffers,
            cached_width: 0,
            cached_height: 0,
            preferred_width,
            preferred_height,
            backbuffer: false,
            resources: None,
        }
    }

    pub fn main() -> Result<Self, RenderTargetError> {
        Ok(Self {
            buffers: TargetBuffers::default()
                .with_depth_stencil("finalDepthStencil".to_owned())?
                .with_color(TargetBuffer {
                    id: "finalColor".to_owned(),
                    value_type: TargetValueType::Color,
                    mipmap: Default::default(),
                })?,
            cached_width: 0,
            cached_height: 0,
            preferred_width: RenderTargetSizeValue::default(),
            preferred_height: RenderTargetSizeValue::default(),
            backbuffer: true,
            resources: None,
        })
    }

    pub fn detailed_info(&self) -> RenderTargetDetailedInfo {
        RenderTargetDetailedInfo {
            buffers: self.buffers.clone(),
            cached_width: self.cached_width,
            cached_height: self.cached_height,
            preferred_width: self.preferred_width,
            preferred_height: self.preferred_height,
            backbuffer: self.backbuffer,
        }
    }

    pub fn buffers(&self) -> &TargetBuffers {
        &self.buffers
    }

    pub fn width(&self) -> usize {
        self.cached_width
    }

    pub fn height(&self) -> usize {
        self.cached_height
    }

    pub fn size(&self) -> (usize, usize) {
        (self.cached_width, self.cached_height)
    }

    pub fn resources(&self, _: &RenderStageResources<'_>) -> Option<&RenderTargetResources> {
        self.resources.as_ref()
    }

    pub fn query_color_data(
        &self,
        index: usize,
        context: &Context,
    ) -> Result<Vec<u8>, RenderTargetError> {
        let (channel_size, gl_type) = if self.backbuffer {
            (std::mem::size_of::<u8>(), UNSIGNED_BYTE)
        } else {
            match self.buffers.colors.get(index) {
                Some(target_buffer) => match target_buffer.value_type {
                    TargetValueType::Color => (std::mem::size_of::<u8>(), UNSIGNED_BYTE),
                    TargetValueType::FloatColor => (std::mem::size_of::<f32>(), FLOAT),
                },
                None => return Err(RenderTargetError::ColorTargetBufferDoesNotExists(index)),
            }
        };
        let width = self.cached_width;
        let height = self.cached_height;
        let size = width * height * channel_size * 4;
        let mut result = vec![0u8; size];
        unsafe {
            if !self.backbuffer {
                context.read_buffer(COLOR_ATTACHMENT0 + index as u32);
            }
            context.read_pixels(
                0,
                0,
                width as _,
                height as _,
                RGBA,
                gl_type,
                PixelPackData::Slice(&mut result),
            );
            if !self.backbuffer {
                context.read_buffer(BACK);
            }
        }
        Ok(result)
    }

    pub(crate) fn fragment_buffers(&self) -> impl Iterator<Item = &'_ str> + '_ {
        self.buffers.colors.iter().map(|buffer| buffer.id.as_str())
    }

    pub(crate) fn depth_stencil_texture_handle(&self) -> Option<<Context as HasContext>::Texture> {
        self.resources
            .as_ref()
            .and_then(|resources| resources.depth_stencil_handle)
    }

    pub(crate) fn color_texture_handle(
        &self,
        name: &str,
    ) -> Option<<Context as HasContext>::Texture> {
        self.resources.as_ref().and_then(|resources| {
            self.buffers
                .colors
                .iter()
                .position(|buffer| buffer.id == name)
                .map(|i| resources.color_handles[i])
        })
    }

    pub(crate) fn screen_resize(
        &mut self,
        context: &Context,
        width: usize,
        height: usize,
    ) -> Result<(), RenderTargetError> {
        let width = self.preferred_width.width(width, height);
        let height = self.preferred_height.height(width, height);
        if width != self.cached_width || height != self.cached_height {
            self.cached_width = width;
            self.cached_height = height;
            self.context_initialize(context)?;
        }
        Ok(())
    }

    pub(crate) fn render<F>(
        &self,
        context: &Context,
        clear_settings: ClearSettings,
        f: F,
    ) -> Result<(), RenderTargetError>
    where
        F: FnOnce(&Context),
    {
        if let Some(resources) = &self.resources {
            unsafe { context.bind_framebuffer(FRAMEBUFFER, Some(resources.buffer_handle)) };
        } else if self.backbuffer {
            unsafe { context.bind_framebuffer(FRAMEBUFFER, None) };
        } else {
            return Err(RenderTargetError::NoResources);
        }
        unsafe {
            context.enable(BLEND);
            context.disable(SCISSOR_TEST);
            context.viewport(0, 0, self.cached_width as _, self.cached_height as _);
            context.color_mask(true, true, true, true);
            context.depth_mask(true);
            let mut mask = 0;
            if let Some(color) = clear_settings.color {
                context.clear_color(color.r as _, color.g as _, color.b as _, color.a as _);
                mask |= COLOR_BUFFER_BIT;
            }
            if clear_settings.depth {
                mask |= DEPTH_BUFFER_BIT;
            }
            if clear_settings.stencil {
                mask |= STENCIL_BUFFER_BIT;
            }
            if mask != 0 {
                context.clear(mask);
            }
        }
        f(context);
        if let Some(resources) = &self.resources {
            for (handle, buffer) in resources
                .color_handles
                .iter()
                .zip(self.buffers.colors.iter())
            {
                if let (TargetValueType::Color, ImageMipmap::Generate(_)) =
                    (buffer.value_type, buffer.mipmap)
                {
                    unsafe {
                        context.bind_texture(TEXTURE_2D, Some(*handle));
                        context.generate_mipmap(TEXTURE_2D);
                    }
                }
            }
            unsafe {
                context.bind_texture(TEXTURE_2D, None);
            }
        }
        Ok(())
    }
}
