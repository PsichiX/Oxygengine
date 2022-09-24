use crate::{
    ha_renderer::RenderStageResources, math::Rect, render_target::RenderTargetId,
    resources::resource_mapping::ResourceMapping, HasContextResources, ResourceReference,
};
use core::{id::ID, Ignite};
use glow::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone)]
pub enum ImageError {
    NoResources,
    /// (provided, expected)
    InvalidSize(usize, usize),
    Internal(String),
}

pub type ImageId = ID<Image>;
pub type VirtualImageId = ID<VirtualImage>;
pub type ImageReference = ResourceReference<ImageId, VirtualImageId>;
pub type ImageResourceMapping = ResourceMapping<Image, VirtualImage>;

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageMode {
    Image2d,
    Image2dArray,
    Image3d,
}

impl Default for ImageMode {
    fn default() -> Self {
        Self::Image2d
    }
}

impl ImageMode {
    pub fn as_gl(&self) -> u32 {
        match self {
            Self::Image2d => TEXTURE_2D,
            Self::Image2dArray => TEXTURE_2D_ARRAY,
            Self::Image3d => TEXTURE_3D,
        }
    }
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    RGBA,
    RGB,
    Luminance,
    Data,
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self::RGBA
    }
}

impl ImageFormat {
    pub fn as_internal_format_gl(&self) -> u32 {
        match self {
            Self::RGBA => RGBA,
            Self::RGB => RGB,
            Self::Luminance => LUMINANCE,
            Self::Data => RGBA32F,
        }
    }

    pub fn as_format_gl(&self) -> u32 {
        match self {
            Self::RGBA => RGBA,
            Self::RGB => RGB,
            Self::Luminance => LUMINANCE,
            Self::Data => RGBA,
        }
    }

    pub fn as_type_gl(&self) -> u32 {
        match self {
            Self::Data => FLOAT,
            _ => UNSIGNED_BYTE,
        }
    }

    pub fn alignment(&self) -> usize {
        let size = match self {
            Self::RGBA | Self::Data => 4,
            _ => 1,
        };
        std::mem::size_of::<u8>() * size
    }

    pub fn bytesize(self) -> usize {
        let size = match self {
            Self::RGBA => 4,
            Self::RGB => 3,
            Self::Luminance => 1,
            Self::Data => 16,
        };
        std::mem::size_of::<u8>() * size
    }
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFiltering {
    Nearest,
    Linear,
    Bilinear,
}

impl Default for ImageFiltering {
    fn default() -> Self {
        Self::Linear
    }
}

impl ImageFiltering {
    pub fn as_gl(&self) -> (u32, u32) {
        match self {
            ImageFiltering::Nearest => (NEAREST, NEAREST),
            ImageFiltering::Linear => (LINEAR, LINEAR),
            ImageFiltering::Bilinear => (LINEAR_MIPMAP_LINEAR, LINEAR),
        }
    }
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageMipmap {
    None,
    /// (maximum levels?)
    Generate(Option<usize>),
}

impl Default for ImageMipmap {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageDescriptor {
    #[serde(default)]
    pub mode: ImageMode,
    #[serde(default)]
    pub format: ImageFormat,
    #[serde(default)]
    pub mipmap: ImageMipmap,
}

#[derive(Debug)]
pub struct ImageResources {
    pub handle: <Context as HasContext>::Texture,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageDetailedInfo {
    pub mode: ImageMode,
    pub format: ImageFormat,
    pub mipmap: ImageMipmap,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
}

#[derive(Debug)]
pub struct Image {
    mode: ImageMode,
    format: ImageFormat,
    mipmap: ImageMipmap,
    width: usize,
    height: usize,
    depth: usize,
    data: Vec<u8>,
    resources: Option<ImageResources>,
    dirty: bool,
}

impl Drop for Image {
    fn drop(&mut self) {
        if self.resources.is_some() {
            panic!(
                "Dropping {} without calling `context_release` to release resources first!",
                std::any::type_name::<Self>()
            );
        }
    }
}

impl HasContextResources<Context> for Image {
    type Error = ImageError;

    fn has_resources(&self) -> bool {
        self.resources.is_some()
    }

    fn context_initialize(&mut self, context: &Context) -> Result<(), Self::Error> {
        self.context_release(context)?;

        let handle = match unsafe { context.create_texture() } {
            Ok(handle) => handle,
            Err(error) => return Err(ImageError::Internal(error)),
        };
        self.resources = Some(ImageResources { handle });
        self.maintain(context)
    }

    fn context_release(&mut self, context: &Context) -> Result<(), Self::Error> {
        if let Some(resources) = std::mem::take(&mut self.resources) {
            unsafe {
                context.delete_texture(resources.handle);
            }
        }
        Ok(())
    }
}

impl Image {
    pub fn new(
        descriptor: ImageDescriptor,
        width: usize,
        height: usize,
        depth: usize,
        data: Vec<u8>,
    ) -> Result<Self, ImageError> {
        let ImageDescriptor {
            mode,
            format,
            mipmap,
        } = descriptor;
        let size = format.bytesize() * width * height * depth;
        if size == data.len() {
            Ok(Self {
                mode,
                format,
                mipmap,
                width,
                height,
                depth,
                data,
                resources: None,
                dirty: true,
            })
        } else {
            Err(ImageError::InvalidSize(data.len(), size))
        }
    }

    pub fn detailed_info(&self) -> ImageDetailedInfo {
        ImageDetailedInfo {
            mode: self.mode,
            format: self.format,
            mipmap: self.mipmap,
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }

    pub fn mode(&self) -> ImageMode {
        self.mode
    }

    pub fn format(&self) -> ImageFormat {
        self.format
    }

    pub fn mipmap(&self) -> ImageMipmap {
        self.mipmap
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) -> Result<(), ImageError> {
        let size = self.format.bytesize() * self.width * self.height * self.depth;
        if size == data.len() {
            self.data = data;
            self.dirty = true;
            Ok(())
        } else {
            Err(ImageError::InvalidSize(data.len(), size))
        }
    }

    pub fn with_data<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut [u8]),
    {
        f(&mut self.data);
        self.dirty = true;
    }

    pub fn overwrite(
        &mut self,
        width: usize,
        height: usize,
        depth: usize,
        data: Vec<u8>,
    ) -> Result<(), ImageError> {
        let size = self.format.bytesize() * width * height * depth;
        if size == data.len() {
            self.width = width;
            self.height = height;
            self.depth = depth;
            self.data = data;
            self.dirty = true;
            Ok(())
        } else {
            Err(ImageError::InvalidSize(data.len(), size))
        }
    }

    pub fn resources<'a>(&self, _: &RenderStageResources<'a>) -> Option<&ImageResources> {
        self.resources.as_ref()
    }

    pub(crate) fn maintain(&mut self, context: &Context) -> Result<(), ImageError> {
        let resources = match &self.resources {
            Some(resources) => resources,
            None => return Err(ImageError::NoResources),
        };
        if self.dirty {
            unsafe {
                let gl_mode = self.mode.as_gl();
                let gl_internal_format = self.format.as_internal_format_gl();
                let gl_format = self.format.as_format_gl();
                let gl_type = self.format.as_type_gl();
                let alignment = self.format.alignment();
                context.bind_texture(gl_mode, Some(resources.handle));
                context.pixel_store_i32(PACK_ALIGNMENT, alignment as _);
                context.pixel_store_i32(UNPACK_ALIGNMENT, alignment as _);
                match self.mode {
                    ImageMode::Image2d => {
                        context.tex_image_2d(
                            gl_mode,
                            0,
                            gl_internal_format as _,
                            self.width as _,
                            self.height as _,
                            0,
                            gl_format,
                            gl_type,
                            Some(&self.data),
                        );
                    }
                    ImageMode::Image2dArray | ImageMode::Image3d => {
                        context.tex_image_3d(
                            gl_mode,
                            0,
                            gl_internal_format as _,
                            self.width as _,
                            self.height as _,
                            self.depth as _,
                            0,
                            gl_format,
                            gl_type,
                            Some(&self.data),
                        );
                    }
                }
                if let ImageMipmap::Generate(limit) = self.mipmap {
                    if let Some(limit) = limit {
                        context.tex_parameter_i32(gl_mode, TEXTURE_MAX_LEVEL, limit as i32);
                    }
                    context.generate_mipmap(gl_mode);
                }
                context.bind_texture(gl_mode, None);
            }
            self.dirty = false;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct VirtualImageDetailedInfo {
    pub source: VirtualImageSource,
    pub uvs: HashMap<ImageId, (Rect, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualImageSource {
    Image(ImageId),
    RenderTargetDepthStencil(RenderTargetId),
    /// (render target id, output name)
    RenderTargetColor(RenderTargetId, String),
}

impl VirtualImageSource {
    pub fn image(&self) -> Option<ImageId> {
        match self {
            Self::Image(id) => Some(*id),
            _ => None,
        }
    }

    pub fn render_target_depth_stencil(&self) -> Option<RenderTargetId> {
        match self {
            Self::RenderTargetDepthStencil(id) => Some(*id),
            _ => None,
        }
    }

    pub fn render_target_color(&self) -> Option<(RenderTargetId, &str)> {
        match self {
            Self::RenderTargetColor(id, name) => Some((*id, name.as_str())),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct VirtualImage {
    source: VirtualImageSource,
    uvs: HashMap<ImageId, (Rect, usize)>,
    map: HashMap<String, ImageId>,
    table: HashMap<ImageId, String>,
}

impl VirtualImage {
    pub fn new(source: VirtualImageSource) -> Self {
        Self {
            source,
            uvs: Default::default(),
            map: Default::default(),
            table: Default::default(),
        }
    }

    pub fn source(&self) -> &VirtualImageSource {
        &self.source
    }

    pub fn register_image_uvs(&mut self, uvs: Rect, page: usize) -> ImageId {
        let id = ImageId::new();
        self.uvs.insert(id, (uvs, page));
        id
    }

    pub fn register_named_image_uvs(
        &mut self,
        name: impl ToString,
        uvs: Rect,
        page: usize,
    ) -> ImageId {
        let id = self.register_image_uvs(uvs, page);
        self.map.insert(name.to_string(), id);
        self.table.insert(id, name.to_string());
        id
    }

    pub fn unregister_image_uvs(&mut self, id: ImageId) -> Option<(Rect, usize)> {
        if let Some(name) = self.table.remove(&id) {
            self.map.remove(&name);
        }
        self.uvs.remove(&id)
    }

    pub fn unregister_named_image_uvs(&mut self, name: &str) -> Option<(Rect, usize)> {
        if let Some(id) = self.map.remove(name) {
            self.table.remove(&id);
            return self.uvs.remove(&id);
        }
        None
    }

    pub fn image_uvs(&self, id: ImageId) -> Option<(Rect, usize)> {
        self.uvs.get(&id).copied()
    }

    pub fn named_image_uvs(&self, name: &str) -> Option<(Rect, usize)> {
        if let Some(id) = self.map.get(name) {
            return self.uvs.get(id).copied();
        }
        None
    }

    pub fn image_name(&self, id: ImageId) -> Option<&str> {
        self.table.get(&id).map(|name| name.as_str())
    }

    pub fn detailed_info(&self) -> VirtualImageDetailedInfo {
        VirtualImageDetailedInfo {
            source: self.source.to_owned(),
            uvs: self.uvs.to_owned(),
        }
    }
}
