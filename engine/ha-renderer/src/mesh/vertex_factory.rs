use crate::mesh::*;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StreamingVertexFactory {
    layout: VertexLayout,
    buffers: Vec<Vec<u8>>,
    indices: Vec<u32>,
    vertices: usize,
    draw_mode: MeshDrawMode,
}

impl StreamingVertexFactory {
    pub fn new(layout: VertexLayout, draw_mode: MeshDrawMode) -> Self {
        let buffers = layout.buffers.iter().map(|_| vec![]).collect();
        let indices = vec![];
        Self {
            layout,
            buffers,
            indices,
            vertices: 0,
            draw_mode,
        }
    }

    pub fn layout(&self) -> &VertexLayout {
        &self.layout
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn draw_mode(&self) -> MeshDrawMode {
        self.draw_mode
    }

    pub fn clear(&mut self) {
        for buffer in &mut self.buffers {
            buffer.clear();
        }
        self.indices.clear();
        self.vertices = 0;
    }

    pub fn shrink_to_fit(&mut self) {
        for buffer in &mut self.buffers {
            buffer.shrink_to_fit();
        }
        self.indices.shrink_to_fit();
    }

    pub fn write_from(&mut self, factory: &StaticVertexFactory) -> Result<(), MeshError> {
        if self.draw_mode != factory.draw_mode {
            return Err(MeshError::IncompatibleDrawMode(
                factory.draw_mode,
                self.draw_mode,
            ));
        }
        let offset = self.vertices as u32;
        if self.layout == factory.layout {
            for (from, to) in factory.buffers.iter().zip(self.buffers.iter_mut()) {
                to.extend(from);
            }
            self.indices.reserve(factory.indices.len());
            for index in &factory.indices {
                self.indices.push(index + offset);
            }
            self.vertices += factory.vertices;
            return Ok(());
        }
        if !self.layout.is_superset_of(&factory.layout) {
            return Err(MeshError::LayoutsMismatch(
                Box::new(factory.layout.to_owned()),
                Box::new(factory.layout.to_owned()),
            ));
        }
        for (buffer, layout_buffer) in self.buffers.iter_mut().zip(self.layout.buffers()) {
            buffer.resize(
                buffer.len() + layout_buffer.bytesize() * factory.vertices,
                0,
            );
        }
        for (ba, na, ca) in self.layout.vertex_attribs() {
            // 4 is for number of bytes, assuming both f32 and i32.
            let linesize = ca.channels() * 4;
            let offset = self.layout.buffers()[ba].bytesize() * self.vertices;
            let oa = ca.offset();
            if let Some((bb, _, cb)) = factory.layout.vertex_attribs().find(|(_, nb, cb)| {
                &na == nb && ca.is_integer() == cb.is_integer() && ca.channels() == cb.channels()
            }) {
                let ob = cb.offset();
                let iter = self.buffers[ba][offset..]
                    .chunks_exact_mut(ca.stride())
                    .map(|ba| &mut ba[oa..(oa + linesize)])
                    .zip(
                        factory.buffers[bb]
                            .chunks_exact(cb.stride())
                            .map(|bb| &bb[ob..(ob + linesize)]),
                    );
                for (ba, bb) in iter {
                    ba.copy_from_slice(bb);
                }
            }
        }
        self.indices.reserve(factory.indices.len());
        for index in &factory.indices {
            self.indices.push(index + offset);
        }
        self.vertices += factory.vertices;
        Ok(())
    }

    pub fn write_into(&self, mesh: &mut Mesh) -> Result<(), MeshError> {
        if mesh.layout != self.layout {
            return Err(MeshError::LayoutsMismatch(
                Box::new(self.layout.to_owned()),
                Box::new(mesh.layout.to_owned()),
            ));
        }
        for (index, data) in self.buffers.iter().enumerate() {
            mesh.set_vertex_data(index, data.to_owned())?;
        }
        mesh.set_index_data(self.indices.to_owned(), self.draw_mode);
        Ok(())
    }

    pub fn consume_write_into(self, mesh: &mut Mesh) -> Result<(), MeshError> {
        if mesh.layout != self.layout {
            return Err(MeshError::LayoutsMismatch(
                Box::new(self.layout),
                Box::new(mesh.layout.to_owned()),
            ));
        }
        for (index, data) in self.buffers.into_iter().enumerate() {
            mesh.set_vertex_data(index, data)?;
        }
        mesh.set_index_data(self.indices, self.draw_mode);
        Ok(())
    }

    /// (vertex layout, vertex buffers, index buffer, vertex count)
    pub fn into_inner(self) -> (VertexLayout, Vec<Vec<u8>>, Vec<u32>, usize) {
        (self.layout, self.buffers, self.indices, self.vertices)
    }
}

impl From<StaticVertexFactory> for StreamingVertexFactory {
    fn from(v: StaticVertexFactory) -> Self {
        let StaticVertexFactory {
            layout,
            buffers,
            indices,
            vertices,
            draw_mode,
        } = v;
        Self {
            layout,
            buffers,
            indices,
            vertices,
            draw_mode,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StaticVertexFactory {
    layout: VertexLayout,
    buffers: Vec<Vec<u8>>,
    indices: Vec<u32>,
    vertices: usize,
    draw_mode: MeshDrawMode,
}

macro_rules! impl_static_vertex_factory_vertices {
    ($method:ident, $value:expr, $type:ty) => {
        pub fn $method(
            &mut self,
            name: &str,
            data: &[$type],
            start: Option<usize>,
        ) -> Result<(), MeshError> {
            if let Some((index, offset, stride)) = self.find_buffer_offset_stride(name, $value) {
                let iter = data.iter().zip(
                    self.buffers[index][start.unwrap_or(0)..]
                        .chunks_exact_mut(stride)
                        .map(|bytes| unsafe { bytes.as_mut_ptr().add(offset) as *mut $type }),
                );
                for (source, target) in iter {
                    unsafe {
                        target.replace(*source);
                    }
                }
                Ok(())
            } else {
                Err(MeshError::InvalidId(name.to_owned()))
            }
        }
    };
}

impl StaticVertexFactory {
    pub fn new(
        layout: VertexLayout,
        vertex_count: usize,
        elements_count: usize,
        draw_mode: MeshDrawMode,
    ) -> Self {
        let buffers = layout
            .buffers
            .iter()
            .map(|buffer| vec![0; buffer.bytesize() * vertex_count])
            .collect();
        let indices = vec![0; elements_count * draw_mode.indices_count()];
        Self {
            layout,
            buffers,
            indices,
            vertices: vertex_count,
            draw_mode,
        }
    }

    pub fn layout(&self) -> &VertexLayout {
        &self.layout
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    pub fn draw_mode(&self) -> MeshDrawMode {
        self.draw_mode
    }

    impl_static_vertex_factory_vertices! {vertices_scalar, VertexValueType::Scalar, f32}
    impl_static_vertex_factory_vertices! {vertices_vec2f, VertexValueType::Vec2F, vek::Vec2<f32>}
    impl_static_vertex_factory_vertices! {vertices_vec3f, VertexValueType::Vec3F, vek::Vec3<f32>}
    impl_static_vertex_factory_vertices! {vertices_vec4f, VertexValueType::Vec4F, vek::Vec4<f32>}
    impl_static_vertex_factory_vertices! {vertices_mat2f, VertexValueType::Mat2F, vek::Mat2<f32>}
    impl_static_vertex_factory_vertices! {vertices_mat3f, VertexValueType::Mat3F, vek::Mat3<f32>}
    impl_static_vertex_factory_vertices! {vertices_mat4f, VertexValueType::Mat4F, vek::Mat4<f32>}
    impl_static_vertex_factory_vertices! {vertices_integer, VertexValueType::Integer, i32}
    impl_static_vertex_factory_vertices! {vertices_vec2i, VertexValueType::Vec2I, vek::Vec2<i32>}
    impl_static_vertex_factory_vertices! {vertices_vec3i, VertexValueType::Vec3I, vek::Vec3<i32>}
    impl_static_vertex_factory_vertices! {vertices_vec4i, VertexValueType::Vec4I, vek::Vec4<i32>}
    impl_static_vertex_factory_vertices! {vertices_mat2i, VertexValueType::Mat2I, vek::Mat2<i32>}
    impl_static_vertex_factory_vertices! {vertices_mat3i, VertexValueType::Mat3I, vek::Mat3<i32>}
    impl_static_vertex_factory_vertices! {vertices_mat4i, VertexValueType::Mat4I, vek::Mat4<i32>}

    pub fn vertices<T>(&mut self, data: &[T], start: Option<usize>) -> Result<(), MeshError>
    where
        T: VertexType,
    {
        let layout = T::vertex_layout()?;
        if self.layout != layout {
            return Err(MeshError::LayoutsMismatch(
                Box::new(layout),
                Box::new(self.layout.to_owned()),
            ));
        }
        if layout.is_compact() {
            self.buffers[0] = unsafe { data.align_to::<u8>().1.to_owned() };
            return Ok(());
        }
        for buffer in layout.buffers {
            for (attribute, _) in buffer.attributes {
                if T::has_attribute(&attribute.id) {
                    match attribute.value_type {
                        VertexValueType::Scalar => {
                            let data = data
                                .iter()
                                .map(|v| v.scalar(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_scalar(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Vec2F => {
                            let data = data
                                .iter()
                                .map(|v| v.vec2f(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_vec2f(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Vec3F => {
                            let data = data
                                .iter()
                                .map(|v| v.vec3f(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_vec3f(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Vec4F => {
                            let data = data
                                .iter()
                                .map(|v| v.vec4f(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_vec4f(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Mat2F => {
                            let data = data
                                .iter()
                                .map(|v| v.mat2f(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_mat2f(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Mat3F => {
                            let data = data
                                .iter()
                                .map(|v| v.mat3f(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_mat3f(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Mat4F => {
                            let data = data
                                .iter()
                                .map(|v| v.mat4f(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_mat4f(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Integer => {
                            let data = data
                                .iter()
                                .map(|v| v.integer(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_integer(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Vec2I => {
                            let data = data
                                .iter()
                                .map(|v| v.vec2i(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_vec2i(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Vec3I => {
                            let data = data
                                .iter()
                                .map(|v| v.vec3i(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_vec3i(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Vec4I => {
                            let data = data
                                .iter()
                                .map(|v| v.vec4i(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_vec4i(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Mat2I => {
                            let data = data
                                .iter()
                                .map(|v| v.mat2i(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_mat2i(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Mat3I => {
                            let data = data
                                .iter()
                                .map(|v| v.mat3i(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_mat3i(&attribute.id, &data, start)?;
                        }
                        VertexValueType::Mat4I => {
                            let data = data
                                .iter()
                                .map(|v| v.mat4i(&attribute.id).unwrap())
                                .collect::<Vec<_>>();
                            self.vertices_mat4i(&attribute.id, &data, start)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn triangles(
        &mut self,
        data: &[(u32, u32, u32)],
        start: Option<usize>,
    ) -> Result<(), MeshError> {
        if self.draw_mode != MeshDrawMode::Triangles {
            return Err(MeshError::IncompatibleDrawMode(
                MeshDrawMode::Triangles,
                self.draw_mode,
            ));
        }
        let iter = data.iter().zip(
            self.indices[start.unwrap_or(0)..].chunks_exact_mut(self.draw_mode.indices_count()),
        );
        for (source, target) in iter {
            target[0] = source.0;
            target[1] = source.1;
            target[2] = source.2;
        }
        Ok(())
    }

    pub fn lines(&mut self, data: &[(u32, u32)], start: Option<usize>) -> Result<(), MeshError> {
        if self.draw_mode != MeshDrawMode::Lines {
            return Err(MeshError::IncompatibleDrawMode(
                MeshDrawMode::Lines,
                self.draw_mode,
            ));
        }
        let iter = data.iter().zip(
            self.indices[start.unwrap_or(0)..].chunks_exact_mut(self.draw_mode.indices_count()),
        );
        for (source, target) in iter {
            target[0] = source.0;
            target[1] = source.1;
        }
        Ok(())
    }

    pub fn points(&mut self, data: &[u32], start: Option<usize>) -> Result<(), MeshError> {
        if self.draw_mode != MeshDrawMode::Points {
            return Err(MeshError::IncompatibleDrawMode(
                MeshDrawMode::Points,
                self.draw_mode,
            ));
        }
        let iter = data
            .iter()
            .zip(self.indices[start.unwrap_or(0)..].iter_mut());
        for (source, target) in iter {
            *target = *source;
        }
        Ok(())
    }

    /// # Safety
    /// Writing to raw bytes may cause underlying vertices having wrong data layout.
    pub unsafe fn access_raw_bytes(&mut self, buffer: usize) -> Option<&mut [u8]> {
        self.buffers
            .get_mut(buffer)
            .map(|bytes| bytes.as_mut_slice())
    }

    /// # Safety
    /// Writing to raw vertices of wrong type may cause underlying vertices having wrong data layout.
    pub unsafe fn access_raw_vertices<T>(&mut self, buffer: usize) -> Option<&mut [T]>
    where
        T: VertexType,
    {
        self.buffers
            .get_mut(buffer)
            .map(|bytes| bytes.as_mut_slice().align_to_mut::<T>().1)
    }

    /// # Safety
    /// Writing to raw indices may cause indices pointing to wrong or even non-existing vertices.
    pub unsafe fn access_raw_indices(&mut self) -> &mut [u32] {
        &mut self.indices
    }

    pub fn write_into(&self, mesh: &mut Mesh) -> Result<(), MeshError> {
        if mesh.layout != self.layout {
            return Err(MeshError::LayoutsMismatch(
                Box::new(self.layout.to_owned()),
                Box::new(mesh.layout.to_owned()),
            ));
        }
        for (index, data) in self.buffers.iter().enumerate() {
            mesh.set_vertex_data(index, data.to_owned())?;
        }
        mesh.set_index_data(self.indices.to_owned(), self.draw_mode);
        Ok(())
    }

    pub fn consume_write_into(self, mesh: &mut Mesh) -> Result<(), MeshError> {
        if mesh.layout != self.layout {
            return Err(MeshError::LayoutsMismatch(
                Box::new(self.layout),
                Box::new(mesh.layout.to_owned()),
            ));
        }
        for (index, data) in self.buffers.into_iter().enumerate() {
            mesh.set_vertex_data(index, data)?;
        }
        mesh.set_index_data(self.indices, self.draw_mode);
        Ok(())
    }

    /// (vertex layout, vertex buffers, index buffer, vertex count, draw mode)
    pub fn into_inner(self) -> (VertexLayout, Vec<Vec<u8>>, Vec<u32>, usize, MeshDrawMode) {
        (
            self.layout,
            self.buffers,
            self.indices,
            self.vertices,
            self.draw_mode,
        )
    }

    /// (buffer index, offset, stride)?
    fn find_buffer_offset_stride(
        &self,
        name: &str,
        value_type: VertexValueType,
    ) -> Option<(usize, usize, usize)> {
        let is_integer = value_type.is_integer();
        let channels = value_type.channels();
        self.layout
            .buffers
            .iter()
            .enumerate()
            .find_map(|(index, buffer)| {
                buffer.vertex_attribs().find_map(|(id, chunk)| {
                    if id == name
                        && chunk.is_integer() == is_integer
                        && chunk.channels() == channels
                    {
                        Some((index, chunk.offset(), chunk.stride()))
                    } else {
                        None
                    }
                })
            })
    }
}

pub trait VertexType {
    fn vertex_layout() -> Result<VertexLayout, MeshError>;
    fn has_attribute(name: &str) -> bool;
    fn scalar(&self, name: &str) -> Option<f32>;
    fn vec2f(&self, name: &str) -> Option<vek::Vec2<f32>>;
    fn vec3f(&self, name: &str) -> Option<vek::Vec3<f32>>;
    fn vec4f(&self, name: &str) -> Option<vek::Vec4<f32>>;
    fn mat2f(&self, name: &str) -> Option<vek::Mat2<f32>>;
    fn mat3f(&self, name: &str) -> Option<vek::Mat3<f32>>;
    fn mat4f(&self, name: &str) -> Option<vek::Mat4<f32>>;
    fn integer(&self, name: &str) -> Option<i32>;
    fn vec2i(&self, name: &str) -> Option<vek::Vec2<i32>>;
    fn vec3i(&self, name: &str) -> Option<vek::Vec3<i32>>;
    fn vec4i(&self, name: &str) -> Option<vek::Vec4<i32>>;
    fn mat2i(&self, name: &str) -> Option<vek::Mat2<i32>>;
    fn mat3i(&self, name: &str) -> Option<vek::Mat3<i32>>;
    fn mat4i(&self, name: &str) -> Option<vek::Mat4<i32>>;
    fn transform(&mut self, _matrix: &vek::Mat4<f32>) {}
}

#[macro_export]
macro_rules! vertex_type {
    (@type float) => (f32);
    (@type vec2) => ($crate::math::vek::Vec2<f32>);
    (@type vec3) => ($crate::math::vek::Vec3<f32>);
    (@type vec4) => ($crate::math::vek::Vec4<f32>);
    (@type mat2) => ($crate::math::vek::Mat2<f32>);
    (@type mat3) => ($crate::math::vek::Mat3<f32>);
    (@type mat4) => ($crate::math::vek::Mat4<f32>);
    (@type int) => (i32);
    (@type ivec2) => ($crate::math::vek::Vec2<i32>);
    (@type ivec3) => ($crate::math::vek::Vec3<i32>);
    (@type ivec4) => ($crate::math::vek::Vec4<i32>);
    (@type imat2) => ($crate::math::vek::Mat2<i32>);
    (@type imat3) => ($crate::math::vek::Mat3<i32>);
    (@type imat4) => ($crate::math::vek::Mat4<i32>);
    (@value float) => ($crate::mesh::VertexValueType::Scalar);
    (@value vec2) => ($crate::mesh::VertexValueType::Vec2F);
    (@value vec3) => ($crate::mesh::VertexValueType::Vec3F);
    (@value vec4) => ($crate::mesh::VertexValueType::Vec4F);
    (@value mat2) => ($crate::mesh::VertexValueType::Mat2F);
    (@value mat3) => ($crate::mesh::VertexValueType::Mat3F);
    (@value mat4) => ($crate::mesh::VertexValueType::Mat4F);
    (@value int) => ($crate::mesh::VertexValueType::Integer);
    (@value ivec2) => ($crate::mesh::VertexValueType::Vec2I);
    (@value ivec3) => ($crate::mesh::VertexValueType::Vec3I);
    (@value ivec4) => ($crate::mesh::VertexValueType::Vec4I);
    (@value imat2) => ($crate::mesh::VertexValueType::Mat2I);
    (@value imat3) => ($crate::mesh::VertexValueType::Mat3I);
    (@value imat4) => ($crate::mesh::VertexValueType::Mat4I);
    (@return $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        if $name == stringify!($field_mapping) {
            return Some($this.$field_name);
        }
    };
    (@data(float, float), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(float, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(vec2, vec2), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(vec2, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(vec3, vec3), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(vec3, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(vec4, vec4), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(vec4, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(mat2, mat2), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(mat2, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(mat3, mat3), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(mat3, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(mat4, mat4), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(mat4, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(int, int), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(int, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(ivec2, ivec2), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(ivec2, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(ivec3, ivec3), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(ivec3, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(ivec4, ivec4), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(ivec4, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(imat2, imat2), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(imat2, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(imat3, imat3), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $name, $this);
    };
    (@data(imat3, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (@data(imat4, imat4), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => {
        $crate::vertex_type!(@return $field_name, $field_mapping, $field_mapping, $name, $this);
    };
    (@data(imat4, $ignore:ident), $field_name:ident, $field_mapping:ident, $name:expr, $this:expr) => ();
    (
        $( #[ $meta:meta ] )*
        $( @ middlewares ( $( $middleware:ident ),* ) )?
        $( @ tags ( $( $tag:ident ),* ) )?
        $visibility:vis struct $name:ident {
            $(
                $( #[ $field_meta:meta ] )*
                $field_visibility:vis $field_name:ident : $field_type:ident
                = $field_mapping:ident ( $field_buffer:literal $(, $field_flag:ident)* ),
            )+
        }
    ) => {
        $( #[ $meta ] )*
        #[repr(C)]
        $visibility struct $name {
            $(
                $( #[ $field_meta ] )*
                $field_visibility $field_name : $crate::vertex_type!(@type $field_type) ,
            )+
        }

        #[allow(unused_variables)]
        impl $crate::mesh::vertex_factory::VertexType for $name {
            fn vertex_layout() -> Result<$crate::mesh::VertexLayout, $crate::mesh::MeshError> {
                #[allow(unused_mut)]
                let mut buffers = std::collections::BTreeMap::<
                    usize,
                    $crate::mesh::VertexBufferLayout,
                >::new();
                #[allow(unused_mut)]
                let mut bounds = Option::<&str>::None;
                $(
                    let index = $field_buffer as usize;
                    #[allow(unused_mut)]
                    let mut normalized = false;
                    $(
                        match stringify!($field_flag) {
                            "normalized" => normalized = true,
                            "bounds" => bounds = Some(stringify!($field_name)),
                            _ => {},
                        }
                    )*
                    if let Some(buffer) = buffers.get_mut(&index) {
                        buffer.add($crate::mesh::VertexAttribute {
                            id: stringify!($field_mapping).to_owned(),
                            count: 1,
                            value_type: $crate::vertex_type!(@value $field_type),
                            normalized,
                        })?;
                    } else {
                        buffers.insert(
                            index,
                            $crate::mesh::VertexBufferLayout::default()
                                .with($crate::mesh::VertexAttribute {
                                    id: stringify!($field_mapping).to_owned(),
                                    count: 1,
                                    value_type: $crate::vertex_type!(@value $field_type),
                                    normalized,
                                })?,
                        );
                    }
                )+
                let mut result = $crate::mesh::VertexLayout::default();
                for (_, buffer) in buffers {
                    result = result.with_buffer(buffer)?;
                }
                if let Some(bounds) = bounds {
                    result = result.with_bounds(Some(bounds.to_owned()));
                }
                result = result.with_middlewares(vec![ $( $( stringify!($middleware).to_owned() ),* )? ]);
                Ok(result)
            }

            fn has_attribute(name: &str) -> bool {
                $(
                    if name == stringify!($field_mapping) {
                        return true;
                    }
                )+
                false
            }

            fn scalar(&self, name: &str) -> Option<f32> {
                $(
                    $crate::vertex_type!(
                        @data(float, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn vec2f(&self, name: &str) -> Option<vek::Vec2<f32>> {
                $(
                    $crate::vertex_type!(
                        @data(vec2, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn vec3f(&self, name: &str) -> Option<vek::Vec3<f32>> {
                $(
                    $crate::vertex_type!(
                        @data(vec3, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn vec4f(&self, name: &str) -> Option<vek::Vec4<f32>> {
                $(
                    $crate::vertex_type!(
                        @data(vec4, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn mat2f(&self, name: &str) -> Option<vek::Mat2<f32>> {
                $(
                    $crate::vertex_type!(
                        @data(mat2, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn mat3f(&self, name: &str) -> Option<vek::Mat3<f32>> {
                $(
                    $crate::vertex_type!(
                        @data(mat3, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn mat4f(&self, name: &str) -> Option<vek::Mat4<f32>> {
                $(
                    $crate::vertex_type!(
                        @data(mat4, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn integer(&self, name: &str) -> Option<i32> {
                $(
                    $crate::vertex_type!(
                        @data(int, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn vec2i(&self, name: &str) -> Option<vek::Vec2<i32>> {
                $(
                    $crate::vertex_type!(
                        @data(ivec2, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn vec3i(&self, name: &str) -> Option<vek::Vec3<i32>> {
                $(
                    $crate::vertex_type!(
                        @data(ivec3, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn vec4i(&self, name: &str) -> Option<vek::Vec4<i32>> {
                $(
                    $crate::vertex_type!(
                        @data(ivec4, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn mat2i(&self, name: &str) -> Option<vek::Mat2<i32>> {
                $(
                    $crate::vertex_type!(
                        @data(imat2, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn mat3i(&self, name: &str) -> Option<vek::Mat3<i32>> {
                $(
                    $crate::vertex_type!(
                        @data(imat3, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }

            fn mat4i(&self, name: &str) -> Option<vek::Mat4<i32>> {
                $(
                    $crate::vertex_type!(
                        @data(imat4, $field_type),
                        $field_name,
                        $field_mapping,
                        name,
                        self
                    );
                )+
                None
            }
        }

        $( $( impl $tag for $name {} )* )?
    };
}

#[macro_export]
macro_rules! compound_vertex_type {
    (
        $( #[ $meta:meta ] )*
        $( @ tags ( $( $tag:ident ),* ) )?
        $visibility:vis struct $name:ident {
            $(
                $( #[ $field_meta:meta ] )*
                $field_visibility:vis $field_name:ident : $field_type:ty,
            )+
        }
    ) => {
        $( #[ $meta ] )*
        $visibility struct $name {
            $(
                $( #[ $field_meta ] )*
                $field_visibility $field_name : $field_type,
            )+
        }

        impl $crate::mesh::vertex_factory::VertexType for $name {
            fn vertex_layout() -> Result<$crate::mesh::VertexLayout, $crate::mesh::MeshError> {
                let mut result = $crate::mesh::VertexLayout::default();
                $(
                    result = result.with(<$field_type>::vertex_layout()?)?;
                )+
                Ok(result)
            }

            fn has_attribute(name: &str) -> bool {
                $(
                    if <$field_type>::has_attribute(name) {
                        return true;
                    }
                )+
                false
            }

            fn scalar(&self, name: &str) -> Option<f32> {
                $(
                    if let Some(result) = self.$field_name.scalar(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn vec2f(&self, name: &str) -> Option<vek::Vec2<f32>> {
                $(
                    if let Some(result) = self.$field_name.vec2f(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn vec3f(&self, name: &str) -> Option<vek::Vec3<f32>> {
                $(
                    if let Some(result) = self.$field_name.vec3f(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn vec4f(&self, name: &str) -> Option<vek::Vec4<f32>> {
                $(
                    if let Some(result) = self.$field_name.vec4f(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn mat2f(&self, name: &str) -> Option<vek::Mat2<f32>> {
                $(
                    if let Some(result) = self.$field_name.mat2f(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn mat3f(&self, name: &str) -> Option<vek::Mat3<f32>> {
                $(
                    if let Some(result) = self.$field_name.mat3f(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn mat4f(&self, name: &str) -> Option<vek::Mat4<f32>> {
                $(
                    if let Some(result) = self.$field_name.mat4f(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn integer(&self, name: &str) -> Option<i32> {
                $(
                    if let Some(result) = self.$field_name.integer(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn vec2i(&self, name: &str) -> Option<vek::Vec2<i32>> {
                $(
                    if let Some(result) = self.$field_name.vec2i(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn vec3i(&self, name: &str) -> Option<vek::Vec3<i32>> {
                $(
                    if let Some(result) = self.$field_name.vec3i(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn vec4i(&self, name: &str) -> Option<vek::Vec4<i32>> {
                $(
                    if let Some(result) = self.$field_name.vec4i(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn mat2i(&self, name: &str) -> Option<vek::Mat2<i32>> {
                $(
                    if let Some(result) = self.$field_name.mat2i(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn mat3i(&self, name: &str) -> Option<vek::Mat3<i32>> {
                $(
                    if let Some(result) = self.$field_name.mat3i(name) {
                        return Some(result);
                    }
                )+
                None
            }

            fn mat4i(&self, name: &str) -> Option<vek::Mat4<i32>> {
                $(
                    if let Some(result) = self.$field_name.mat4i(name) {
                        return Some(result);
                    }
                )+
                None
            }
        }

        $( $( impl $tag for $name {} )* )?
    }
}

impl VertexType for () {
    fn vertex_layout() -> Result<VertexLayout, MeshError> {
        Ok(VertexLayout::default())
    }

    fn has_attribute(_name: &str) -> bool {
        false
    }

    fn scalar(&self, _name: &str) -> Option<f32> {
        None
    }

    fn vec2f(&self, _name: &str) -> Option<vek::Vec2<f32>> {
        None
    }

    fn vec3f(&self, _name: &str) -> Option<vek::Vec3<f32>> {
        None
    }

    fn vec4f(&self, _name: &str) -> Option<vek::Vec4<f32>> {
        None
    }

    fn mat2f(&self, _name: &str) -> Option<vek::Mat2<f32>> {
        None
    }

    fn mat3f(&self, _name: &str) -> Option<vek::Mat3<f32>> {
        None
    }

    fn mat4f(&self, _name: &str) -> Option<vek::Mat4<f32>> {
        None
    }

    fn integer(&self, _name: &str) -> Option<i32> {
        None
    }

    fn vec2i(&self, _name: &str) -> Option<vek::Vec2<i32>> {
        None
    }

    fn vec3i(&self, _name: &str) -> Option<vek::Vec3<i32>> {
        None
    }

    fn vec4i(&self, _name: &str) -> Option<vek::Vec4<i32>> {
        None
    }

    fn mat2i(&self, _name: &str) -> Option<vek::Mat2<i32>> {
        None
    }

    fn mat3i(&self, _name: &str) -> Option<vek::Mat3<i32>> {
        None
    }

    fn mat4i(&self, _name: &str) -> Option<vek::Mat4<i32>> {
        None
    }
}
