//! Point-group detection for labelled 3D point sets, plus a few
//! pseudoscalars (signed volume, torsion sign, writhe).

use macroquad::math::{Mat3, Vec3};
use std::f32::consts::{PI, TAU};

pub struct PointGroup {
    pub name: String,
    pub proper: usize,
    pub improper: usize,
    pub has_mirror: bool,
    pub has_inversion: bool,
}

impl PointGroup {
    pub fn chiral(&self) -> bool {
        self.improper == 0
    }
}

fn axis_of_rotation(m: Mat3) -> Vec3 {
    let v = Vec3::new(m.y_axis.z - m.z_axis.y, m.z_axis.x - m.x_axis.z, m.x_axis.y - m.y_axis.x);
    if v.length() > 1e-3 {
        return v.normalize();
    }
    // Angle ~ 0 or PI: the axis is the dominant column of (M + I).
    let s = m + Mat3::IDENTITY;
    [s.x_axis, s.y_axis, s.z_axis]
        .into_iter()
        .max_by(|a, b| a.length().total_cmp(&b.length()))
        .unwrap()
        .normalize_or_zero()
}

fn rotation_angle(m: Mat3) -> f32 {
    let tr = m.x_axis.x + m.y_axis.y + m.z_axis.z;
    ((tr - 1.0) / 2.0).clamp(-1.0, 1.0).acos()
}

fn mat_close(a: Mat3, b: Mat3) -> bool {
    a.abs_diff_eq(b, 2e-3)
}

/// All orthogonal maps (about the centroid) sending the labelled set onto itself.
pub fn symmetry_ops(points: &[(Vec3, u8)]) -> Vec<Mat3> {
    let n = points.len();
    let c = points.iter().map(|p| p.0).sum::<Vec3>() / n as f32;
    let p: Vec<(Vec3, u8)> = points.iter().map(|&(v, k)| (v - c, k)).collect();
    let scale = p.iter().map(|q| q.0.length()).fold(0.0, f32::max).max(1e-6);
    let tol = scale * 2e-3;

    let i = (0..n).max_by(|&a, &b| p[a].0.length().total_cmp(&p[b].0.length())).unwrap();
    let j = match (0..n)
        .filter(|&b| p[i].0.cross(p[b].0).length() > tol * scale * 5.0)
        .max_by(|&a, &b| p[i].0.cross(p[a].0).length().total_cmp(&p[i].0.cross(p[b].0).length()))
    {
        Some(j) => j,
        None => return vec![Mat3::IDENTITY], // collinear: not needed here
    };
    let (pi, pj) = (p[i].0, p[j].0);
    let basis = Mat3::from_cols(pi, pj, pi.cross(pj));
    let inv = basis.inverse();

    let maps_onto_itself = |m: Mat3| {
        p.iter().all(|&(v, k)| {
            let w = m * v;
            p.iter().any(|&(u, kk)| kk == k && u.distance(w) < tol * 4.0)
        })
    };

    let mut ops: Vec<Mat3> = Vec::new();
    for k in 0..n {
        if p[k].1 != p[i].1 || (p[k].0.length() - pi.length()).abs() > tol {
            continue;
        }
        for l in 0..n {
            if p[l].1 != p[j].1
                || (p[l].0.length() - pj.length()).abs() > tol
                || (p[k].0.dot(p[l].0) - pi.dot(pj)).abs() > tol * scale * 2.0
            {
                continue;
            }
            for s in [1.0f32, -1.0] {
                let target = Mat3::from_cols(p[k].0, p[l].0, p[k].0.cross(p[l].0) * s);
                let m = target * inv;
                if !(m.transpose() * m).abs_diff_eq(Mat3::IDENTITY, 1e-2) {
                    continue;
                }
                if ops.iter().any(|&o| mat_close(o, m)) {
                    continue;
                }
                if maps_onto_itself(m) {
                    ops.push(m);
                }
            }
        }
    }
    ops
}

pub fn point_group(points: &[(Vec3, u8)]) -> PointGroup {
    let ops = symmetry_ops(points);
    let proper: Vec<Mat3> = ops.iter().copied().filter(|m| m.determinant() > 0.0).collect();
    let improper: Vec<Mat3> = ops.iter().copied().filter(|m| m.determinant() < 0.0).collect();

    // Mirrors are improper ops with trace +1; their normal spans I - M.
    let mirrors: Vec<Vec3> = improper
        .iter()
        .filter(|m| (m.x_axis.x + m.y_axis.y + m.z_axis.z - 1.0).abs() < 1e-2)
        .map(|&m| {
            let s = Mat3::IDENTITY - m;
            [s.x_axis, s.y_axis, s.z_axis]
                .into_iter()
                .max_by(|a, b| a.length().total_cmp(&b.length()))
                .unwrap()
                .normalize()
        })
        .collect();
    let has_inversion = improper.iter().any(|&m| mat_close(m, -Mat3::IDENTITY));

    // Rotation orders and axes.
    let mut axes: Vec<(Vec3, usize)> = Vec::new();
    for &m in &proper {
        let a = rotation_angle(m);
        if a < 1e-2 {
            continue;
        }
        let order = (TAU / a).round() as usize;
        let ax = axis_of_rotation(m);
        if let Some(e) = axes.iter_mut().find(|(v, _)| v.cross(ax).length() < 1e-2) {
            e.1 = e.1.max(order);
        } else {
            axes.push((ax, order));
        }
    }
    let n = axes.iter().map(|a| a.1).max().unwrap_or(1);
    let high = axes.iter().filter(|a| a.1 >= 3).count() > 1;

    let name = if high {
        let base = match proper.len() {
            12 => "T",
            24 => "O",
            60 => "I",
            _ => "?",
        };
        if improper.is_empty() {
            base.to_string()
        } else if base == "T" && !has_inversion {
            "Td".into()
        } else {
            format!("{base}h")
        }
    } else if n == 1 {
        if !mirrors.is_empty() {
            "Cs".into()
        } else if has_inversion {
            "Ci".into()
        } else {
            "C1".into()
        }
    } else {
        let u = axes.iter().find(|a| a.1 == n).unwrap().0;
        let perp_c2 = axes.iter().filter(|a| a.1 == 2 && a.0.dot(u).abs() < 1e-2).count();
        let sigma_h = mirrors.iter().any(|m| m.cross(u).length() < 1e-2);
        if perp_c2 >= n {
            if sigma_h {
                format!("D{n}h")
            } else if !mirrors.is_empty() {
                format!("D{n}d")
            } else {
                format!("D{n}")
            }
        } else if sigma_h {
            format!("C{n}h")
        } else if !mirrors.is_empty() {
            format!("C{n}v")
        } else if !improper.is_empty() {
            format!("S{}", 2 * n)
        } else {
            format!("C{n}")
        }
    };

    PointGroup {
        name,
        proper: proper.len(),
        improper: improper.len(),
        has_mirror: !mirrors.is_empty(),
        has_inversion,
    }
}

/// χ = det[v1 − v4, v2 − v4, v3 − v4] (signed volume, a pseudoscalar).
pub fn signed_volume(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> f32 {
    Mat3::from_cols(v1 - v4, v2 - v4, v3 - v4).determinant()
}

/// Mean sign of the discrete torsion det[Δ0, Δ1, Δ2] along a polyline (+1 right-handed).
pub fn torsion_sign(pts: &[Vec3]) -> f32 {
    let mut s = 0.0;
    for w in pts.windows(4) {
        s += Mat3::from_cols(w[1] - w[0], w[2] - w[1], w[3] - w[2]).determinant().signum();
    }
    s / (pts.len().saturating_sub(3).max(1)) as f32
}

/// Writhe of a closed polygon via the discretised Gauss double integral.
pub fn writhe(pts: &[Vec3]) -> f32 {
    let n = pts.len();
    let seg: Vec<(Vec3, Vec3)> = (0..n).map(|i| ((pts[i] + pts[(i + 1) % n]) * 0.5, pts[(i + 1) % n] - pts[i])).collect();
    let mut w = 0.0;
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let r = seg[i].0 - seg[j].0;
            let d = r.length();
            if d < 1e-6 {
                continue;
            }
            w += seg[i].1.cross(seg[j].1).dot(r) / (d * d * d);
        }
    }
    w / (4.0 * PI)
}
