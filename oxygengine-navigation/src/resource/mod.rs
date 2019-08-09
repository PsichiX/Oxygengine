use crate::Scalar;

mod nav_mesh;
mod nav_vec3;

pub use nav_mesh::*;
pub use nav_vec3::*;

pub(crate) const ZERO_TRESHOLD: Scalar = 1e-6;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_between_points() {
        assert_eq!(
            true,
            NavVec3::is_line_between_points(
                (0.0, -1.0, 0.0).into(),
                (0.0, 1.0, 0.0).into(),
                (-1.0, 0.0, 0.0).into(),
                (1.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            ),
        );
        assert_eq!(
            false,
            NavVec3::is_line_between_points(
                (-2.0, -1.0, 0.0).into(),
                (-2.0, 1.0, 0.0).into(),
                (-1.0, 0.0, 0.0).into(),
                (1.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            ),
        );
        assert_eq!(
            false,
            NavVec3::is_line_between_points(
                (2.0, -1.0, 0.0).into(),
                (2.0, 1.0, 0.0).into(),
                (-1.0, 0.0, 0.0).into(),
                (1.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            ),
        );
        assert_eq!(
            true,
            NavVec3::is_line_between_points(
                (-1.0, -1.0, 0.0).into(),
                (-1.0, 1.0, 0.0).into(),
                (-1.0, 0.0, 0.0).into(),
                (1.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            ),
        );
        assert_eq!(
            true,
            NavVec3::is_line_between_points(
                (1.0, -1.0, 0.0).into(),
                (1.0, 1.0, 0.0).into(),
                (-1.0, 0.0, 0.0).into(),
                (1.0, 0.0, 0.0).into(),
                (0.0, 0.0, 1.0).into(),
            ),
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
            let path = mesh
                .find_path(
                    (0.0, 0.0, 0.0).into(),
                    (2.0, 0.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::MidPoints,
                )
                .unwrap();
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
            let path = mesh
                .find_path(
                    (0.0, 0.0, 0.0).into(),
                    (2.0, 0.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                )
                .unwrap();
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
        }
        {
            let path = mesh
                .find_path(
                    (2.0, 0.0, 0.0).into(),
                    (0.0, 2.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::MidPoints,
                )
                .unwrap();
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
            let path = mesh
                .find_path(
                    (2.0, 0.0, 0.0).into(),
                    (0.0, 2.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                )
                .unwrap();
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(20, 0, 0), (10, 10, 0), (0, 20, 0),]
            );
        }
        {
            let path = mesh
                .find_path(
                    (2.0, 1.0, 0.0).into(),
                    (1.0, 2.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::MidPoints,
                )
                .unwrap();
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
            let path = mesh
                .find_path(
                    (2.0, 1.0, 0.0).into(),
                    (1.0, 2.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                )
                .unwrap();
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(20, 10, 0), (10, 10, 0), (10, 20, 0),]
            );
        }
        {
            let path = mesh
                .find_path(
                    (0.5, 0.0, 0.0).into(),
                    (0.5, 2.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::MidPoints,
                )
                .unwrap();
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
            let path = mesh
                .find_path(
                    (0.5, 0.0, 0.0).into(),
                    (0.5, 2.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                )
                .unwrap();
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
        }

        let vertices = vec![
            (0.0, 0.0, 0.0).into(), // 0
            (2.0, 0.0, 0.0).into(), // 1
            (2.0, 1.0, 0.0).into(), // 2
            (1.0, 1.0, 0.0).into(), // 3
            (0.0, 2.0, 0.0).into(), // 4
        ];
        let triangles = vec![
            (0, 3, 4).into(), // 0
            (0, 1, 3).into(), // 1
            (1, 2, 3).into(), // 2
        ];
        let mesh = NavMesh::new(vertices.clone(), triangles.clone()).unwrap();
        {
            let path = mesh.find_path_triangles(0, 2).unwrap().0;
            assert_eq!(path, vec![0, 1, 2]);
        }
        {
            let path = mesh
                .find_path(
                    (2.0, 1.0, 0.0).into(),
                    (0.0, 2.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::MidPoints,
                )
                .unwrap();
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(20, 10, 0), (5, 5, 0), (0, 20, 0),]
            );
            let path = mesh
                .find_path(
                    (2.0, 1.0, 0.0).into(),
                    (0.0, 2.0, 0.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                )
                .unwrap();
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(20, 10, 0), (10, 10, 0), (0, 20, 0),]
            );
        }

        let vertices = vec![
            (0.0, 0.0, 0.0).into(), // 0
            (1.0, 0.0, 0.0).into(), // 1
            (2.0, 0.0, 1.0).into(), // 2
            (0.0, 1.0, 0.0).into(), // 3
            (1.0, 1.0, 0.0).into(), // 4
            (2.0, 1.0, 1.0).into(), // 5
        ];
        let triangles = vec![
            (0, 1, 4).into(), // 0
            (4, 3, 0).into(), // 1
            (1, 2, 5).into(), // 2
            (5, 4, 1).into(), // 3
        ];
        let mesh = NavMesh::new(vertices.clone(), triangles.clone()).unwrap();
        {
            let path = mesh.find_path_triangles(1, 2).unwrap().0;
            assert_eq!(path, vec![1, 0, 3, 2]);
        }
        {
            let path = mesh
                .find_path(
                    (0.0, 0.5, 0.0).into(),
                    (2.0, 0.5, 1.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::MidPoints,
                )
                .unwrap();
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(0, 5, 0), (10, 5, 0), (20, 5, 10),]
            );
            let path = mesh
                .find_path(
                    (0.0, 0.5, 0.0).into(),
                    (2.0, 0.5, 1.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                )
                .unwrap();
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(0, 5, 0), (10, 5, 0), (20, 5, 10),]
            );
        }
        {
            let path = mesh
                .find_path(
                    (0.0, 0.0, 0.0).into(),
                    (2.0, 1.0, 1.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::MidPoints,
                )
                .unwrap();
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(0, 0, 0), (10, 5, 0), (20, 10, 10),]
            );
            let path = mesh
                .find_path(
                    (0.0, 0.0, 0.0).into(),
                    (2.0, 1.0, 1.0).into(),
                    NavQuery::Accuracy,
                    NavPathMode::Accuracy,
                )
                .unwrap();
            assert_eq!(
                path.into_iter()
                    .map(|v| (
                        (v.x * 10.0) as i32,
                        (v.y * 10.0) as i32,
                        (v.z * 10.0) as i32,
                    ))
                    .collect::<Vec<_>>(),
                vec![(0, 0, 0), (10, 5, 0), (20, 10, 10),]
            );
        }
    }
}
