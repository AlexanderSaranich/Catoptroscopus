//! Minimal software 3D renderer: right-handed orbit camera, perspective
//! projection and depth-sorted shaded spheres, sticks and arrows.

use crate::ui::*;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct Cam {
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
}

impl Cam {
    pub fn new(yaw: f32, pitch: f32, dist: f32) -> Cam {
        Cam { yaw, pitch, dist }
    }

    /// World -> view rotation. View space: x right, y up, camera on +z looking at -z.
    pub fn rot(&self) -> Quat {
        Quat::from_rotation_x(self.pitch) * Quat::from_rotation_y(self.yaw)
    }

    pub fn focal(&self) -> f32 {
        (screen_width() - SIDEBAR_W).min(screen_height()) * 1.25
    }

    pub fn center2d(&self) -> Vec2 {
        vec2(SIDEBAR_W + (screen_width() - SIDEBAR_W) / 2.0, screen_height() / 2.0)
    }

    /// Camera position in world space.
    pub fn eye(&self) -> Vec3 {
        self.rot().inverse() * vec3(0.0, 0.0, self.dist)
    }

    /// World point -> (screen point, depth). None if behind the camera.
    pub fn project(&self, p: Vec3) -> Option<(Vec2, f32)> {
        let v = self.rot() * p;
        let z = self.dist - v.z;
        if z < 0.5 {
            return None;
        }
        let f = self.focal() / z;
        let c = self.center2d();
        Some((vec2(c.x + v.x * f, c.y - v.y * f), z))
    }

    /// World-space rotation corresponding to a screen-space mouse drag ("trackball").
    pub fn drag_rotation(&self, d: Vec2) -> Quat {
        let k = 0.01;
        let rv = Quat::from_rotation_x(d.y * k) * Quat::from_rotation_y(d.x * k);
        let v = self.rot();
        v.inverse() * rv * v
    }

    pub fn orbit(&mut self, d: Vec2) {
        self.yaw += d.x * 0.008;
        self.pitch = (self.pitch + d.y * 0.008).clamp(-1.4, 1.4);
    }

    /// Arrow keys orbit, wheel zooms. Returns nothing; call once per frame.
    pub fn keys_and_wheel(&mut self, allow_arrows: bool) {
        let dt = get_frame_time();
        if allow_arrows {
            if is_key_down(KeyCode::Left) {
                self.yaw -= dt;
            }
            if is_key_down(KeyCode::Right) {
                self.yaw += dt;
            }
            if is_key_down(KeyCode::Up) {
                self.pitch = (self.pitch + dt).clamp(-1.4, 1.4);
            }
            if is_key_down(KeyCode::Down) {
                self.pitch = (self.pitch - dt).clamp(-1.4, 1.4);
            }
        }
        let wheel = mouse_wheel().1;
        if wheel != 0.0 && in_canvas(mouse()) {
            self.dist = (self.dist * if wheel > 0.0 { 0.92 } else { 1.08 }).clamp(5.0, 34.0);
        }
    }
}

pub enum Prim {
    Ball { c: Vec2, r: f32, col: Color, z: f32 },
    Stick { a: Vec2, b: Vec2, w: f32, col: Color, z: f32 },
    Dash { a: Vec2, b: Vec2, col: Color, z: f32 },
}

impl Prim {
    fn z(&self) -> f32 {
        match self {
            Prim::Ball { z, .. } | Prim::Stick { z, .. } | Prim::Dash { z, .. } => *z,
        }
    }
}

pub fn push_ball(cam: &Cam, out: &mut Vec<Prim>, p: Vec3, r: f32, col: Color) {
    if let Some((c, z)) = cam.project(p) {
        out.push(Prim::Ball { c, r: r * cam.focal() / z, col, z });
    }
}

pub fn push_stick(cam: &Cam, out: &mut Vec<Prim>, a: Vec3, b: Vec3, w: f32, col: Color) {
    if let (Some((pa, za)), Some((pb, zb))) = (cam.project(a), cam.project(b)) {
        let z = (za + zb) / 2.0;
        out.push(Prim::Stick { a: pa, b: pb, w: w * cam.focal() / z, col, z: z + 0.01 });
    }
}

pub fn push_dash(cam: &Cam, out: &mut Vec<Prim>, a: Vec3, b: Vec3, col: Color) {
    if let (Some((pa, za)), Some((pb, zb))) = (cam.project(a), cam.project(b)) {
        out.push(Prim::Dash { a: pa, b: pb, col, z: za.min(zb) - 1.0 });
    }
}

pub fn draw_ball(c: Vec2, r: f32, col: Color) {
    let dark = Color::new(col.r * 0.28, col.g * 0.28, col.b * 0.32, col.a);
    draw_circle(c.x, c.y, r, dark);
    let steps = if r > 6.0 { 9 } else { 4 };
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let rr = r * (1.0 - 0.8 * t);
        let off = vec2(-0.32, -0.36) * r * t;
        let k = 0.28 + 0.72 * t.powf(0.6);
        let hi = (t - 0.75).max(0.0) * 1.6;
        let c2 = Color::new(
            (col.r * k + hi).min(1.0),
            (col.g * k + hi).min(1.0),
            (col.b * k + hi).min(1.0),
            col.a,
        );
        draw_circle(c.x + off.x, c.y + off.y, rr, c2);
    }
}

/// Sort far-to-near and draw.
pub fn draw_prims(list: &mut [Prim]) {
    list.sort_by(|a, b| b.z().total_cmp(&a.z()));
    for p in list.iter() {
        match *p {
            Prim::Ball { c, r, col, .. } => draw_ball(c, r, col),
            Prim::Stick { a, b, w, col, .. } => {
                draw_line(a.x, a.y, b.x, b.y, w, Color::new(0.12, 0.13, 0.17, col.a));
                draw_line(a.x, a.y, b.x, b.y, w * 0.7, col);
            }
            Prim::Dash { a, b, col, .. } => dashed_line(a, b, 5.0, 2.5, col),
        }
    }
}

/// Translucent quad (e.g. a mirror plane).
pub fn draw_quad(cam: &Cam, corners: [Vec3; 4], col: Color) {
    let p: Vec<Vec2> = match corners.iter().map(|&c| cam.project(c).map(|x| x.0)).collect::<Option<Vec<_>>>() {
        Some(p) => p,
        None => return,
    };
    let fill = with_alpha(col, 0.13);
    draw_triangle(p[0], p[1], p[2], fill);
    draw_triangle(p[0], p[2], p[3], fill);
    for i in 0..4 {
        let (a, b) = (p[i], p[(i + 1) % 4]);
        draw_line(a.x, a.y, b.x, b.y, 2.0, with_alpha(col, 0.7));
    }
    for k in 1..4 {
        let f = k as f32 / 4.0;
        let a = p[0].lerp(p[3], f);
        let b = p[1].lerp(p[2], f);
        draw_line(a.x, a.y, b.x, b.y, 1.0, with_alpha(col, 0.12));
    }
}

pub fn draw_floor(cam: &Cam, y: f32) {
    let col = Color::new(1.0, 1.0, 1.0, 0.05);
    for i in -8..=8 {
        let t = i as f32;
        for (a, b) in [(vec3(t, y, -8.0), vec3(t, y, 8.0)), (vec3(-8.0, y, t), vec3(8.0, y, t))] {
            if let (Some((pa, _)), Some((pb, _))) = (cam.project(a), cam.project(b)) {
                draw_line(pa.x, pa.y, pb.x, pb.y, 1.0, col);
            }
        }
    }
}

/// A 2D arrow between projected points, with an optional label at the tip.
pub fn draw_arrow3(cam: &Cam, a: Vec3, b: Vec3, col: Color, label: &str) {
    if let (Some((pa, _)), Some((pb, _))) = (cam.project(a), cam.project(b)) {
        draw_line(pa.x, pa.y, pb.x, pb.y, 3.0, col);
        let d = (pb - pa).normalize_or_zero();
        let n = vec2(-d.y, d.x);
        draw_triangle(pb + d * 4.0, pb - d * 10.0 + n * 6.0, pb - d * 10.0 - n * 6.0, col);
        if !label.is_empty() {
            text(label, pb.x + d.x * 10.0 + 4.0, pb.y + d.y * 10.0 + 5.0, 18.0, col);
        }
    }
}

/// Text label anchored at a world point.
pub fn label3(cam: &Cam, p: Vec3, s: &str, size: f32, col: Color) {
    if let Some((q, _)) = cam.project(p) {
        let w = text_width(s, size);
        text(s, q.x - w / 2.0, q.y, size, col);
    }
}
