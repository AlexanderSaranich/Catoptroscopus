//! A smooth, anatomical LEFT hand built as a signed-distance field
//! (palm, thenar/hypothenar pads, jointed fingers, thumb, wrist, sleeve)
//! and polygonised once with naive surface nets.
//!
//! Local frame: palm faces +z, fingers point +y, thumb on the +x side.

use macroquad::math::{vec3, Vec3};

#[derive(Clone, Copy, PartialEq)]
pub enum Region {
    Skin,
    Nail,
    Sleeve,
}

pub struct HandMesh {
    pub pos: Vec<Vec3>,
    pub normal: Vec<Vec3>,
    pub region: Vec<Region>,
    /// Triangles split into chunks that fit one macroquad draw call:
    /// (global vertex ids, local u16 indices).
    pub chunks: Vec<(Vec<u32>, Vec<u16>)>,
    /// Fingertips and thumb tip (for "these don't line up" markers).
    pub tips: Vec<Vec3>,
    /// Centre of the palm surface.
    pub palm: Vec3,
}

// ---------- SDF primitives ----------

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// Capsule with linearly varying radius.
fn capsule(p: Vec3, a: Vec3, b: Vec3, ra: f32, rb: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
    (pa - ba * h).length() - (ra + (rb - ra) * h)
}

fn ellipsoid(p: Vec3, c: Vec3, r: Vec3) -> f32 {
    let q = (p - c) / r;
    (q.length() - 1.0) * r.min_element()
}

struct Segment {
    a: Vec3,
    b: Vec3,
    ra: f32,
    rb: f32,
}

struct Nail {
    center: Vec3,
    normal: Vec3,
    radius: f32,
}

struct Skeleton {
    fingers: Vec<Vec<Segment>>,
    thumb: Vec<Segment>,
    nails: Vec<Nail>,
    tips: Vec<Vec3>,
}

fn skeleton() -> Skeleton {
    // (base x, base y, length, base radius, tip radius, splay degrees)
    let fingers = [
        // Average proportions: the middle finger is about 0.43 of hand length.
        (0.415f32, 0.44f32, 0.88f32, 0.142f32, 0.114f32, 4.0f32), // index
        (0.14, 0.50, 0.98, 0.148, 0.118, 0.0),                       // middle
        (-0.14, 0.46, 0.92, 0.140, 0.110, -3.0),                     // ring
        (-0.40, 0.36, 0.74, 0.124, 0.098, -8.0),                     // little
    ];
    let phal = [0.46f32, 0.30, 0.24];
    let curl = [8.0f32, 24.0, 36.0];
    let z = Vec3::Z;
    let mut out = Skeleton { fingers: vec![], thumb: vec![], nails: vec![], tips: vec![] };
    for (bx, by, len, r0, r1, splay) in fingers {
        let s = splay.to_radians();
        let d = vec3(s.sin(), s.cos(), 0.0);
        // Start inside the palm so the knuckle blends in.
        let mut p = vec3(bx - d.x * 0.25, by - 0.25, 0.02);
        let mut segs = vec![Segment { a: p, b: vec3(bx, by, 0.02), ra: r0 * 1.15, rb: r0 * 1.05 }];
        p = vec3(bx, by, 0.02);
        let mut acc = 0.0;
        for (k, (&f, &c)) in phal.iter().zip(&curl).enumerate() {
            let c = c.to_radians();
            let dir = d * c.cos() + z * c.sin();
            let l = len * f;
            let ra = r0 + (r1 - r0) * acc;
            acc += f;
            let rb = r0 + (r1 - r0) * acc;
            let q = p + dir * l;
            // Slight knuckle bulge at the start of each phalanx.
            segs.push(Segment { a: p, b: q, ra: ra * if k == 0 { 1.06 } else { 1.03 }, rb });
            if k == 2 {
                let palm_side = z * c.cos() - d * c.sin();
                out.nails.push(Nail { center: p + dir * (l * 0.6) - palm_side * (rb * 0.8), normal: -palm_side, radius: rb * 1.05 });
                out.tips.push(q + dir * rb);
            }
            p = q;
        }
        out.fingers.push(segs);
    }
    // Thumb: metacarpal, proximal and distal phalanges.
    let j = [vec3(0.30, -0.42, 0.06), vec3(0.68, -0.14, 0.22), vec3(0.86, 0.08, 0.30), vec3(0.97, 0.29, 0.34)];
    let r = [(0.20, 0.168), (0.158, 0.142), (0.138, 0.115)];
    for k in 0..3 {
        out.thumb.push(Segment { a: j[k], b: j[k + 1], ra: r[k].0, rb: r[k].1 });
    }
    let dir = (j[3] - j[2]).normalize();
    let back = (dir.cross(z).normalize() - z).normalize();
    out.nails.push(Nail { center: j[2] + (j[3] - j[2]) * 0.6 + back * 0.08, normal: back, radius: 0.11 });
    out.tips.push(j[3] + dir * 0.098);
    out
}

fn forearm(p: Vec3) -> f32 {
    let q = vec3(p.x, p.y, p.z / 0.72);
    capsule(q, vec3(0.0, -0.55, 0.0), vec3(0.0, -2.05, 0.0), 0.34, 0.38) * 0.8
}

fn sleeve(p: Vec3) -> f32 {
    let q = vec3(p.x, p.y, p.z / 0.82);
    capsule(q, vec3(0.0, -1.18, 0.0), vec3(0.0, -2.05, 0.0), 0.45, 0.47) * 0.88
}

fn smax(a: f32, b: f32, k: f32) -> f32 {
    -smin(-a, -b, k)
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Palm: a curved outline (bulging sides, knuckle-line arch, narrow wrist)
/// given thickness with rounded rims, slightly cupped on the palm side.
fn palm(p: Vec3) -> f32 {
    let (x, y) = (p.x, p.y);
    // Little-finger side swells out past the wrist, then eases in at the knuckle.
    let left = -(0.34 + 0.27 * smoothstep(-0.8, -0.05, y) - 0.07 * smoothstep(-0.02, 0.45, y));
    // Index side flares more gently towards the index knuckle.
    let right = 0.34 + 0.22 * smoothstep(-0.8, 0.08, y);
    // Top follows the knuckle arc: highest under the middle finger.
    let top = 0.52 - 0.55 * (x - 0.08) * (x - 0.08);
    let bottom = -0.75;
    let k = 0.16;
    let d2 = smax(smax(left - x, x - right, k), smax(y - top, bottom - y, k), k);
    // Domed cross-section: thickest in the middle, thinning towards the rim,
    // with a shallow hollow in the palm.
    let dome = smoothstep(0.05, -0.45, d2);
    let hollow = 0.045 * (-((x - 0.02).powi(2) + (y + 0.06).powi(2)) / 0.10).exp();
    let t = if p.z > 0.0 { 0.12 + 0.12 * dome - hollow } else { 0.12 + 0.14 * dome };
    smax(d2, p.z.abs() - t, 0.14)
}

fn sdf(sk: &Skeleton, p: Vec3) -> f32 {
    let mut d = palm(p);
    // Thenar (thumb ball) and hypothenar pads on the palm side.
    d = smin(d, ellipsoid(p, vec3(0.36, -0.30, 0.10), vec3(0.30, 0.36, 0.20)), 0.12);
    d = smin(d, ellipsoid(p, vec3(-0.40, -0.22, 0.08), vec3(0.22, 0.38, 0.17)), 0.10);
    // Wrist and forearm.
    d = smin(d, forearm(p), 0.22);
    let body = d;
    // Thumb blends into the thenar region, tightly enough to keep a clean crotch.
    let mut th = f32::MAX;
    for s in &sk.thumb {
        th = smin(th, capsule(p, s.a, s.b, s.ra, s.rb), 0.05);
    }
    d = smin(d, th, 0.08);
    // Fingers: each blends into the palm only, then joins with a hard union,
    // so neighbouring fingers stay separate down to the knuckles.
    for f in &sk.fingers {
        let mut fd = f32::MAX;
        for s in f {
            fd = smin(fd, capsule(p, s.a, s.b, s.ra, s.rb), 0.035);
        }
        d = d.min(smin(body, fd, 0.06));
    }
    // Sleeve: a crisp union.
    d.min(sleeve(p))
}

fn gradient(sk: &Skeleton, p: Vec3) -> Vec3 {
    let e = 0.004;
    vec3(
        sdf(sk, p + vec3(e, 0.0, 0.0)) - sdf(sk, p - vec3(e, 0.0, 0.0)),
        sdf(sk, p + vec3(0.0, e, 0.0)) - sdf(sk, p - vec3(0.0, e, 0.0)),
        sdf(sk, p + vec3(0.0, 0.0, e)) - sdf(sk, p - vec3(0.0, 0.0, e)),
    )
    .normalize_or_zero()
}

/// Polygonise the hand with naive surface nets.
pub fn build() -> HandMesh {
    let sk = skeleton();
    let h = 0.03f32;
    let lo = vec3(-0.85, -2.25, -0.6);
    let hi = vec3(1.35, 2.05, 0.75);
    let n = ((hi - lo) / h).ceil();
    let (sx, sy, sz) = (n.x as usize + 1, n.y as usize + 1, n.z as usize + 1);
    let at = |i: usize, j: usize, k: usize| lo + vec3(i as f32, j as f32, k as f32) * h;
    let idx = |i: usize, j: usize, k: usize| i + sx * (j + sy * k);

    let mut f = vec![0.0f32; sx * sy * sz];
    for k in 0..sz {
        for j in 0..sy {
            for i in 0..sx {
                f[idx(i, j, k)] = sdf(&sk, at(i, j, k));
            }
        }
    }

    // One vertex per cell that straddles the surface.
    const CORNERS: [(usize, usize, usize); 8] =
        [(0, 0, 0), (1, 0, 0), (0, 1, 0), (1, 1, 0), (0, 0, 1), (1, 0, 1), (0, 1, 1), (1, 1, 1)];
    const EDGES: [(usize, usize); 12] =
        [(0, 1), (2, 3), (4, 5), (6, 7), (0, 2), (1, 3), (4, 6), (5, 7), (0, 4), (1, 5), (2, 6), (3, 7)];
    let cidx = |i: usize, j: usize, k: usize| i + (sx - 1) * (j + (sy - 1) * k);
    let mut cell_vert = vec![u32::MAX; (sx - 1) * (sy - 1) * (sz - 1)];
    let mut pos = Vec::new();
    for k in 0..sz - 1 {
        for j in 0..sy - 1 {
            for i in 0..sx - 1 {
                let v: Vec<f32> = CORNERS.iter().map(|&(a, b, c)| f[idx(i + a, j + b, k + c)]).collect();
                let inside = v.iter().filter(|&&x| x < 0.0).count();
                if inside == 0 || inside == 8 {
                    continue;
                }
                let mut sum = Vec3::ZERO;
                let mut cnt = 0.0;
                for &(e0, e1) in &EDGES {
                    let (a, b) = (v[e0], v[e1]);
                    if (a < 0.0) != (b < 0.0) {
                        let t = a / (a - b);
                        let (c0, c1) = (CORNERS[e0], CORNERS[e1]);
                        let p0 = at(i + c0.0, j + c0.1, k + c0.2);
                        let p1 = at(i + c1.0, j + c1.1, k + c1.2);
                        sum += p0 + (p1 - p0) * t;
                        cnt += 1.0;
                    }
                }
                cell_vert[cidx(i, j, k)] = pos.len() as u32;
                pos.push(sum / cnt);
            }
        }
    }

    // A quad around every grid edge that crosses the surface.
    let mut tris: Vec<[u32; 3]> = Vec::new();
    let mut quad = |c: [u32; 4], flip: bool| {
        if c.iter().any(|&x| x == u32::MAX) {
            return;
        }
        if flip {
            tris.push([c[0], c[2], c[1]]);
            tris.push([c[0], c[3], c[2]]);
        } else {
            tris.push([c[0], c[1], c[2]]);
            tris.push([c[0], c[2], c[3]]);
        }
    };
    for k in 1..sz - 1 {
        for j in 1..sy - 1 {
            for i in 1..sx - 1 {
                let a = f[idx(i, j, k)] < 0.0;
                if i < sx - 1 && a != (f[idx(i + 1, j, k)] < 0.0) {
                    quad([cell_vert[cidx(i, j - 1, k - 1)], cell_vert[cidx(i, j, k - 1)], cell_vert[cidx(i, j, k)], cell_vert[cidx(i, j - 1, k)]], a);
                }
                if j < sy - 1 && a != (f[idx(i, j + 1, k)] < 0.0) {
                    quad([cell_vert[cidx(i - 1, j, k - 1)], cell_vert[cidx(i - 1, j, k)], cell_vert[cidx(i, j, k)], cell_vert[cidx(i, j, k - 1)]], a);
                }
                if k < sz - 1 && a != (f[idx(i, j, k + 1)] < 0.0) {
                    quad([cell_vert[cidx(i - 1, j - 1, k)], cell_vert[cidx(i, j - 1, k)], cell_vert[cidx(i, j, k)], cell_vert[cidx(i - 1, j, k)]], a);
                }
            }
        }
    }

    let normal: Vec<Vec3> = pos.iter().map(|&p| gradient(&sk, p)).collect();
    let region: Vec<Region> = pos
        .iter()
        .zip(&normal)
        .map(|(&p, &nrm)| {
            if sleeve(p) < 0.02 {
                Region::Sleeve
            } else if sk.nails.iter().any(|nl| p.distance(nl.center) < nl.radius && nrm.dot(nl.normal) > 0.35) {
                Region::Nail
            } else {
                Region::Skin
            }
        })
        .collect();

    // Centre the model.
    let c = pos.iter().copied().sum::<Vec3>() / pos.len() as f32;
    let pos: Vec<Vec3> = pos.into_iter().map(|p| p - c).collect();
    let tips = sk.tips.iter().map(|&t| t - c).collect();
    let palm = vec3(0.0, -0.03, 0.24) - c;

    // Split into draw-call sized chunks (macroquad: <= 5000 indices per call).
    let mut chunks = Vec::new();
    for group in tris.chunks(1500) {
        let mut map = std::collections::HashMap::new();
        let mut ids: Vec<u32> = Vec::new();
        let mut ind: Vec<u16> = Vec::new();
        for t in group {
            for &g in t {
                let l = *map.entry(g).or_insert_with(|| {
                    ids.push(g);
                    (ids.len() - 1) as u16
                });
                ind.push(l);
            }
        }
        chunks.push((ids, ind));
    }

    HandMesh { pos, normal, region, chunks, tips, palm }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_mesh_is_reasonable() {
        let m = build();
        let tris: usize = m.chunks.iter().map(|c| c.1.len() / 3).sum();
        assert!(m.pos.len() > 3000 && m.pos.len() < 60000, "verts {}", m.pos.len());
        assert!(tris > 5000, "tris {tris}");
        assert!(m.region.iter().any(|&r| r == Region::Nail));
        assert!(m.region.iter().any(|&r| r == Region::Sleeve));
        assert_eq!(m.tips.len(), 5);
        // The thumb is on the +x side and the fingers point +y: a left hand.
        let thumb = m.tips[4];
        let middle = m.tips[1];
        assert!(thumb.x > middle.x && middle.y > thumb.y);
        for c in &m.chunks {
            assert!(c.1.len() <= 4500 && c.0.len() <= 4500);
        }
    }
}

