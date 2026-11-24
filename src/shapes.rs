//! Library of 2D shapes, some chiral and some achiral.

use crate::geom::{centroid, flip, mat_angle, symmetry_group, triangulate};
use macroquad::math::{vec2, Mat2, Vec2};
use std::f32::consts::{PI, TAU};

pub const SHAPE_RADIUS: f32 = 90.0;

pub struct ShapeDef {
    pub name: &'static str,
    pub blurb: &'static str,
    /// Vertices centred on the centroid, scaled to SHAPE_RADIUS.
    pub pts: Vec<Vec2>,
    pub tris: Vec<[usize; 3]>,
    /// Symmetry group (orthogonal matrices about the centroid).
    pub group: Vec<Mat2>,
    /// Vertex-based continuous chirality measure (0 = achiral, 0..100).
    pub ccm: f32,
}

impl ShapeDef {
    fn new(name: &'static str, blurb: &'static str, raw: Vec<Vec2>) -> ShapeDef {
        let c = centroid(&raw);
        let mut pts: Vec<Vec2> = raw.iter().map(|&p| p - c).collect();
        let r = pts.iter().map(|p| p.length()).fold(0.0, f32::max);
        for p in &mut pts {
            *p *= SHAPE_RADIUS / r;
        }
        let tris = triangulate(&pts);
        let group = symmetry_group(&pts);
        let ccm = chirality_measure(&pts);
        ShapeDef { name, blurb, pts, tris, group, ccm }
    }

    pub fn is_chiral(&self) -> bool {
        !self.group.iter().any(|m| m.determinant() < 0.0)
    }

    pub fn rotation_order(&self) -> usize {
        self.group.iter().filter(|m| m.determinant() > 0.0).count()
    }

    /// 2D point group: Cn (rotations only, chiral) or Dn (n mirror lines, achiral).
    pub fn group_name(&self) -> String {
        let n = self.rotation_order();
        if self.is_chiral() {
            format!("C{n}")
        } else {
            format!("D{n}")
        }
    }

    /// Angles of the shape's own mirror lines (through its centroid).
    pub fn mirror_line_angles(&self) -> Vec<f32> {
        self.group
            .iter()
            .filter(|m| m.determinant() < 0.0)
            .map(|&m| mat_angle(m) * 0.5)
            .collect()
    }
}

/// Zabrodsky–Avnir style continuous chirality measure on the polygon's vertices:
/// S = 100 · mean |Pᵢ − Qᵢ|² / r_max², where Q is the nearest achiral polygon,
/// found by averaging P with its best-aligned mirror image (all vertex pairings tried).
fn chirality_measure(pts: &[Vec2]) -> f32 {
    let n = pts.len();
    let mean = pts.iter().copied().sum::<Vec2>() / n as f32;
    let p: Vec<Vec2> = pts.iter().map(|&v| v - mean).collect();
    let rmax = p.iter().map(|v| v.length()).fold(0.0, f32::max);
    let mut best = f32::MAX;
    for k in 0..n {
        // Mirror image with reversed vertex order, so q[i] is paired with p[i].
        let q: Vec<Vec2> = (0..n).map(|i| flip() * p[(k + n - i) % n]).collect();
        let (mut c, mut s) = (0.0, 0.0);
        for i in 0..n {
            c += q[i].dot(p[i]);
            s += q[i].perp_dot(p[i]);
        }
        let r = Mat2::from_angle(s.atan2(c));
        let d: f32 = (0..n).map(|i| (r * q[i] - p[i]).length_squared()).sum();
        best = best.min(d);
    }
    // |P − Q|² = |P − M·P|² / 4.
    100.0 * best / 4.0 / n as f32 / (rmax * rmax)
}

fn grid(pts: &[(f32, f32)]) -> Vec<Vec2> {
    pts.iter().map(|&(x, y)| vec2(x, y)).collect()
}

/// Star-shaped pinwheel: `n` skewed blades -> n-fold rotation, no mirror lines.
fn pinwheel(n: usize, inner: f32, skew: f32) -> Vec<Vec2> {
    let step = TAU / n as f32;
    let polar = |r: f32, a: f32| vec2(r * a.cos(), r * a.sin());
    let mut v = Vec::new();
    for k in 0..n {
        let a = k as f32 * step;
        v.push(polar(inner, a));
        v.push(polar(1.0, a));
        v.push(polar(0.72, a + skew * step));
    }
    v
}

fn regular_star(n: usize, inner: f32) -> Vec<Vec2> {
    (0..2 * n)
        .map(|i| {
            let r = if i % 2 == 0 { 1.0 } else { inner };
            let a = -PI / 2.0 + i as f32 * PI / n as f32;
            vec2(r * a.cos(), r * a.sin())
        })
        .collect()
}

pub fn library() -> Vec<ShapeDef> {
    vec![
        ShapeDef::new(
            "F pentomino",
            "No rotation or slide turns it into its mirror image.",
            grid(&[(1., 0.), (3., 0.), (3., 1.), (2., 1.), (2., 3.), (1., 3.), (1., 2.), (0., 2.), (0., 1.), (1., 1.)]),
        ),
        ShapeDef::new(
            "Mitten",
            "Like your hands: a left and a right mitten are enantiomorphs.",
            grid(&[
                (1.4, 0.0), (3.0, 0.0), (3.4, 0.4), (3.4, 4.6), (2.8, 5.4), (1.6, 5.4), (1.0, 4.6),
                (1.0, 3.6), (0.1, 2.4), (0.1, 1.8), (0.5, 1.6), (1.0, 2.2), (1.0, 0.4),
            ]),
        ),
        ShapeDef::new(
            "L tetromino",
            "The L and J pieces of a falling-block game are mirror twins.",
            grid(&[(0., 0.), (1., 0.), (1., 2.), (2., 2.), (2., 3.), (0., 3.)]),
        ),
        ShapeDef::new(
            "S tetromino",
            "Has 2-fold rotation symmetry, yet is still chiral (S vs Z).",
            grid(&[(1., 0.), (3., 0.), (3., 1.), (2., 1.), (2., 2.), (0., 2.), (0., 1.), (1., 1.)]),
        ),
        ShapeDef::new(
            "Scalene triangle",
            "Three unequal sides: in the plane, even a triangle can be chiral.",
            grid(&[(0., 0.), (5., 0.), (1., 2.)]),
        ),
        ShapeDef::new(
            "Parallelogram",
            "Surprise: a slanted parallelogram is chiral in 2D.",
            grid(&[(0., 0.), (3., 0.), (4., 2.), (1., 2.)]),
        ),
        ShapeDef::new(
            "Pinwheel",
            "4-fold rotation symmetry but no mirror lines: chiral.",
            pinwheel(4, 0.3, 0.55),
        ),
        ShapeDef::new(
            "Triskelion",
            "3-fold rotation, spinning one way only.",
            pinwheel(3, 0.25, 0.45),
        ),
        ShapeDef::new(
            "Isosceles triangle",
            "One mirror line: its reflection is itself.",
            grid(&[(0., 3.), (2., 3.), (1., 0.)]),
        ),
        ShapeDef::new(
            "T tetromino",
            "One mirror line -> achiral.",
            grid(&[(0., 0.), (3., 0.), (3., 1.), (2., 1.), (2., 2.), (1., 2.), (1., 1.), (0., 1.)]),
        ),
        ShapeDef::new(
            "Arrow",
            "Achiral: reflect it along its shaft and nothing changes.",
            grid(&[(0., 1.), (2.5, 1.), (2.5, 0.), (4., 1.5), (2.5, 3.), (2.5, 2.), (0., 2.)]),
        ),
        ShapeDef::new("Square", "4 mirror lines, 4-fold rotation.", grid(&[(0., 0.), (1., 0.), (1., 1.), (0., 1.)])),
        ShapeDef::new("Star", "5 mirror lines, 5-fold rotation.", regular_star(5, 0.42)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chirality_and_symmetry() {
        // (name, chiral, rotation order, mirror lines)
        let expect = [
            ("F pentomino", true, 1, 0),
            ("Mitten", true, 1, 0),
            ("L tetromino", true, 1, 0),
            ("S tetromino", true, 2, 0),
            ("Scalene triangle", true, 1, 0),
            ("Parallelogram", true, 2, 0),
            ("Pinwheel", true, 4, 0),
            ("Triskelion", true, 3, 0),
            ("Isosceles triangle", false, 1, 1),
            ("T tetromino", false, 1, 1),
            ("Arrow", false, 1, 1),
            ("Square", false, 4, 4),
            ("Star", false, 5, 5),
        ];
        let lib = library();
        assert_eq!(lib.len(), expect.len());
        for (s, (name, chiral, rot, lines)) in lib.iter().zip(expect) {
            assert_eq!(s.name, name);
            assert_eq!(s.is_chiral(), chiral, "{name}");
            assert_eq!(s.rotation_order(), rot, "{name}");
            assert_eq!(s.mirror_line_angles().len(), lines, "{name}");
            assert_eq!(s.tris.len(), s.pts.len() - 2, "{name} triangulation");
            if chiral {
                assert!(s.ccm > 0.1, "{name} ccm {}", s.ccm);
            } else {
                assert!(s.ccm < 1e-3, "{name} ccm {}", s.ccm);
            }
            println!("{name}: group {} ccm {:.2}", s.group_name(), s.ccm);
        }
    }
}
