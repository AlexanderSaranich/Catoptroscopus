//! Small 2D geometry kit: affine transforms, reflections, polygons,
//! triangulation and symmetry-group detection.

use macroquad::math::{vec2, Mat2, Vec2};

/// The reflection matrix diag(1, -1) (flip across the local x axis).
pub fn flip() -> Mat2 {
    Mat2::from_cols(vec2(1.0, 0.0), vec2(0.0, -1.0))
}

/// Angle (radians) of the first column of a 2x2 matrix.
pub fn mat_angle(m: Mat2) -> f32 {
    m.x_axis.y.atan2(m.x_axis.x)
}

/// An isometry x -> m * x + t.
#[derive(Clone, Copy, Debug)]
pub struct Xf {
    pub m: Mat2,
    pub t: Vec2,
}

impl Xf {
    pub const IDENTITY: Xf = Xf { m: Mat2::IDENTITY, t: Vec2::ZERO };

    pub fn apply(&self, p: Vec2) -> Vec2 {
        self.m * p + self.t
    }

    /// First apply `self`, then `other`.
    pub fn then(&self, other: &Xf) -> Xf {
        Xf { m: other.m * self.m, t: other.m * self.t + other.t }
    }

    /// +1 = orientation preserving (same hand), -1 = mirror image (opposite hand).
    pub fn det(&self) -> f32 {
        self.m.determinant()
    }

    pub fn approx_eq(&self, o: &Xf) -> bool {
        self.m.abs_diff_eq(o.m, 1e-3) && self.t.distance(o.t) < 0.5
    }
}

/// Reflection across the infinite line through `a` and `b`.
pub fn reflection(a: Vec2, b: Vec2) -> Xf {
    let d = (b - a).normalize_or_zero();
    if d == Vec2::ZERO {
        return Xf::IDENTITY;
    }
    let c = d.x * d.x - d.y * d.y; // cos 2θ
    let s = 2.0 * d.x * d.y; // sin 2θ
    let m = Mat2::from_cols(vec2(c, s), vec2(s, -c));
    Xf { m, t: a - m * a }
}

pub fn signed_area(p: &[Vec2]) -> f32 {
    let n = p.len();
    (0..n).map(|i| p[i].perp_dot(p[(i + 1) % n])).sum::<f32>() * 0.5
}

pub fn centroid(p: &[Vec2]) -> Vec2 {
    let n = p.len();
    let mut a = 0.0;
    let mut c = Vec2::ZERO;
    for i in 0..n {
        let j = (i + 1) % n;
        let cr = p[i].perp_dot(p[j]);
        a += cr;
        c += (p[i] + p[j]) * cr;
    }
    c / (3.0 * a)
}

pub fn point_in_polygon(pt: Vec2, poly: &[Vec2]) -> bool {
    let n = poly.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (poly[i], poly[j]);
        if (a.y > pt.y) != (b.y > pt.y) && pt.x < (b.x - a.x) * (pt.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Distance from `p` to the infinite line through `a`, `b`.
pub fn dist_to_line(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let d = (b - a).normalize_or_zero();
    if d == Vec2::ZERO {
        return p.distance(a);
    }
    (p - a).perp_dot(d).abs()
}

fn point_in_tri(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let d1 = (b - a).perp_dot(p - a);
    let d2 = (c - b).perp_dot(p - b);
    let d3 = (a - c).perp_dot(p - c);
    let eps = 1e-6;
    let has_neg = d1 < -eps || d2 < -eps || d3 < -eps;
    let has_pos = d1 > eps || d2 > eps || d3 > eps;
    !(has_neg && has_pos)
}

/// Ear-clipping triangulation of a simple polygon (any winding).
pub fn triangulate(poly: &[Vec2]) -> Vec<[usize; 3]> {
    let sign = signed_area(poly).signum();
    let mut idx: Vec<usize> = (0..poly.len()).collect();
    let mut tris = Vec::new();
    while idx.len() > 3 {
        let m = idx.len();
        let mut clipped = None;
        for i in 0..m {
            let (ia, ib, ic) = (idx[(i + m - 1) % m], idx[i], idx[(i + 1) % m]);
            let (a, b, c) = (poly[ia], poly[ib], poly[ic]);
            if (b - a).perp_dot(c - b) * sign <= 1e-6 {
                continue; // reflex or straight corner
            }
            let blocked = idx
                .iter()
                .filter(|&&k| k != ia && k != ib && k != ic)
                .any(|&k| point_in_tri(poly[k], a, b, c));
            if !blocked {
                clipped = Some(i);
                break;
            }
        }
        let i = clipped.unwrap_or(0); // degenerate fallback: force progress
        let m = idx.len();
        tris.push([idx[(i + m - 1) % m], idx[i], idx[(i + 1) % m]]);
        idx.remove(i);
    }
    if idx.len() == 3 {
        tris.push([idx[0], idx[1], idx[2]]);
    }
    tris
}

/// All orthogonal maps M (about the origin) with M(poly) == poly.
/// `poly` must already be centred on its centroid.
/// det(M) = +1 -> rotation symmetry, det(M) = -1 -> mirror symmetry.
pub fn symmetry_group(p: &[Vec2]) -> Vec<Mat2> {
    let n = p.len();
    let r = (0..n)
        .max_by(|&a, &b| p[a].length().total_cmp(&p[b].length()))
        .unwrap();
    let tol = p[r].length() * 4e-3;
    let ang = |v: Vec2| v.y.atan2(v.x);
    let mut out: Vec<Mat2> = Vec::new();
    for k in 0..n {
        if (p[k].length() - p[r].length()).abs() > tol {
            continue;
        }
        // Proper rotation: keeps vertex order.
        let m = Mat2::from_angle(ang(p[k]) - ang(p[r]));
        if (0..n).all(|i| (m * p[(r + i) % n] - p[(k + i) % n]).length() < tol) {
            out.push(m);
        }
        // Reflection: reverses vertex order.
        let fr = flip() * p[r];
        let m = Mat2::from_angle(ang(p[k]) - ang(fr)) * flip();
        if (0..n).all(|i| (m * p[(r + i) % n] - p[(k + n - i) % n]).length() < tol) {
            out.push(m);
        }
    }
    out
}
