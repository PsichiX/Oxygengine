use crate::Scalar;
use petgraph::{algo::astar, graph::NodeIndex, visit::EdgeRef, Graph, Undirected};
use spade::{rtree::RTree, BoundingRect, PointN, SpatialObject};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::{Add, Div, Mul, Sub},
    result::Result as StdResult,
};

#[derive(Debug, Clone)]
pub enum Error {
    /// (triangle index, local vertice index, global vertice index)
    TriangleVerticeIndexOutOfBounds(u32, u8, u32),
}

pub type Result<T> = StdResult<T, Error>;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct NavVec3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl NavVec3 {
    #[inline]
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn sqr_magnitude(self) -> Scalar {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    #[inline]
    pub fn magnitude(self) -> Scalar {
        self.sqr_magnitude().sqrt()
    }

    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: (self.y * other.z) - (self.z * other.y),
            y: (self.z * other.x) - (self.x * other.z),
            z: (self.x * other.y) - (self.y * other.x),
        }
    }

    #[inline]
    pub fn dot(self, other: Self) -> Scalar {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = self.magnitude();
        Self::new(self.x / len, self.y / len, self.z / len)
    }

    #[inline]
    pub fn project(self, from: Self, to: Self) -> Scalar {
        let diff = to - from;
        (self - from).dot(diff) / diff.sqr_magnitude()
    }

    #[inline]
    pub fn unproject(from: Self, to: Self, t: Scalar) -> Self {
        let diff = to - from;
        from + Self::new(diff.x * t, diff.y * t, diff.z * t)
    }

    #[inline]
    pub fn is_above_plane(self, origin: Self, normal: Self) -> bool {
        normal.dot(self - origin) > 0.0
    }

    pub fn project_on_plane(self, origin: Self, normal: Self) -> Self {
        let v = self - origin;
        let n = normal.normalize();
        let dot = v.dot(n);
        let d = NavVec3::new(normal.x * dot, normal.y * dot, normal.z * dot);
        self - d
    }

    pub fn lines_intersects(
        a_from: NavVec3,
        a_to: NavVec3,
        b_from: NavVec3,
        b_to: NavVec3,
        normal: NavVec3,
    ) -> bool {
        let a = a_to - a_from;
        let b = b_to - b_from;
        let an = a.cross(normal);
        let bn = b.cross(normal);
        let afs = Self::side(bn.dot(a_from - b_from));
        let ats = Self::side(bn.dot(a_to - b_from));
        let bfs = Self::side(an.dot(b_from - a_from));
        let bts = Self::side(an.dot(b_to - a_from));
        Self::different_sides(afs, ats) && Self::different_sides(bfs, bts)
    }

    fn side(v: Scalar) -> i8 {
        if v.abs() < 1e-6 {
            0
        } else {
            v.signum() as i8
        }
    }

    fn different_sides(a: i8, b: i8) -> bool {
        (a == 0 && b == 0) || (a < 0 && b >= 0) || (a > 0 && b <= 0)
    }
}

impl From<(Scalar, Scalar, Scalar)> for NavVec3 {
    fn from(value: (Scalar, Scalar, Scalar)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

impl Add for NavVec3 {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add<Scalar> for NavVec3 {
    type Output = Self;

    #[inline]
    fn add(self, other: Scalar) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
            z: self.z + other,
        }
    }
}

impl Sub for NavVec3 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Sub<Scalar> for NavVec3 {
    type Output = Self;

    #[inline]
    fn sub(self, other: Scalar) -> Self {
        Self {
            x: self.x - other,
            y: self.y - other,
            z: self.z - other,
        }
    }
}

impl Mul for NavVec3 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Mul<Scalar> for NavVec3 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Scalar) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}

impl Div for NavVec3 {
    type Output = Self;

    #[inline]
    fn div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl Div<Scalar> for NavVec3 {
    type Output = Self;

    #[inline]
    fn div(self, other: Scalar) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}

impl PointN for NavVec3 {
    type Scalar = Scalar;

    fn dimensions() -> usize {
        3
    }

    fn nth(&self, index: usize) -> &Self::Scalar {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => unreachable!(),
        }
    }

    fn from_value(value: Self::Scalar) -> Self {
        NavVec3::new(value, value, value)
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct NavTriangle {
    pub first: u32,
    pub second: u32,
    pub third: u32,
}

impl From<(u32, u32, u32)> for NavTriangle {
    fn from(value: (u32, u32, u32)) -> Self {
        Self {
            first: value.0,
            second: value.1,
            third: value.2,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct NavArea {
    pub triangle: u32,
    pub size: Scalar,
    pub cost: Scalar,
    pub inv_cost: Scalar,
    pub center: NavVec3,
    pub radius: Scalar,
    pub radius_sqr: Scalar,
}

impl NavArea {
    #[inline]
    pub fn calculate_area(a: NavVec3, b: NavVec3, c: NavVec3) -> Scalar {
        let ab = b - a;
        let ac = c - a;
        ab.cross(ac).magnitude() * 0.5
    }

    #[inline]
    pub fn calculate_center(a: NavVec3, b: NavVec3, c: NavVec3) -> NavVec3 {
        let v = a + b + c;
        NavVec3::new(v.x / 3.0, v.y / 3.0, v.z / 3.0)
    }
}

#[derive(Debug, Default, Copy, Clone, Eq)]
struct NavConnection(pub u32, pub u32);

impl Hash for NavConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let first = self.0.min(self.1);
        let second = self.0.max(self.1);
        first.hash(state);
        second.hash(state);
    }
}

impl PartialEq for NavConnection {
    fn eq(&self, other: &Self) -> bool {
        let first = self.0.min(self.1);
        let second = self.0.max(self.1);
        let ofirst = other.0.min(other.1);
        let osecond = other.0.max(other.1);
        first == ofirst && second == osecond
    }
}

#[derive(Debug, Clone)]
struct NavSpatialObject {
    pub index: usize,
    pub a: NavVec3,
    pub b: NavVec3,
    pub c: NavVec3,
    ab: NavVec3,
    bc: NavVec3,
    ca: NavVec3,
    normal: NavVec3,
    dab: NavVec3,
    dbc: NavVec3,
    dca: NavVec3,
}

impl NavSpatialObject {
    pub fn new(index: usize, a: NavVec3, b: NavVec3, c: NavVec3) -> Self {
        let ab = b - a;
        let bc = c - b;
        let ca = a - c;
        let normal = (a - b).cross(a - c).normalize();
        let dab = normal.cross(ab);
        let dbc = normal.cross(bc);
        let dca = normal.cross(ca);
        Self {
            index,
            a,
            b,
            c,
            ab,
            bc,
            ca,
            normal,
            dab,
            dbc,
            dca,
        }
    }

    #[inline]
    pub fn normal(&self) -> NavVec3 {
        self.normal
    }

    pub fn closest_point(&self, point: NavVec3) -> NavVec3 {
        let pab = point.project(self.a, self.b);
        let pbc = point.project(self.b, self.c);
        let pca = point.project(self.c, self.a);
        if pca > 1.0 && pab < 0.0 {
            return self.a;
        } else if pab > 1.0 && pbc < 0.0 {
            return self.b;
        } else if pbc > 1.0 && pca < 0.0 {
            return self.c;
        } else if pab >= 0.0 && pab <= 1.0 && !point.is_above_plane(self.a, self.dab) {
            return NavVec3::unproject(self.a, self.b, pab);
        } else if pbc >= 0.0 && pbc <= 1.0 && !point.is_above_plane(self.b, self.dbc) {
            return NavVec3::unproject(self.b, self.c, pbc);
        } else if pca >= 0.0 && pca <= 1.0 && !point.is_above_plane(self.c, self.dca) {
            return NavVec3::unproject(self.c, self.a, pca);
        }
        point.project_on_plane(self.a, self.normal)
    }
}

impl SpatialObject for NavSpatialObject {
    type Point = NavVec3;

    fn mbr(&self) -> BoundingRect<Self::Point> {
        let min = NavVec3::new(
            self.a.x.min(self.b.x).min(self.c.x),
            self.a.y.min(self.b.y).min(self.c.y),
            self.a.z.min(self.b.z).min(self.c.z),
        );
        let max = NavVec3::new(
            self.a.x.max(self.b.x).max(self.c.x),
            self.a.y.max(self.b.y).max(self.c.y),
            self.a.z.max(self.b.z).max(self.c.z),
        );
        BoundingRect::from_corners(&min, &max)
    }

    fn distance2(&self, point: &Self::Point) -> Scalar {
        (*point - self.closest_point(*point)).sqr_magnitude()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NavQuery {
    Accuracy,
    Closest,
    ClosestFirst,
}

#[derive(Debug, Clone, Copy)]
pub enum NavPathMode {
    Accuracy,
    MidPoints,
    CenterPoints,
}

#[derive(Debug, Default, Clone)]
pub struct NavMesh {
    vertices: Vec<NavVec3>,
    triangles: Vec<NavTriangle>,
    areas: Vec<NavArea>,
    // {triangle connection: (distance sqr, vertex connection)}
    connections: HashMap<NavConnection, (Scalar, NavConnection)>,
    graph: Graph<(), Scalar, Undirected>,
    nodes: Vec<NodeIndex>,
    nodes_map: HashMap<NodeIndex, usize>,
    rtree: RTree<NavSpatialObject>,
    spatials: Vec<NavSpatialObject>,
    // {triangle index: [(from, to)]}
    hard_edges: HashMap<usize, Vec<(NavVec3, NavVec3)>>,
}

impl NavMesh {
    pub fn new(vertices: Vec<NavVec3>, triangles: Vec<NavTriangle>) -> Result<Self> {
        let areas = triangles
            .iter()
            .enumerate()
            .map(|(i, triangle)| {
                if triangle.first >= vertices.len() as u32 {
                    return Err(Error::TriangleVerticeIndexOutOfBounds(
                        i as u32,
                        0,
                        triangle.first,
                    ));
                }
                if triangle.second >= vertices.len() as u32 {
                    return Err(Error::TriangleVerticeIndexOutOfBounds(
                        i as u32,
                        1,
                        triangle.second,
                    ));
                }
                if triangle.third >= vertices.len() as u32 {
                    return Err(Error::TriangleVerticeIndexOutOfBounds(
                        i as u32,
                        2,
                        triangle.third,
                    ));
                }
                let first = vertices[triangle.first as usize];
                let second = vertices[triangle.second as usize];
                let third = vertices[triangle.third as usize];
                let center = NavArea::calculate_center(first, second, third);
                let radius = (first - center)
                    .magnitude()
                    .max((second - center).magnitude())
                    .max((third - center).magnitude());
                Ok(NavArea {
                    triangle: i as u32,
                    size: NavArea::calculate_area(first, second, third),
                    cost: 1.0,
                    inv_cost: 1.0,
                    center,
                    radius,
                    radius_sqr: radius * radius,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        // {edge: [triangle index]}
        let mut edges = HashMap::<NavConnection, Vec<usize>>::with_capacity(triangles.len() * 3);
        for (index, triangle) in triangles.iter().enumerate() {
            let edge_a = NavConnection(triangle.first, triangle.second);
            let edge_b = NavConnection(triangle.second, triangle.third);
            let edge_c = NavConnection(triangle.third, triangle.first);
            if let Some(tris) = edges.get_mut(&edge_a) {
                tris.push(index);
            } else {
                edges.insert(edge_a, vec![index]);
            }
            if let Some(tris) = edges.get_mut(&edge_b) {
                tris.push(index);
            } else {
                edges.insert(edge_b, vec![index]);
            }
            if let Some(tris) = edges.get_mut(&edge_c) {
                tris.push(index);
            } else {
                edges.insert(edge_c, vec![index]);
            }
        }

        let connections = edges
            .iter()
            .flat_map(|(verts, tris)| {
                let mut result = HashMap::with_capacity(tris.len() * tris.len());
                for a in tris {
                    for b in tris {
                        if a != b {
                            result.insert(NavConnection(*a as u32, *b as u32), *verts);
                        }
                    }
                }
                result
            })
            .collect::<HashMap<_, _>>()
            .into_iter()
            .map(|(tri_conn, vert_conn)| {
                let a = areas[tri_conn.0 as usize].center;
                let b = areas[tri_conn.1 as usize].center;
                let weight = (b - a).sqr_magnitude();
                (tri_conn, (weight, vert_conn))
            })
            .collect::<HashMap<_, _>>();

        let mut graph = Graph::<(), Scalar, Undirected>::new_undirected();
        let nodes = (0..triangles.len())
            .map(|_| graph.add_node(()))
            .collect::<Vec<_>>();
        graph.extend_with_edges(
            connections
                .iter()
                .map(|(conn, (w, _))| (nodes[conn.0 as usize], nodes[conn.1 as usize], w)),
        );
        let nodes_map = nodes.iter().enumerate().map(|(i, n)| (*n, i)).collect();

        let spatials = triangles
            .iter()
            .enumerate()
            .map(|(index, triangle)| {
                NavSpatialObject::new(
                    index,
                    vertices[triangle.first as usize],
                    vertices[triangle.second as usize],
                    vertices[triangle.third as usize],
                )
            })
            .collect::<Vec<_>>();

        let mut rtree = RTree::new();
        for spatial in &spatials {
            rtree.insert(spatial.clone());
        }

        let hard_edges = triangles
            .iter()
            .enumerate()
            .filter_map(|(index, triangle)| {
                let edge_a = NavConnection(triangle.first, triangle.second);
                let edge_b = NavConnection(triangle.second, triangle.third);
                let edge_c = NavConnection(triangle.third, triangle.first);
                let mut planes = vec![];
                if edges[&edge_a].len() < 2 {
                    planes.push((
                        vertices[triangle.first as usize],
                        vertices[triangle.second as usize],
                    ));
                }
                if edges[&edge_b].len() < 2 {
                    planes.push((
                        vertices[triangle.second as usize],
                        vertices[triangle.third as usize],
                    ));
                }
                if edges[&edge_c].len() < 2 {
                    planes.push((
                        vertices[triangle.third as usize],
                        vertices[triangle.first as usize],
                    ));
                }
                if planes.is_empty() {
                    None
                } else {
                    Some((index, planes))
                }
            })
            .collect::<HashMap<_, _>>();

        Ok(Self {
            vertices,
            triangles,
            areas,
            connections,
            graph,
            nodes,
            nodes_map,
            rtree,
            spatials,
            hard_edges,
        })
    }

    #[inline]
    pub fn vertices(&self) -> &[NavVec3] {
        &self.vertices
    }

    #[inline]
    pub fn triangles(&self) -> &[NavTriangle] {
        &self.triangles
    }

    #[inline]
    pub fn areas(&self) -> &[NavArea] {
        &self.areas
    }

    #[inline]
    pub fn set_area_cost(&mut self, index: usize, cost: Scalar) -> Scalar {
        let area = &mut self.areas[index];
        let old = area.cost;
        let cost = cost.max(0.0);
        area.cost = cost;
        area.inv_cost = if cost.abs() == 0.0 { 0.0 } else { 1.0 / cost };
        old
    }

    pub fn find_path(
        &self,
        from: NavVec3,
        to: NavVec3,
        query: NavQuery,
        mode: NavPathMode,
    ) -> Vec<NavVec3> {
        let start = if let Some(start) = self.find_closest_triangle(from, query) {
            start
        } else {
            return vec![];
        };
        let end = if let Some(end) = self.find_closest_triangle(to, query) {
            end
        } else {
            return vec![];
        };
        if let Some((triangles, _)) = self.find_path_triangles(start, end) {
            if triangles.len() < 2 {
                return vec![from, to];
            }
            match mode {
                NavPathMode::Accuracy => {
                    // let count = triangles.len();
                    // let spatials_iter = triangles.iter().enumerate().filter_map(|(i, triangle)| {
                    //     if i > 0 && i < count - 1 {
                    //         Some((self.spatial[*triangle].center, *triangle))
                    //     } else {
                    //         None
                    //     }
                    // });
                    let mut points = Vec::with_capacity(triangles.len() + 1);
                    // let mut pt = from;
                    points.push(from);
                    // for (point, triangle) in points_iter {
                    //     let n = self.spatials[triangle].normal;
                    //     let c = self.hard_edges[&triangle]
                    //         .iter()
                    //         .any(|(from, to)| NavVec3::lines_intersects(pt, point, *from, *to, n));
                    //     if c {
                    //         points.push(point);
                    //         pt = point;
                    //     }
                    //     // points.push(point);
                    // }
                    points.push(to);
                    points
                }
                NavPathMode::MidPoints => {
                    let meta_iter = triangles
                        .windows(2)
                        .map(|pair| {
                            let NavConnection(a, b) =
                                self.connections[&NavConnection(pair[0] as u32, pair[1] as u32)].1;
                            let a = self.vertices[a as usize];
                            let b = self.vertices[b as usize];
                            (
                                (a + b) * 0.5,
                                self.hard_edges.get(&pair[0]),
                                self.spatials[pair[0]].normal(),
                            )
                        })
                        .chain(std::iter::once({
                            let triangle = triangles.last().unwrap();
                            (
                                to,
                                self.hard_edges.get(triangle),
                                self.spatials[*triangle].normal(),
                            )
                        }));
                    let mut points = Vec::with_capacity(triangles.len() + 1);
                    points.push(from);
                    let mut start = from;
                    let mut last = from;
                    for (center, hard_edges, normal) in meta_iter {
                        if let Some(hard_edges) = hard_edges {
                            if hard_edges.iter().any(|(a, b)| {
                                NavVec3::lines_intersects(start, center, *a, *b, normal)
                            }) {
                                points.push(last);
                                start = last;
                            }
                        }
                        last = center;
                    }
                    points.push(to);
                    points
                }
                NavPathMode::CenterPoints => {
                    let meta_iter = triangles.iter().map(|triangle| {
                        (
                            self.areas[*triangle].center,
                            self.hard_edges.get(triangle),
                            self.spatials[*triangle].normal(),
                        )
                    });
                    let mut points = Vec::with_capacity(triangles.len() + 2);
                    points.push(from);
                    let mut start = from;
                    let mut last = from;
                    for (center, hard_edges, normal) in meta_iter {
                        if let Some(hard_edges) = hard_edges {
                            if hard_edges.iter().any(|(a, b)| {
                                NavVec3::lines_intersects(start, center, *a, *b, normal)
                            }) {
                                points.push(last);
                                start = last;
                            }
                        }
                        last = center;
                    }
                    points.push(to);
                    points
                }
            }
        } else {
            vec![]
        }
    }

    #[inline]
    pub fn find_path_triangles(&self, from: usize, to: usize) -> Option<(Vec<usize>, Scalar)> {
        let to = self.nodes[to];
        astar(
            &self.graph,
            self.nodes[from],
            |n| n == to,
            |e| {
                let a = self.areas[self.nodes_map[&e.source()]].cost;
                let b = self.areas[self.nodes_map[&e.target()]].cost;
                *e.weight() * a * b
            },
            |_| 0.0,
        )
        .map(|(c, v)| (v.iter().map(|v| self.nodes_map[&v]).collect(), c))
    }

    pub fn find_closest_triangle(&self, point: NavVec3, query: NavQuery) -> Option<usize> {
        match query {
            NavQuery::Accuracy => self.rtree.nearest_neighbor(&point).map(|t| t.index),
            NavQuery::ClosestFirst => self.rtree.close_neighbor(&point).map(|t| t.index),
            NavQuery::Closest => self
                .rtree
                .nearest_neighbors(&point)
                .into_iter()
                .map(|o| (o.distance2(&point), o))
                .fold(None, |a: Option<(Scalar, &NavSpatialObject)>, i| {
                    if let Some(a) = a {
                        if i.0 < a.0 {
                            Some(i)
                        } else {
                            Some(a)
                        }
                    } else {
                        Some(i)
                    }
                })
                .map(|(_, t)| t.index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lines_intersection() {
        assert_eq!(
            false,
            NavVec3::lines_intersects(
                (-3.0, -1.0, 0.0).into(),
                (-1.0, 1.0, 0.0).into(),
                (-1.0, 0.0, 0.0).into(),
                (1.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            )
        );
        assert_eq!(
            true,
            NavVec3::lines_intersects(
                (-1.0, -1.0, 0.0).into(),
                (1.0, 1.0, 0.0).into(),
                (-1.0, 0.0, 0.0).into(),
                (1.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            )
        );
        assert_eq!(
            false,
            NavVec3::lines_intersects(
                (1.0, -1.0, 0.0).into(),
                (3.0, 1.0, 0.0).into(),
                (-1.0, 0.0, 0.0).into(),
                (1.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            )
        );
        assert_eq!(
            false,
            NavVec3::lines_intersects(
                (-2.0, -2.0, 0.0).into(),
                (2.0, 2.0, 0.0).into(),
                (0.0, -1.0, 0.0).into(),
                (2.0, -1.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            )
        );
        assert_eq!(
            false,
            NavVec3::lines_intersects(
                (-2.0, -2.0, 0.0).into(),
                (2.0, 2.0, 0.0).into(),
                (0.0, 0.0, 0.0).into(),
                (2.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            )
        );
    }

    #[test]
    fn test_spatials() {
        let vertices = vec![
            (1.0, 2.0, 0.0).into(),
            (2.0, 2.0, 0.0).into(),
            (2.0, 3.0, 0.0).into(),
            (1.0, 3.0, 0.0).into(),
        ];

        let s = NavSpatialObject::new(0, vertices[0], vertices[1], vertices[2]);
        assert_eq!(s.closest_point(vertices[0]), vertices[0]);
        assert_eq!(s.closest_point(vertices[1]), vertices[1]);
        assert_eq!(s.closest_point(vertices[2]), vertices[2]);
        assert_eq!(
            s.closest_point((1.75, 2.25, 0.0).into()),
            (1.75, 2.25, 0.0).into()
        );
        assert_eq!(
            s.closest_point((1.5, 1.0, 0.0).into()),
            (1.5, 2.0, 0.0).into()
        );
        assert_eq!(
            s.closest_point((3.0, 2.5, 0.0).into()),
            (2.0, 2.5, 0.0).into()
        );
        assert_eq!(
            s.closest_point((1.0, 3.0, 0.0).into()),
            (1.5, 2.5, 0.0).into()
        );

        let s = NavSpatialObject::new(0, vertices[2], vertices[3], vertices[0]);
        assert_eq!(s.closest_point(vertices[2]), vertices[2]);
        assert_eq!(s.closest_point(vertices[3]), vertices[3]);
        assert_eq!(s.closest_point(vertices[0]), vertices[0]);
        assert_eq!(
            s.closest_point((1.25, 2.75, 0.0).into()),
            (1.25, 2.75, 0.0).into()
        );
        assert_eq!(
            s.closest_point((2.0, 2.0, 0.0).into()),
            (1.5, 2.5, 0.0).into()
        );
        assert_eq!(
            s.closest_point((1.5, 4.0, 0.0).into()),
            (1.5, 3.0, 0.0).into()
        );
        assert_eq!(
            s.closest_point((0.0, 2.5, 0.0).into()),
            (1.0, 2.5, 0.0).into()
        );
    }

    #[test]
    fn test_general() {
        let vertices = vec![
            (0.0, 0.0, 0.0).into(), // 0
            (1.0, 0.0, 0.0).into(), // 1
            (2.0, 0.0, 0.0).into(), // 2
            (0.0, 1.0, 0.0).into(), // 3
            (1.0, 1.0, 0.0).into(), // 4
            (2.0, 1.0, 0.0).into(), // 5
            (0.0, 2.0, 0.0).into(), // 6
            (1.0, 2.0, 0.0).into(), // 7
        ];
        let triangles = vec![
            (0, 1, 4).into(), // 0
            (4, 3, 0).into(), // 1
            (1, 2, 5).into(), // 2
            (5, 4, 1).into(), // 3
            (3, 4, 7).into(), // 4
            (7, 6, 3).into(), // 5
        ];
        let mesh = NavMesh::new(vertices.clone(), triangles.clone()).unwrap();
        {
            let path = mesh.find_path_triangles(0, 0).unwrap().0;
            assert_eq!(path, vec![0]);
        }
        {
            let path = mesh.find_path_triangles(2, 5).unwrap().0;
            assert_eq!(path, vec![2, 3, 0, 1, 4, 5]);
        }
        {
            let path = mesh.find_path(
                (0.0, 0.0, 0.0).into(),
                (2.0, 0.0, 0.0).into(),
                NavQuery::Accuracy,
                NavPathMode::CenterPoints,
            );
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(0, 0, 0), (20, 0, 0),]
            );
            let path = mesh.find_path(
                (0.0, 0.0, 0.0).into(),
                (2.0, 0.0, 0.0).into(),
                NavQuery::Accuracy,
                NavPathMode::MidPoints,
            );
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(0, 0, 0), (15, 5, 0), (20, 0, 0),]
            );
        }
        {
            let path = mesh.find_path(
                (2.0, 0.0, 0.0).into(),
                (0.0, 2.0, 0.0).into(),
                NavQuery::Accuracy,
                NavPathMode::CenterPoints,
            );
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(20, 0, 0), (0, 20, 0),]
            );
            let path = mesh.find_path(
                (2.0, 0.0, 0.0).into(),
                (0.0, 2.0, 0.0).into(),
                NavQuery::Accuracy,
                NavPathMode::MidPoints,
            );
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(20, 0, 0), (5, 15, 0), (0, 20, 0),]
            );
        }
        {
            let path = mesh.find_path(
                (2.0, 1.0, 0.0).into(),
                (1.0, 2.0, 0.0).into(),
                NavQuery::Accuracy,
                NavPathMode::CenterPoints,
            );
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(20, 10, 0), (3, 6, 0), (10, 20, 0),]
            );
            let path = mesh.find_path(
                (2.0, 1.0, 0.0).into(),
                (1.0, 2.0, 0.0).into(),
                NavQuery::Accuracy,
                NavPathMode::MidPoints,
            );
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(20, 10, 0), (5, 10, 0), (10, 20, 0),]
            );
        }
        {
            let path = mesh.find_path(
                (0.5, 0.0, 0.0).into(),
                (0.5, 2.0, 0.0).into(),
                NavQuery::Accuracy,
                NavPathMode::CenterPoints,
            );
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(5, 0, 0), (5, 20, 0),]
            );
            let path = mesh.find_path(
                (0.5, 0.0, 0.0).into(),
                (0.5, 2.0, 0.0).into(),
                NavQuery::Accuracy,
                NavPathMode::MidPoints,
            );
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(5, 0, 0), (5, 15, 0), (5, 20, 0),]
            );
        }
    }
}
