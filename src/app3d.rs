//! 3D mode: a ball-and-stick molecule, its reflection in a mirror plane,
//! and an automatic "try to superimpose" search.
//! Rendering is a small software projection with depth-sorted, shaded spheres.

use crate::ui::*;
use macroquad::prelude::*;

const ORIG_C: Vec3 = Vec3::new(-2.8, 0.0, 0.0);
const COPY_C: Vec3 = Vec3::new(2.8, 0.0, 0.0);

#[derive(Clone, Copy)]
struct Atom {
    p: Vec3,
    r: f32,
    col: Color,
    kind: u8,
}

struct Model {
    name: &'static str,
    blurb: &'static str,
    atoms: Vec<Atom>,
    bonds: Vec<(usize, usize)>,
    swappable: bool,
    legend: Vec<(Color, &'static str)>,
}

const C_GREY: Color = Color::new(0.45, 0.47, 0.52, 1.0);
const H_WHITE: Color = Color::new(0.93, 0.94, 0.96, 1.0);
const F_YELLOW: Color = Color::new(0.93, 0.85, 0.30, 1.0);
const CL_GREEN: Color = Color::new(0.30, 0.80, 0.35, 1.0);
const BR_RED: Color = Color::new(0.75, 0.22, 0.20, 1.0);
const N_BLUE: Color = Color::new(0.30, 0.50, 0.95, 1.0);

fn tetra() -> [Vec3; 4] {
    let s = 1.6 / 3f32.sqrt();
    [
        vec3(1.0, 1.0, 1.0) * s,
        vec3(1.0, -1.0, -1.0) * s,
        vec3(-1.0, 1.0, -1.0) * s,
        vec3(-1.0, -1.0, 1.0) * s,
    ]
}

fn atom(p: Vec3, r: f32, col: Color, kind: u8) -> Atom {
    Atom { p, r, col, kind }
}

fn star_molecule(subs: [(f32, Color, u8); 4]) -> (Vec<Atom>, Vec<(usize, usize)>) {
    let t = tetra();
    let mut atoms = vec![atom(Vec3::ZERO, 0.5, C_GREY, 0)];
    for (i, (r, c, k)) in subs.iter().enumerate() {
        atoms.push(atom(t[i], *r, *c, *k));
    }
    (atoms, (1..5).map(|i| (0, i)).collect())
}

fn models() -> Vec<Model> {
    let mut out = Vec::new();

    let (atoms, bonds) = star_molecule([
        (0.30, H_WHITE, 1),
        (0.40, F_YELLOW, 2),
        (0.48, CL_GREEN, 3),
        (0.56, BR_RED, 4),
    ]);
    out.push(Model {
        name: "CHFClBr (chiral carbon)",
        blurb: "Four different groups on one carbon: a stereocentre. Its mirror image is a different molecule.",
        atoms,
        bonds,
        swappable: true,
        legend: vec![(H_WHITE, "H"), (F_YELLOW, "F"), (CL_GREEN, "Cl"), (BR_RED, "Br")],
    });

    let (atoms, bonds) = star_molecule([
        (0.30, H_WHITE, 1),
        (0.30, H_WHITE, 1),
        (0.48, CL_GREEN, 3),
        (0.56, BR_RED, 4),
    ]);
    out.push(Model {
        name: "CH2ClBr (achiral)",
        blurb: "Two identical groups give it a mirror plane, so it matches its reflection.",
        atoms,
        bonds,
        swappable: false,
        legend: vec![(H_WHITE, "H"), (CL_GREEN, "Cl"), (BR_RED, "Br")],
    });

    // Helix: right-handed spiral of beads.
    let n = 11;
    let atoms: Vec<Atom> = (0..n)
        .map(|i| {
            let a = i as f32 * 0.85;
            let y = -2.0 + i as f32 * 0.4;
            let col = Color::new(0.35 + 0.05 * i as f32, 0.55, 0.95 - 0.04 * i as f32, 1.0);
            atom(vec3(1.0 * a.cos(), y, 1.0 * a.sin()), 0.3, col, 1)
        })
        .collect();
    out.push(Model {
        name: "Helix",
        blurb: "A spiral staircase twists one way. Its reflection twists the other way: a left vs right helix.",
        atoms,
        bonds: (0..n - 1).map(|i| (i, i + 1)).collect(),
        swappable: false,
        legend: vec![],
    });

    // Twisted "propeller": three blades on an axis, all tilted the same way.
    let mut atoms = vec![atom(vec3(0.0, 0.0, 0.0), 0.45, C_GREY, 0)];
    let mut bonds = Vec::new();
    for k in 0..3 {
        let a = k as f32 * std::f32::consts::TAU / 3.0;
        let dir = vec3(a.cos(), 0.0, a.sin());
        let tilt = vec3(-a.sin(), 0.0, a.cos());
        let base = atoms.len();
        atoms.push(atom(dir * 1.3, 0.35, N_BLUE, 1));
        atoms.push(atom(dir * 2.1 + tilt * 0.55 + vec3(0.0, 0.5, 0.0), 0.28, F_YELLOW, 2));
        bonds.push((0, base));
        bonds.push((base, base + 1));
    }
    out.push(Model {
        name: "Propeller",
        blurb: "Three identical twisted blades: lots of rotation symmetry, but no mirror plane.",
        atoms,
        bonds,
        swappable: false,
        legend: vec![],
    });

    // Achiral comparison: same propeller with untwisted, flat blades.
    let mut atoms = vec![atom(vec3(0.0, 0.0, 0.0), 0.45, C_GREY, 0)];
    let mut bonds = Vec::new();
    for k in 0..3 {
        let a = k as f32 * std::f32::consts::TAU / 3.0;
        let dir = vec3(a.cos(), 0.0, a.sin());
        let base = atoms.len();
        atoms.push(atom(dir * 1.3, 0.35, N_BLUE, 1));
        atoms.push(atom(dir * 2.1 + vec3(0.0, 0.5, 0.0), 0.28, F_YELLOW, 2));
        bonds.push((0, base));
        bonds.push((base, base + 1));
    }
    out.push(Model {
        name: "Flat fan (achiral)",
        blurb: "The same fan with untwisted blades has mirror planes: it is achiral.",
        atoms,
        bonds,
        swappable: false,
        legend: vec![],
    });

    out
}

fn reflect_x(v: Vec3) -> Vec3 {
    vec3(-v.x, v.y, v.z)
}

/// Mismatch between two labelled point clouds (symmetric nearest same-kind distance).
fn fit_cost(o: &[(Vec3, u8)], c: &[(Vec3, u8)], q: Quat) -> f32 {
    let rc: Vec<(Vec3, u8)> = c.iter().map(|&(p, k)| (q * p, k)).collect();
    let nearest = |p: Vec3, k: u8, set: &[(Vec3, u8)]| {
        set.iter()
            .filter(|(_, kk)| *kk == k)
            .map(|(s, _)| s.distance_squared(p))
            .fold(f32::MAX, f32::min)
    };
    rc.iter().map(|&(p, k)| nearest(p, k, o)).sum::<f32>() + o.iter().map(|&(p, k)| nearest(p, k, &rc)).sum::<f32>()
}

/// Search for the rotation that best overlays `c` onto `o`. Returns (rotation, rms distance).
fn best_rotation(o: &[(Vec3, u8)], c: &[(Vec3, u8)]) -> (Quat, f32) {
    let mut seed: u32 = 12345;
    let mut rnd = || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) as f32 / (1u32 << 24) as f32
    };
    let axes = [Vec3::X, Vec3::Y, Vec3::Z, -Vec3::X, -Vec3::Y, -Vec3::Z];
    let mut starts = vec![Quat::IDENTITY];
    for _ in 0..80 {
        // Uniform random rotation (Shoemake).
        let (u1, u2, u3) = (rnd(), rnd() * std::f32::consts::TAU, rnd() * std::f32::consts::TAU);
        let (a, b) = ((1.0 - u1).sqrt(), u1.sqrt());
        starts.push(Quat::from_xyzw(a * u2.sin(), a * u2.cos(), b * u3.sin(), b * u3.cos()).normalize());
    }
    let mut best = (Quat::IDENTITY, f32::MAX);
    for s in starts {
        let mut q = s;
        let mut cost = fit_cost(o, c, q);
        let mut step = 0.5;
        while step > 2e-4 {
            let mut improved = false;
            for ax in axes {
                let qn = (Quat::from_axis_angle(ax, step) * q).normalize();
                let cn = fit_cost(o, c, qn);
                if cn < cost {
                    q = qn;
                    cost = cn;
                    improved = true;
                }
            }
            if !improved {
                step *= 0.5;
            }
        }
        if cost < best.1 {
            best = (q, cost);
        }
    }
    let n = (o.len() + c.len()) as f32;
    (best.0, (best.1 / n).sqrt())
}

#[derive(PartialEq, Clone, Copy)]
enum Drag3 {
    None,
    Orig,
    Copy,
    Camera,
}

struct Overlay {
    start_q: Quat,
    target_q: Quat,
    t: f32,
}

pub struct App3D {
    models: Vec<Model>,
    cur: usize,
    q_orig: Quat,
    q_copy: Quat,
    yaw: f32,
    pitch: f32,
    dist: f32,
    drag: Drag3,
    last: Vec2,
    overlay: Option<Overlay>,
    show_mirror: bool,
    swap: bool,
    verdict: Option<(bool, f32)>,
}

enum Prim {
    Ball { c: Vec2, r: f32, col: Color, z: f32 },
    Stick { a: Vec2, b: Vec2, w: f32, col: Color, z: f32 },
    Gap { a: Vec2, b: Vec2, z: f32 },
}

impl Prim {
    fn z(&self) -> f32 {
        match self {
            Prim::Ball { z, .. } | Prim::Stick { z, .. } | Prim::Gap { z, .. } => *z,
        }
    }
}

impl App3D {
    pub fn new() -> App3D {
        App3D {
            models: models(),
            cur: 0,
            q_orig: Quat::from_rotation_y(0.4) * Quat::from_rotation_x(-0.2),
            q_copy: Quat::IDENTITY,
            yaw: -0.5,
            pitch: -0.28,
            dist: 13.0,
            drag: Drag3::None,
            last: Vec2::ZERO,
            overlay: None,
            show_mirror: true,
            swap: false,
            verdict: None,
        }
    }

    fn model(&self) -> &Model {
        &self.models[self.cur]
    }

    fn view_rot(&self) -> Quat {
        Quat::from_rotation_x(self.pitch) * Quat::from_rotation_y(self.yaw)
    }

    fn focal(&self) -> f32 {
        (screen_width() - SIDEBAR_W).min(screen_height()) * 1.25
    }

    fn center2d(&self) -> Vec2 {
        vec2(SIDEBAR_W + (screen_width() - SIDEBAR_W) / 2.0, screen_height() / 2.0)
    }

    /// World -> (screen point, view depth). None if behind the camera.
    fn project(&self, p: Vec3) -> Option<(Vec2, f32)> {
        let v = self.view_rot() * p;
        let z = v.z + self.dist;
        if z < 0.5 {
            return None;
        }
        let f = self.focal() / z;
        let c = self.center2d();
        Some((vec2(c.x + v.x * f, c.y - v.y * f), z))
    }

    /// Local (centred) atom positions of the original, after its rotation.
    fn orig_local(&self) -> Vec<Vec3> {
        self.model().atoms.iter().map(|a| self.q_orig * a.p).collect()
    }

    /// Local positions of the mirror image copy (before q_copy / overlay rotation).
    fn copy_local_unrotated(&self) -> Vec<Vec3> {
        let mut pts: Vec<Vec3> = self.orig_local().into_iter().map(reflect_x).collect();
        if self.swap && self.model().swappable && pts.len() > 2 {
            pts.swap(1, 2);
        }
        pts
    }

    fn copy_pose(&self) -> (Vec3, Quat) {
        match &self.overlay {
            None => (COPY_C, self.q_copy),
            Some(o) => {
                let s = o.t * o.t * (3.0 - 2.0 * o.t); // smoothstep
                (COPY_C.lerp(ORIG_C, s), o.start_q.slerp(o.target_q, s))
            }
        }
    }

    fn superimpose(&mut self) {
        let kinds: Vec<u8> = self.model().atoms.iter().map(|a| a.kind).collect();
        let o: Vec<(Vec3, u8)> = self.orig_local().into_iter().zip(kinds.iter().copied()).collect();
        let c: Vec<(Vec3, u8)> = self
            .copy_local_unrotated()
            .into_iter()
            .map(|p| self.q_copy * p)
            .zip(kinds.iter().copied())
            .collect();
        let (q_fit, rms) = best_rotation(&o, &c);
        self.overlay = Some(Overlay { start_q: self.q_copy, target_q: q_fit * self.q_copy, t: 0.0 });
        self.verdict = Some((rms < 0.05, rms));
    }

    fn separate(&mut self) {
        self.overlay = None;
    }

    fn set_model(&mut self, i: usize) {
        self.cur = i % self.models.len();
        self.overlay = None;
        self.verdict = None;
        self.swap = false;
        self.q_copy = Quat::IDENTITY;
    }

    /// Rotation in world space matching a screen-space mouse drag.
    fn drag_rotation(&self, d: Vec2) -> Quat {
        let k = 0.01;
        let rv = Quat::from_rotation_x(-d.y * k) * Quat::from_rotation_y(-d.x * k);
        let v = self.view_rot();
        v.inverse() * rv * v
    }

    fn handle_input(&mut self) {
        let m = mouse();
        if is_mouse_button_pressed(MouseButton::Left) && in_canvas(m) {
            let split = self.project(Vec3::ZERO).map_or(self.center2d().x, |(p, _)| p.x);
            self.drag = if m.x < split { Drag3::Orig } else { Drag3::Copy };
            if self.overlay.is_some() {
                self.separate();
            }
            self.verdict = None;
        }
        if is_mouse_button_pressed(MouseButton::Right) && in_canvas(m) {
            self.drag = Drag3::Camera;
        }
        if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
            let d = m - self.last;
            match self.drag {
                Drag3::Orig => self.q_orig = (self.drag_rotation(d) * self.q_orig).normalize(),
                Drag3::Copy => self.q_copy = (self.drag_rotation(d) * self.q_copy).normalize(),
                Drag3::Camera => {
                    self.yaw -= d.x * 0.008;
                    self.pitch = (self.pitch - d.y * 0.008).clamp(-1.4, 1.4);
                }
                Drag3::None => {}
            }
        } else {
            self.drag = Drag3::None;
        }
        self.last = m;

        let dt = get_frame_time();
        if is_key_down(KeyCode::Left) {
            self.yaw += dt;
        }
        if is_key_down(KeyCode::Right) {
            self.yaw -= dt;
        }
        if is_key_down(KeyCode::Up) {
            self.pitch = (self.pitch - dt).clamp(-1.4, 1.4);
        }
        if is_key_down(KeyCode::Down) {
            self.pitch = (self.pitch + dt).clamp(-1.4, 1.4);
        }
        let wheel = mouse_wheel().1;
        if wheel != 0.0 && in_canvas(m) {
            self.dist = (self.dist * if wheel > 0.0 { 0.92 } else { 1.08 }).clamp(6.0, 30.0);
        }
        if is_key_pressed(KeyCode::Space) {
            if self.overlay.is_some() {
                self.separate();
            } else {
                self.superimpose();
            }
        }
        if is_key_pressed(KeyCode::Tab) {
            self.set_model(self.cur + 1);
        }
        if let Some(o) = &mut self.overlay {
            o.t = (o.t + dt / 1.4).min(1.0);
        }
    }

    fn push_molecule(&self, prims: &mut Vec<Prim>, center: Vec3, local: &[Vec3], alpha: f32) {
        let model = self.model();
        let proj: Vec<Option<(Vec2, f32)>> = local.iter().map(|&p| self.project(center + p)).collect();
        for &(i, j) in &model.bonds {
            if let (Some((a, za)), Some((b, zb))) = (proj[i], proj[j]) {
                let z = (za + zb) / 2.0;
                let w = 0.16 * self.focal() / z;
                prims.push(Prim::Stick { a, b, w, col: Color::new(0.62, 0.65, 0.70, alpha), z: z + 0.01 });
            }
        }
        for (k, a) in model.atoms.iter().enumerate() {
            if let Some((c, z)) = proj[k] {
                let r = a.r * self.focal() / z;
                prims.push(Prim::Ball { c, r, col: with_alpha(a.col, alpha), z });
            }
        }
    }

    fn draw_ball(c: Vec2, r: f32, col: Color) {
        let dark = Color::new(col.r * 0.28, col.g * 0.28, col.b * 0.32, col.a);
        draw_circle(c.x, c.y, r, dark);
        let steps = 9;
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

    fn draw_mirror_plane(&self) {
        let s = 3.2;
        let corners = [vec3(0.0, -s, -s), vec3(0.0, -s, s), vec3(0.0, s, s), vec3(0.0, s, -s)];
        let p: Vec<Vec2> = match corners.iter().map(|&c| self.project(c).map(|x| x.0)).collect::<Option<Vec<_>>>() {
            Some(p) => p,
            None => return,
        };
        let fill = with_alpha(MIRROR, 0.13);
        draw_triangle(p[0], p[1], p[2], fill);
        draw_triangle(p[0], p[2], p[3], fill);
        for i in 0..4 {
            let (a, b) = (p[i], p[(i + 1) % 4]);
            draw_line(a.x, a.y, b.x, b.y, 2.0, with_alpha(MIRROR, 0.7));
        }
        // A few faint "glass" streaks.
        for k in 1..4 {
            let f = k as f32 / 4.0;
            let a = p[0].lerp(p[3], f);
            let b = p[1].lerp(p[2], f);
            draw_line(a.x, a.y, b.x, b.y, 1.0, with_alpha(MIRROR, 0.12));
        }
    }

    fn draw_floor(&self) {
        let y = -3.2;
        let col = Color::new(1.0, 1.0, 1.0, 0.05);
        for i in -8..=8 {
            let t = i as f32;
            for (a, b) in [(vec3(t, y, -8.0), vec3(t, y, 8.0)), (vec3(-8.0, y, t), vec3(8.0, y, t))] {
                if let (Some((pa, _)), Some((pb, _))) = (self.project(a), self.project(b)) {
                    draw_line(pa.x, pa.y, pb.x, pb.y, 1.0, col);
                }
            }
        }
    }

    pub fn update_and_draw(&mut self) {
        self.handle_input();
        self.draw_floor();

        let orig = self.orig_local();
        let (cc, cq) = self.copy_pose();
        let copy: Vec<Vec3> = self.copy_local_unrotated().into_iter().map(|p| cq * p).collect();
        let overlaid = self.overlay.as_ref().map_or(false, |o| o.t >= 1.0);
        let in_flight = self.overlay.is_some();

        // Split primitives by side of the mirror relative to the camera so the
        // translucent plane is layered correctly.
        let cam = self.view_rot().inverse() * vec3(0.0, 0.0, -self.dist);
        let mut back = Vec::new();
        let mut front = Vec::new();
        {
            let (orig_side, copy_side) = (ORIG_C.x.signum(), cc.x.signum());
            let cam_side = cam.x.signum();
            let target = |side: f32| side == cam_side || in_flight;
            let mut po = Vec::new();
            self.push_molecule(&mut po, ORIG_C, &orig, 1.0);
            let mut pc = Vec::new();
            self.push_molecule(&mut pc, cc, &copy, if overlaid { 0.55 } else { 1.0 });
            if target(orig_side) { front.extend(po) } else { back.extend(po) }
            if target(copy_side) { front.extend(pc) } else { back.extend(pc) }
        }

        // Mismatch markers once overlaid.
        if overlaid && !self.verdict.map_or(true, |v| v.0) {
            let kinds: Vec<u8> = self.model().atoms.iter().map(|a| a.kind).collect();
            for (i, &p) in copy.iter().enumerate() {
                let best = orig
                    .iter()
                    .zip(&kinds)
                    .filter(|(_, &k)| k == kinds[i])
                    .map(|(&o, _)| o)
                    .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
                if let Some(o) = best {
                    if o.distance(p) > 0.25 {
                        if let (Some((a, za)), Some((b, zb))) = (self.project(cc + p), self.project(ORIG_C + o)) {
                            front.push(Prim::Gap { a, b, z: za.min(zb) - 1.0 });
                        }
                    }
                }
            }
        }

        for list in [&mut back, &mut front] {
            list.sort_by(|a, b| b.z().total_cmp(&a.z()));
        }
        let draw_list = |list: &Vec<Prim>| {
            for p in list {
                match *p {
                    Prim::Ball { c, r, col, .. } => App3D::draw_ball(c, r, col),
                    Prim::Stick { a, b, w, col, .. } => {
                        draw_line(a.x, a.y, b.x, b.y, w, Color::new(0.15, 0.16, 0.2, col.a));
                        draw_line(a.x, a.y, b.x, b.y, w * 0.7, col);
                    }
                    Prim::Gap { a, b, .. } => dashed_line(a, b, 5.0, 2.5, Color::new(1.0, 0.3, 0.3, 0.95)),
                }
            }
        };
        draw_list(&back);
        if self.show_mirror {
            self.draw_mirror_plane();
        }
        draw_list(&front);

        // Captions.
        let label = |p: Vec3, s: &str, col: Color| {
            if let Some((q, _)) = self.project(p) {
                let w = measure_text(s, None, 20, 1.0).width;
                draw_text(s, q.x - w / 2.0, q.y, 20.0, col);
            }
        };
        if !in_flight {
            label(ORIG_C + vec3(0.0, -3.0, 0.0), "Original", SAME);
            let copy_name = if self.swap && self.model().swappable { "Image (2 groups swapped)" } else if self.q_copy == Quat::IDENTITY { "Mirror image" } else { "Mirror image (rotated)" };
            label(COPY_C + vec3(0.0, -3.0, 0.0), copy_name, OPP);
        }
        if let (Some((ok, rms)), true) = (self.verdict, overlaid) {
            let (msg, col) = if ok {
                ("Perfect overlap: superimposable", GOOD)
            } else {
                ("Best possible fit still misses: NOT superimposable", OPP)
            };
            let w = measure_text(msg, None, 24, 1.0).width;
            let c = self.center2d();
            draw_text(msg, c.x - w / 2.0, screen_height() - 60.0, 24.0, col);
            let sub = format!("best-fit RMS distance = {rms:.2} (dashed red = atoms that can't line up)");
            let w = measure_text(&sub, None, 18, 1.0).width;
            draw_text(&sub, c.x - w / 2.0, screen_height() - 34.0, 18.0, DIM);
        }
    }

    pub fn sidebar(&mut self, ui: &mut Ui) {
        ui.heading("Model");
        if let Some(i) = ui.row(&["< Prev", "Next > [Tab]"], None) {
            let n = self.models.len();
            self.set_model(if i == 0 { self.cur + n - 1 } else { self.cur + 1 });
        }
        let (name, blurb, swappable) = (self.model().name, self.model().blurb, self.model().swappable);
        ui.label(name, 20.0, TEXT);
        ui.wrapped(blurb, 16.0, DIM);
        let legend = self.model().legend.clone();
        if !legend.is_empty() {
            ui.space(2.0);
            for (c, n) in legend {
                ui.swatch(c, n);
            }
        }

        ui.heading("Experiment");
        if self.overlay.is_some() {
            if ui.button("Separate [Space]") {
                self.separate();
            }
        } else if ui.button("Try to superimpose [Space]") {
            self.superimpose();
        }
        if let Some((ok, rms)) = self.verdict {
            if ok {
                ui.label("Superimposable -> ACHIRAL", 18.0, GOOD);
            } else {
                ui.label("Not superimposable -> CHIRAL", 18.0, OPP);
            }
            ui.label(&format!("(best-fit RMS = {rms:.3})"), 15.0, DIM);
        }
        if ui.button("Reset rotations") {
            self.q_orig = Quat::from_rotation_y(0.4) * Quat::from_rotation_x(-0.2);
            self.q_copy = Quat::IDENTITY;
            self.overlay = None;
            self.verdict = None;
        }
        ui.toggle("Show mirror plane", &mut self.show_mirror);
        if swappable {
            let before = self.swap;
            ui.toggle("Swap H and F on the image", &mut self.swap);
            if before != self.swap {
                self.overlay = None;
                self.verdict = None;
            }
            ui.wrapped("Swapping any two groups converts one enantiomer into the other.", 15.0, DIM);
        }

        ui.heading("Controls");
        for line in [
            "L-drag left side: rotate original",
            "   (its reflection follows live)",
            "L-drag right side: rotate the image",
            "R-drag / arrow keys: orbit camera",
            "Wheel: zoom",
        ] {
            ui.label(line, 15.0, DIM);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn molecule_chirality() {
        let expect = [true, false, true, true, false];
        for (m, chiral) in models().iter().zip(expect) {
            let q = Quat::from_rotation_y(0.7) * Quat::from_rotation_x(0.3);
            let o: Vec<(Vec3, u8)> = m.atoms.iter().map(|a| (q * a.p, a.kind)).collect();
            let c: Vec<(Vec3, u8)> = o.iter().map(|&(p, k)| (Quat::from_rotation_z(1.1) * reflect_x(p), k)).collect();
            let (_, rms) = best_rotation(&o, &c);
            assert_eq!(rms >= 0.05, chiral, "{} rms={rms}", m.name);
        }
    }
}
