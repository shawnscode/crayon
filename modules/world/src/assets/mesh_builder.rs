use crayon::errors::*;

use crayon::utils::hash::FastHashMap;
use crayon::video;
use crayon::video::assets::mesh::*;

impl_vertex!{
    Vertex {
        position => [Position; Float; 3; false],
        normal => [Normal; Float; 3; false],
        texcoord => [Texcoord0; Float; 2; false],
    }
}

pub fn quad() -> Result<MeshHandle> {
    let verts: [Vertex; 4] = [
        Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0, -1.0], [0.0, 0.0]),
        Vertex::new([0.5, -0.5, 0.0], [0.0, 0.0, -1.0], [1.0, 0.0]),
        Vertex::new([0.5, 0.5, 0.0], [0.0, 0.0, -1.0], [1.0, 1.0]),
        Vertex::new([-0.5, 0.5, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0]),
    ];

    let idxes: [u16; 6] = [0, 1, 2, 0, 2, 3];

    let mut params = MeshParams::default();
    params.num_verts = verts.len();
    params.num_idxes = idxes.len();
    params.layout = Vertex::layout();

    let data = MeshData {
        vptr: Vertex::encode(&verts[..]).into(),
        iptr: IndexFormat::encode(&idxes).into(),
    };

    let mesh = video::create_mesh(params, Some(data))?;
    Ok(mesh)
}

pub fn cube() -> Result<MeshHandle> {
    let texcoords = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let points = [
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
    ];

    let normals = [
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
    ];

    let verts = vec![
        Vertex::new(points[0], normals[0], texcoords[0]),
        Vertex::new(points[1], normals[0], texcoords[1]),
        Vertex::new(points[2], normals[0], texcoords[2]),
        Vertex::new(points[3], normals[0], texcoords[3]),
        Vertex::new(points[1], normals[1], texcoords[0]),
        Vertex::new(points[5], normals[1], texcoords[1]),
        Vertex::new(points[6], normals[1], texcoords[2]),
        Vertex::new(points[2], normals[1], texcoords[3]),
        Vertex::new(points[5], normals[2], texcoords[0]),
        Vertex::new(points[4], normals[2], texcoords[1]),
        Vertex::new(points[7], normals[2], texcoords[2]),
        Vertex::new(points[6], normals[2], texcoords[3]),
        Vertex::new(points[4], normals[3], texcoords[0]),
        Vertex::new(points[0], normals[3], texcoords[1]),
        Vertex::new(points[3], normals[3], texcoords[2]),
        Vertex::new(points[7], normals[3], texcoords[3]),
        Vertex::new(points[3], normals[4], texcoords[0]),
        Vertex::new(points[2], normals[4], texcoords[1]),
        Vertex::new(points[6], normals[4], texcoords[2]),
        Vertex::new(points[7], normals[4], texcoords[3]),
        Vertex::new(points[4], normals[5], texcoords[0]),
        Vertex::new(points[5], normals[5], texcoords[1]),
        Vertex::new(points[1], normals[5], texcoords[2]),
        Vertex::new(points[0], normals[5], texcoords[3]),
    ];

    let idxes: [u16; 36] = [
        0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17,
        18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
    ];

    let mut params = MeshParams::default();
    params.num_verts = verts.len();
    params.num_idxes = idxes.len();
    params.layout = Vertex::layout();

    let data = MeshData {
        vptr: Vertex::encode(&verts[..]).into(),
        iptr: IndexFormat::encode(&idxes).into(),
    };

    let mesh = video::create_mesh(params, Some(data))?;
    Ok(mesh)
}

pub fn sphere(iteration: usize) -> Result<MeshHandle> {
    use std::f32::consts::FRAC_1_PI;

    fn normalize(v: [f32; 3]) -> Vertex {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let v = [v[0] / l, v[1] / l, v[2] / l];
        let uv = [v[0].asin() * FRAC_1_PI + 0.5, v[1].asin() * FRAC_1_PI + 0.5];

        Vertex::new(v, v, uv)
    }

    let t = (1.0f32 + 5.0f32.sqrt()) / 2.0f32;
    let mut verts = vec![
        normalize([-1.0, t, 0.0]),
        normalize([1.0, t, 0.0]),
        normalize([-1.0, -t, 0.0]),
        normalize([1.0, -t, 0.0]),
        normalize([0.0, -1.0, t]),
        normalize([0.0, 1.0, t]),
        normalize([0.0, -1.0, -t]),
        normalize([0.0, 1.0, -t]),
        normalize([t, 0.0, -1.0]),
        normalize([t, 0.0, 1.0]),
        normalize([-t, 0.0, -1.0]),
        normalize([-t, 0.0, 1.0]),
    ];

    let mut faces: Vec<[u16; 3]> = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];

    {
        let mut cache = FastHashMap::default();
        let mut mid = |p1: usize, p2: usize| {
            // first check if we have it already
            let first_is_smaller = p1 < p2;
            let smaller = if first_is_smaller { p1 } else { p2 };
            let greater = if first_is_smaller { p2 } else { p1 };
            let k = (smaller, greater);

            if let Some(&v) = cache.get(&k) {
                return v;
            }

            let p1 = verts[p1];
            let p2 = verts[p2];
            let mid = normalize([
                (p1.position[0] + p2.position[0]) * 0.5,
                (p1.position[1] + p2.position[1]) * 0.5,
                (p1.position[2] + p2.position[2]) * 0.5,
            ]);

            verts.push(mid);
            cache.insert(k, verts.len() - 1);
            return verts.len() - 1;
        };

        let mut buf = Vec::new();
        for _ in 0..iteration {
            buf.clear();
            for face in &faces {
                let a = mid(face[0] as usize, face[1] as usize) as u16;
                let b = mid(face[1] as usize, face[2] as usize) as u16;
                let c = mid(face[2] as usize, face[0] as usize) as u16;

                buf.push([face[0], a, c]);
                buf.push([face[1], b, a]);
                buf.push([face[2], c, b]);
                buf.push([a, b, c]);
            }

            ::std::mem::swap(&mut faces, &mut buf);
        }
    }

    let idxes: Vec<u16> = faces.iter().flat_map(|v| v.iter().cloned()).collect();
    let mut params = MeshParams::default();
    params.num_verts = verts.len();
    params.num_idxes = idxes.len();
    params.layout = Vertex::layout();

    let data = MeshData {
        vptr: Vertex::encode(&verts[..]).into(),
        iptr: IndexFormat::encode(&idxes).into(),
    };

    let mesh = video::create_mesh(params, Some(data))?;
    Ok(mesh)
}
