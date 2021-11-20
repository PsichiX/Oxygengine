use crate::{
    ha_renderer::RenderStageResources, math::Rect, render_target::RenderTargetId,
    resources::resource_mapping::ResourceMapping, HasContextResources, ResourceInstanceReference,
};
use core::{id::ID, Ignite};
use glow::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ImageError {
    /// (provided, expected)
    InvalidSize(usize, usize),
    Internal(String),
}

pub type ImageInstanceReference = ResourceInstanceReference<ImageId, VirtualImageId>;
pub type ImageResourceMapping = ResourceMapping<Image, VirtualImage>;

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    RGBA,
    RGB,
    Luminance,
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self::RGBA
    }
}

impl ImageFormat {
    pub fn bytesize(self) -> usize {
        let size = match self {
            Self::RGBA => 4,
            Self::RGB => 3,
            Self::Luminance => 1,
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

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImageDescriptor {
    #[serde(default)]
    pub format: ImageFormat,
    #[serde(default)]
    pub filtering: ImageFiltering,
    #[serde(default)]
    pub mipmap: bool,
}

#[derive(Debug)]
pub struct ImageResources {
    pub handle: <Context as HasContext>::Texture,
}

pub type ImageId = ID<Image>;

#[derive(Debug)]
pub struct ImageDetailedInfo {
    pub format: ImageFormat,
    pub filtering: ImageFiltering,
    pub mipmap: bool,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug)]
pub struct Image {
    format: ImageFormat,
    filtering: ImageFiltering,
    mipmap: bool,
    width: usize,
    height: usize,
    data: Vec<u8>,
    resources: Option<ImageResources>,
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

        unsafe {
            context.bind_texture(TEXTURE_2D, Some(handle));
            let format = match self.format {
                ImageFormat::RGBA => (RGBA),
                ImageFormat::RGB => (RGB),
                ImageFormat::Luminance => (LUMINANCE),
            };
            context.tex_image_2d(
                TEXTURE_2D,
                0,
                format as _,
                self.width as _,
                self.height as _,
                0,
                format,
                UNSIGNED_BYTE,
                Some(&self.data),
            );
            let (min, mag) = match self.filtering {
                ImageFiltering::Nearest => (NEAREST, NEAREST),
                ImageFiltering::Linear => (LINEAR, LINEAR),
                ImageFiltering::Bilinear => (LINEAR_MIPMAP_LINEAR, LINEAR),
            };
            context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, min as _);
            context.tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, mag as _);
            context.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _);
            context.tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _);
            context.generate_mipmap(TEXTURE_2D);
        }

        self.resources = Some(ImageResources { handle });
        Ok(())
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
        data: Vec<u8>,
    ) -> Result<Self, ImageError> {
        let ImageDescriptor {
            format,
            filtering,
            mipmap,
        } = descriptor;
        let size = format.bytesize() * width * height;
        if size == data.len() {
            Ok(Self {
                format,
                filtering,
                mipmap,
                width,
                height,
                data,
                resources: None,
            })
        } else {
            Err(ImageError::InvalidSize(data.len(), size))
        }
    }

    pub fn detailed_info(&self) -> ImageDetailedInfo {
        ImageDetailedInfo {
            format: self.format,
            filtering: self.filtering,
            mipmap: self.mipmap,
            width: self.width,
            height: self.height,
        }
    }

    pub fn format(&self) -> ImageFormat {
        self.format
    }

    pub fn filtering(&self) -> ImageFiltering {
        self.filtering
    }

    pub fn mipmap(&self) -> bool {
        self.mipmap
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn resources<'a>(&self, _: &RenderStageResources<'a>) -> Option<&ImageResources> {
        self.resources.as_ref()
    }
}

pub type VirtualImageId = ID<VirtualImage>;

#[derive(Debug)]
pub struct VirtualImageDetailedInfo {
    pub source: VirtualImageSource,
    pub uvs: HashMap<ImageId, Rect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualImageSource {
    Image(ImageId),
    /// (render target id, output name)
    RenderTarget(RenderTargetId, String),
}

impl VirtualImageSource {
    pub fn image(&self) -> Option<ImageId> {
        match self {
            Self::Image(id) => Some(*id),
            _ => None,
        }
    }

    pub fn render_target(&self) -> Option<(RenderTargetId, &str)> {
        match self {
            Self::RenderTarget(id, name) => Some((*id, name.as_str())),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct VirtualImage {
    source: VirtualImageSource,
    uvs: HashMap<ImageId, Rect>,
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

    pub fn register_image_uvs(&mut self, uvs: Rect) -> ImageId {
        let id = ImageId::new();
        self.uvs.insert(id, uvs);
        id
    }

    pub fn register_named_image_uvs(&mut self, name: impl ToString, uvs: Rect) -> ImageId {
        let id = self.register_image_uvs(uvs);
        self.map.insert(name.to_string(), id);
        self.table.insert(id, name.to_string());
        id
    }

    pub fn unregister_image_uvs(&mut self, id: ImageId) -> Option<Rect> {
        if let Some(name) = self.table.remove(&id) {
            self.map.remove(&name);
        }
        self.uvs.remove(&id)
    }

    pub fn unregister_named_image_uvs(&mut self, name: &str) -> Option<Rect> {
        if let Some(id) = self.map.remove(name) {
            self.table.remove(&id);
            return self.uvs.remove(&id);
        }
        None
    }

    pub fn image_uvs(&self, id: ImageId) -> Option<Rect> {
        self.uvs.get(&id).copied()
    }

    pub fn named_image_uvs(&self, name: &str) -> Option<Rect> {
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
