pub mod vertex_factory;

use crate::{
    ha_renderer::{RenderStageResources, RenderStats},
    math::*,
    resources::resource_mapping::ResourceMapping,
    HasContextResources, ResourceInstanceReference,
};
use core::id::ID;
use glow::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Range};

#[derive(Debug, Clone)]
pub enum MeshError {
    InvalidId(String),
    DuplicateId(String),
    ZeroSize,
    /// (provided, expected)
    InvalidSize(usize, usize),
    /// (provided limit, expected limit)
    OutOfBounds(usize, usize),
    NoResources,
    /// (provided index, buffers count)
    NoBuffer(usize, usize),
    /// (provided, expected)
    LayoutsMismatch(VertexLayout, VertexLayout),
    /// (layout, attribute name)
    MissingRequiredLayoutAttribute(VertexLayout, String),
    /// (source, target)
    IncompatibleDrawMode(MeshDrawMode, MeshDrawMode),
    Internal(String),
}

pub type MeshInstanceReference = ResourceInstanceReference<MeshId, VirtualMeshId>;
pub type MeshResourceMapping = ResourceMapping<Mesh, VirtualMesh>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BufferStorage {
    Static,
    Dynamic,
    Stream,
}

impl Default for BufferStorage {
    fn default() -> Self {
        Self::Static
    }
}

impl BufferStorage {
    pub fn as_gl(self) -> u32 {
        match self {
            Self::Static => STATIC_DRAW,
            Self::Dynamic => DYNAMIC_DRAW,
            Self::Stream => STREAM_DRAW,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VertexValueType {
    Scalar,
    Vec2F,
    Vec3F,
    Vec4F,
    Mat2F,
    Mat3F,
    Mat4F,
    Integer,
    Vec2I,
    Vec3I,
    Vec4I,
    Mat2I,
    Mat3I,
    Mat4I,
}

impl VertexValueType {
    pub fn is_integer(self) -> bool {
        matches!(
            self,
            Self::Integer
                | Self::Vec2I
                | Self::Vec3I
                | Self::Vec4I
                | Self::Mat2I
                | Self::Mat3I
                | Self::Mat4I
        )
    }

    pub fn channels(self) -> usize {
        match self {
            Self::Scalar | Self::Integer => 1,
            Self::Vec2F | Self::Vec2I => 2,
            Self::Vec3F | Self::Vec3I => 3,
            Self::Vec4F | Self::Vec4I => 4,
            Self::Mat2F | Self::Mat2I => 2,
            Self::Mat3F | Self::Mat3I => 3,
            Self::Mat4F | Self::Mat4I => 4,
        }
    }

    pub fn count(self) -> usize {
        match self {
            Self::Scalar | Self::Integer => 1,
            Self::Vec2F | Self::Vec2I => 2,
            Self::Vec3F | Self::Vec3I => 3,
            Self::Vec4F | Self::Vec4I => 4,
            Self::Mat2F | Self::Mat2I => 4,
            Self::Mat3F | Self::Mat3I => 9,
            Self::Mat4F | Self::Mat4I => 16,
        }
    }

    pub fn locations(self) -> usize {
        match self {
            Self::Scalar
            | Self::Integer
            | Self::Vec2F
            | Self::Vec2I
            | Self::Vec3F
            | Self::Vec3I
            | Self::Vec4F
            | Self::Vec4I => 1,
            Self::Mat2F | Self::Mat2I => 2,
            Self::Mat3F | Self::Mat3I => 3,
            Self::Mat4F | Self::Mat4I => 4,
        }
    }

    pub fn single_bytesize(self) -> usize {
        if self.is_integer() {
            std::mem::size_of::<i32>()
        } else {
            std::mem::size_of::<f32>()
        }
    }

    pub fn bytesize(self) -> usize {
        self.count() * self.single_bytesize()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexAttribute {
    pub id: String,
    pub count: usize,
    pub value_type: VertexValueType,
    pub normalized: bool,
}

impl VertexAttribute {
    pub fn single(id: String, value_type: VertexValueType) -> Self {
        Self {
            id,
            count: 1,
            value_type,
            normalized: false,
        }
    }

    pub fn single_normalized(id: String, value_type: VertexValueType) -> Self {
        Self {
            id,
            count: 1,
            value_type,
            normalized: true,
        }
    }

    pub fn array(id: String, count: usize, value_type: VertexValueType) -> Self {
        Self {
            id,
            count,
            value_type,
            normalized: false,
        }
    }

    pub fn array_normalized(id: String, count: usize, value_type: VertexValueType) -> Self {
        Self {
            id,
            count,
            value_type,
            normalized: true,
        }
    }

    pub fn locations(&self) -> usize {
        self.count * self.value_type.locations()
    }

    pub fn bytesize(&self) -> usize {
        self.count * self.value_type.bytesize()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexBufferLayout {
    /// [(attribute, byte offset)]
    attributes: Vec<(VertexAttribute, usize)>,
    base_location: usize,
    bytesize: usize,
}

impl VertexBufferLayout {
    pub fn with(mut self, attribute: VertexAttribute) -> Result<Self, MeshError> {
        if self.attributes.iter().any(|a| a.0.id == attribute.id) {
            return Err(MeshError::DuplicateId(attribute.id));
        }
        let offset = self.bytesize;
        self.bytesize += attribute.bytesize();
        self.attributes.push((attribute, offset));
        Ok(self)
    }

    pub fn bytesize(&self) -> usize {
        self.bytesize
    }

    fn vertex_attribs<'a>(&'a self) -> impl Iterator<Item = (&'a str, VertexAttribChunk)> + '_ {
        let stride = self.bytesize;
        let mut base_location = self.base_location;
        self.attributes
            .iter()
            .map(move |(attribute, attribute_offset)| {
                let is_integer = attribute.value_type.is_integer();
                let channels = attribute.value_type.channels();
                let locations = attribute.locations();
                let location = base_location;
                base_location += locations;
                if is_integer {
                    (
                        attribute.id.as_str(),
                        VertexAttribChunk::Integer {
                            location,
                            channels,
                            stride,
                            offset: *attribute_offset,
                        },
                    )
                } else {
                    (
                        attribute.id.as_str(),
                        VertexAttribChunk::Float {
                            location,
                            channels,
                            normalized: attribute.normalized,
                            stride,
                            offset: *attribute_offset,
                        },
                    )
                }
            })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum VertexAttribChunk {
    Float {
        location: usize,
        channels: usize,
        normalized: bool,
        stride: usize,
        offset: usize,
    },
    Integer {
        location: usize,
        channels: usize,
        stride: usize,
        offset: usize,
    },
}

impl VertexAttribChunk {
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Integer { .. })
    }

    pub fn location(&self) -> usize {
        match self {
            Self::Float { location, .. } => *location,
            Self::Integer { location, .. } => *location,
        }
    }

    pub fn channels(&self) -> usize {
        match self {
            Self::Float { channels, .. } => *channels,
            Self::Integer { channels, .. } => *channels,
        }
    }

    pub fn stride(&self) -> usize {
        match self {
            Self::Float { stride, .. } => *stride,
            Self::Integer { stride, .. } => *stride,
        }
    }

    pub fn offset(&self) -> usize {
        match self {
            Self::Float { offset, .. } => *offset,
            Self::Integer { offset, .. } => *offset,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexLayout {
    buffers: Vec<VertexBufferLayout>,
    locations: usize,
    bounds: Option<String>,
}

impl VertexLayout {
    pub fn with(mut self, mut buffer: VertexBufferLayout) -> Result<Self, MeshError> {
        for b in &self.buffers {
            for attribute in &b.attributes {
                if buffer.attributes.iter().any(|a| a.0.id == attribute.0.id) {
                    return Err(MeshError::DuplicateId(attribute.0.id.to_owned()));
                }
            }
        }
        buffer.base_location += self.locations;
        self.locations += buffer.attributes.iter().fold(0, |a, v| a + v.0.locations());
        self.buffers.push(buffer);
        Ok(self)
    }

    pub fn with_bounds(mut self, bounds: Option<String>) -> Self {
        self.bounds = bounds;
        self
    }

    pub fn buffers(&self) -> &[VertexBufferLayout] {
        &self.buffers
    }

    pub fn bounds(&self) -> Option<&str> {
        self.bounds.as_deref()
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.vertex_attribs().all(|(_, na, ca)| {
            other.vertex_attribs().any(|(_, nb, cb)| {
                na == nb && ca.is_integer() == cb.is_integer() && ca.channels() == cb.channels()
            })
        })
    }

    #[inline]
    pub fn is_superset_of(&self, other: &Self) -> bool {
        other.is_subset_of(self)
    }

    pub fn vertex_attribs<'a>(
        &'a self,
    ) -> impl Iterator<Item = (usize, &'a str, VertexAttribChunk)> + '_ {
        self.buffers.iter().enumerate().flat_map(|(index, buffer)| {
            buffer
                .vertex_attribs()
                .map(move |(name, chunk)| (index, name, chunk))
        })
    }

    /// (buffer, channels, offset, stride)?
    pub fn bounds_vertex_attrib(&self) -> Option<(usize, usize, usize, usize)> {
        self.bounds.as_ref().and_then(|bounds| {
            self.vertex_attribs()
                .find(|(_, name, chunk)| name == bounds && !chunk.is_integer())
                .map(|(index, _, chunk)| (index, chunk.channels(), chunk.offset(), chunk.stride()))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshDrawRange {
    All,
    Range(Range<usize>),
    Chunks(Vec<Range<usize>>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshDrawMode {
    Triangles,
    Lines,
    Points,
}

impl Default for MeshDrawMode {
    fn default() -> Self {
        Self::Triangles
    }
}

impl MeshDrawMode {
    pub fn as_gl(self) -> u32 {
        match self {
            Self::Triangles => TRIANGLES,
            Self::Lines => LINES,
            Self::Points => POINTS,
        }
    }

    pub fn indices_count(self) -> usize {
        match self {
            Self::Triangles => 3,
            Self::Lines => 2,
            Self::Points => 1,
        }
    }
}

#[derive(Debug)]
pub struct MeshResources {
    pub vertices_handles: Vec<<Context as HasContext>::Buffer>,
    pub indices_handle: <Context as HasContext>::Buffer,
    pub array_handle: <Context as HasContext>::VertexArray,
}

pub type MeshId = ID<Mesh>;

#[derive(Debug)]
pub struct MeshDetailedInfo {
    pub layout: VertexLayout,
    pub vertex_data: Vec<(usize, BufferStorage, bool)>,
    pub index_data: (usize, BufferStorage, bool),
    pub draw_mode: MeshDrawMode,
}

#[derive(Debug)]
pub struct Mesh {
    layout: VertexLayout,
    /// [([bytes], storage, dirty)]
    vertex_data: Vec<(Vec<u8>, BufferStorage, bool)>,
    /// [(indices, storage, dirty)]
    index_data: (Vec<u32>, BufferStorage, bool),
    draw_mode: MeshDrawMode,
    resources: Option<MeshResources>,
    bounds: Option<BoundsVolume>,
}

impl Drop for Mesh {
    fn drop(&mut self) {
        if self.resources.is_some() {
            panic!(
                "Dropping {} without calling `context_release` to release resources first!",
                std::any::type_name::<Self>()
            );
        }
    }
}

impl HasContextResources<Context> for Mesh {
    type Error = MeshError;

    fn has_resources(&self) -> bool {
        self.resources.is_some()
    }

    fn context_initialize(&mut self, context: &Context) -> Result<(), Self::Error> {
        self.context_release(context)?;

        let array_handle = match unsafe { context.create_vertex_array() } {
            Ok(handle) => handle,
            Err(error) => return Err(MeshError::Internal(error)),
        };
        let indices_handle = match unsafe { context.create_buffer() } {
            Ok(handle) => handle,
            Err(error) => return Err(MeshError::Internal(error)),
        };
        let mut vertices_handles = Vec::with_capacity(self.layout.buffers.len());
        for _ in 0..self.layout.buffers.len() {
            match unsafe { context.create_buffer() } {
                Ok(handle) => vertices_handles.push(handle),
                Err(error) => return Err(MeshError::Internal(error)),
            };
        }

        unsafe {
            context.bind_vertex_array(Some(array_handle));
            context.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(indices_handle));
            for (buffer, handle) in self.layout.buffers.iter().zip(vertices_handles.iter()) {
                context.bind_buffer(ARRAY_BUFFER, Some(*handle));
                for (_, chunk) in buffer.vertex_attribs() {
                    match chunk {
                        VertexAttribChunk::Float {
                            location,
                            channels,
                            normalized,
                            stride,
                            offset,
                        } => {
                            context.vertex_attrib_pointer_f32(
                                location as _,
                                channels as _,
                                FLOAT,
                                normalized,
                                stride as _,
                                offset as _,
                            );
                            context.enable_vertex_attrib_array(location as _);
                        }
                        VertexAttribChunk::Integer {
                            location,
                            channels,
                            stride,
                            offset,
                        } => {
                            context.vertex_attrib_pointer_i32(
                                location as _,
                                channels as _,
                                INT,
                                stride as _,
                                offset as _,
                            );
                            context.enable_vertex_attrib_array(location as _);
                        }
                    }
                }
            }
            context.bind_vertex_array(None);
        }

        self.resources = Some(MeshResources {
            vertices_handles,
            indices_handle,
            array_handle,
        });
        self.maintain(context)
    }

    fn context_release(&mut self, context: &Context) -> Result<(), Self::Error> {
        if let Some(resources) = std::mem::take(&mut self.resources) {
            unsafe {
                context.delete_buffer(resources.indices_handle);
                for handle in resources.vertices_handles {
                    context.delete_buffer(handle);
                }
                context.delete_vertex_array(resources.array_handle);
            }
        }
        Ok(())
    }
}

impl Mesh {
    pub fn new(layout: VertexLayout) -> Self {
        let vertex_data = (0..layout.buffers.len())
            .map(|_| (vec![], BufferStorage::default(), false))
            .collect();
        Self {
            layout,
            vertex_data,
            index_data: (vec![], BufferStorage::default(), false),
            draw_mode: MeshDrawMode::default(),
            resources: None,
            bounds: None,
        }
    }

    pub fn detailed_info(&self) -> MeshDetailedInfo {
        MeshDetailedInfo {
            layout: self.layout.clone(),
            vertex_data: self
                .vertex_data
                .iter()
                .map(|(v, s, d)| (v.len(), *s, *d))
                .collect(),
            index_data: (
                self.index_data.0.len(),
                self.index_data.1,
                self.index_data.2,
            ),
            draw_mode: self.draw_mode,
        }
    }

    pub fn draw_mode(&self) -> MeshDrawMode {
        self.draw_mode
    }

    pub fn are_vertices_dirty(&self, buffer: usize) -> bool {
        match self.vertex_data.get(buffer) {
            Some((_, _, dirty)) => *dirty,
            None => false,
        }
    }

    pub fn are_indices_dirty(&self) -> bool {
        self.index_data.2
    }

    pub fn mark_vertices_dirty(&mut self, buffer: usize) {
        if let Some((_, _, dirty)) = self.vertex_data.get_mut(buffer) {
            *dirty = true;
        }
    }

    pub fn mark_indices_dirty(&mut self) {
        self.index_data.2 = true;
    }

    pub fn layout(&self) -> &VertexLayout {
        &self.layout
    }

    pub fn bounds(&self) -> Option<&BoundsVolume> {
        self.bounds.as_ref()
    }

    pub fn vertex_storage(&self, buffer: usize) -> Option<BufferStorage> {
        self.vertex_data.get(buffer).map(|(_, storage, _)| *storage)
    }

    pub fn index_storage(&self) -> BufferStorage {
        self.index_data.1
    }

    pub fn set_vertex_storage(
        &mut self,
        buffer: usize,
        storage: BufferStorage,
    ) -> Result<(), MeshError> {
        if let Some((_, s, d)) = self.vertex_data.get_mut(buffer) {
            *s = storage;
            *d = true;
            return Ok(());
        }
        Err(MeshError::NoBuffer(buffer, self.vertex_data.len()))
    }

    pub fn set_vertex_storage_all(&mut self, storage: BufferStorage) {
        for (_, s, d) in &mut self.vertex_data {
            *s = storage;
            *d = true;
        }
    }

    pub fn set_index_storage(&mut self, storage: BufferStorage) {
        self.index_data.1 = storage;
        self.index_data.2 = true;
    }

    pub fn vertex_data(&self, buffer: usize) -> Option<&[u8]> {
        self.vertex_data
            .get(buffer)
            .map(|(data, _, _)| data.as_slice())
    }

    pub fn index_data(&self) -> &[u32] {
        &self.index_data.0
    }

    pub fn set_vertex_data(&mut self, buffer: usize, data: Vec<u8>) -> Result<(), MeshError> {
        let (vertex_data, dirty) = match self.vertex_data.get_mut(buffer) {
            Some((vertex_data, _, dirty)) => (vertex_data, dirty),
            None => return Err(MeshError::NoBuffer(buffer, self.vertex_data.len())),
        };
        let buffer = match self.layout.buffers.get(buffer) {
            Some(buffer) => buffer,
            None => return Err(MeshError::NoBuffer(buffer, self.layout.buffers.len())),
        };
        let bytesize = buffer.bytesize();
        if bytesize == 0 {
            return Err(MeshError::ZeroSize);
        }
        let size = data.len();
        let count = size / bytesize;
        let expected_bytesize = count * bytesize;
        if size != expected_bytesize {
            return Err(MeshError::InvalidSize(size, expected_bytesize));
        }
        *vertex_data = data;
        *dirty = true;
        self.bounds = None;
        Ok(())
    }

    pub fn set_index_data(&mut self, data: Vec<u32>, draw_mode: MeshDrawMode) {
        self.index_data.0 = data;
        self.index_data.2 = true;
        self.draw_mode = draw_mode;
    }

    pub fn write_vertex_data_range(
        &mut self,
        buffer: usize,
        data: Vec<u8>,
        start: usize,
    ) -> Result<(), MeshError> {
        let (vertex_data, dirty) = match self.vertex_data.get_mut(buffer) {
            Some((vertex_data, _, dirty)) => (vertex_data, dirty),
            None => return Err(MeshError::NoBuffer(buffer, self.vertex_data.len())),
        };
        let buffer = match self.layout.buffers.get(buffer) {
            Some(buffer) => buffer,
            None => return Err(MeshError::NoBuffer(buffer, self.layout.buffers.len())),
        };
        let bytesize = buffer.bytesize();
        if bytesize == 0 {
            return Err(MeshError::ZeroSize);
        }
        let size = data.len();
        let count = size / bytesize;
        let expected_bytesize = count * bytesize;
        if size != expected_bytesize {
            return Err(MeshError::InvalidSize(size, expected_bytesize));
        }
        let limit = start + size;
        let count = vertex_data.len();
        if limit > count {
            return Err(MeshError::OutOfBounds(limit, count));
        }
        let start = start * bytesize;
        vertex_data[start..limit].copy_from_slice(&data);
        *dirty = true;
        self.bounds = None;
        Ok(())
    }

    pub fn write_index_data_range(
        &mut self,
        data: Vec<u32>,
        start: usize,
    ) -> Result<(), MeshError> {
        let limit = start + data.len();
        let count = self.index_data.0.len();
        if limit > count {
            return Err(MeshError::OutOfBounds(limit, count));
        }
        let start = start * std::mem::size_of::<u32>();
        self.index_data.0[start..limit].copy_from_slice(&data);
        self.index_data.2 = true;
        Ok(())
    }

    pub fn with_vertices<F>(&mut self, buffer: usize, mut f: F) -> Result<(), MeshError>
    where
        F: FnMut(&mut [u8]),
    {
        let (vertex_data, dirty) = match self.vertex_data.get_mut(buffer) {
            Some((vertex_data, _, dirty)) => (vertex_data, dirty),
            None => return Err(MeshError::NoBuffer(buffer, self.vertex_data.len())),
        };
        f(vertex_data);
        *dirty = true;
        self.bounds = None;
        Ok(())
    }

    pub fn with_indices<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut [u32]),
    {
        f(&mut self.index_data.0);
        self.index_data.2 = true;
    }

    pub fn resources<'a>(&self, _: &RenderStageResources<'a>) -> Option<&MeshResources> {
        self.resources.as_ref()
    }

    pub(crate) fn activate(
        &self,
        context: &Context,
        render_stats: &mut RenderStats,
    ) -> Result<(), MeshError> {
        let resources = match &self.resources {
            Some(resources) => resources,
            None => return Err(MeshError::NoResources),
        };
        unsafe {
            context.bind_vertex_array(Some(resources.array_handle));
            render_stats.mesh_changes += 1;
        }
        Ok(())
    }

    pub(crate) fn draw(
        &self,
        range: MeshDrawRange,
        context: &Context,
        render_stats: &mut RenderStats,
    ) -> Result<(), MeshError> {
        if self.resources.is_none() {
            return Err(MeshError::NoResources);
        }
        match range {
            MeshDrawRange::All => {
                self.draw_range(0..self.index_data.0.len(), context, render_stats)
            }
            MeshDrawRange::Range(range) => self.draw_range(range, context, render_stats),
            MeshDrawRange::Chunks(chunks) => {
                for range in chunks {
                    self.draw_range(range, context, render_stats)?;
                }
                Ok(())
            }
        }
    }

    fn draw_range(
        &self,
        range: Range<usize>,
        context: &Context,
        render_stats: &mut RenderStats,
    ) -> Result<(), MeshError> {
        let count = range.end - range.start;
        let offset = range.start;
        unsafe {
            context.draw_elements(
                self.draw_mode.as_gl(),
                count as i32,
                UNSIGNED_INT,
                (offset * std::mem::size_of::<u32>()) as i32,
            );
            render_stats.draw_calls += 1;
        }
        Ok(())
    }

    pub(crate) fn maintain(&mut self, context: &Context) -> Result<(), MeshError> {
        let resources = match &self.resources {
            Some(resources) => resources,
            None => return Err(MeshError::NoResources),
        };
        if self.index_data.2 {
            unsafe {
                context.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(resources.indices_handle));
                context.buffer_data_u8_slice(
                    ELEMENT_ARRAY_BUFFER,
                    self.index_data.0.align_to().1,
                    self.index_data.1.as_gl(),
                );
            }
            self.index_data.2 = false;
        }
        for (index, (vertex_data, storage, dirty)) in self.vertex_data.iter_mut().enumerate() {
            if *dirty {
                unsafe {
                    context.bind_buffer(ARRAY_BUFFER, Some(resources.vertices_handles[index]));
                    context.buffer_data_u8_slice(ARRAY_BUFFER, vertex_data, storage.as_gl());
                }
                *dirty = false;
            }
        }
        if self.bounds.is_none() {
            if let Some((buffer, channels, offset, stride)) = self.layout().bounds_vertex_attrib() {
                if let Some((buffer, _, _)) = self.vertex_data.get(buffer) {
                    self.bounds = BoundsVolume::from_points_cloud(
                        buffer.chunks_exact(stride).filter_map(|bytes| unsafe {
                            let bytes =
                                &bytes[offset..(offset + channels * std::mem::size_of::<f32>())];
                            let values = bytes.as_ptr() as *const f32;
                            match channels {
                                1 => Some(Vec3::new(*values, 0.0, 0.0)),
                                2 => Some(Vec3::new(*values, *values.offset(1), 0.0)),
                                3 | 4 => {
                                    Some(Vec3::new(*values, *values.offset(1), *values.offset(2)))
                                }
                                _ => None,
                            }
                        }),
                    );
                }
            }
        }
        Ok(())
    }
}

pub type VirtualMeshId = ID<VirtualMesh>;

#[derive(Debug)]
pub struct VirtualMeshDetailedInfo {
    pub source: MeshId,
    pub ranges: HashMap<MeshId, Range<usize>>,
}

#[derive(Debug)]
pub struct VirtualMesh {
    source: MeshId,
    ranges: HashMap<MeshId, Range<usize>>,
}

impl VirtualMesh {
    pub fn new(source: MeshId) -> Self {
        Self {
            source,
            ranges: Default::default(),
        }
    }

    pub fn source(&self) -> MeshId {
        self.source
    }

    pub fn register_mesh_range(&mut self, range: Range<usize>) -> MeshId {
        let id = MeshId::new();
        self.ranges.insert(id, range);
        id
    }

    pub fn unregister_mesh_range(&mut self, id: MeshId) -> Option<Range<usize>> {
        self.ranges.remove(&id)
    }

    pub fn mesh_range(&self, id: MeshId) -> Option<Range<usize>> {
        self.ranges.get(&id).cloned()
    }

    pub fn detailed_info(&self) -> VirtualMeshDetailedInfo {
        VirtualMeshDetailedInfo {
            source: self.source,
            ranges: self.ranges.to_owned(),
        }
    }
}
