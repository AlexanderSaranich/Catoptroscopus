//! 3D mode: ball-and-stick models (molecules, a helix, knots), their reflection
//! in a mirror plane, an automatic "try to superimpose" search, and live
//! pseudoscalar readouts (signed volume / R-S, torsion, writhe, point group).

use crate::render3d::*;
use crate::symmetry3d::*;
use crate::ui::*;
use macroquad::prelude::*;
use std::f32::consts::TAU;

const ORIG_C: Vec3 = Vec3::new(-2.8, 0.0, 0.0);
const COPY_C: Vec3 = Vec3::new(2.8, 0.0, 0.0);

#[derive(Clone, Copy)]
struct Atom {
    p: Vec3,
    r: f32,
    col: Color,
    kind: u8,
}

#[derive(Clone, Copy)]
enum Info {
    /// Tetrahedral centre: substituent atom indices by CIP priority (highest first).
    Stereo { prio: [usize; 4], names: [&'static str; 4] },
    NotStereo,
    Helix { a: f32, b: f32 },
    Knot { jones: &'static str, jones_mirror: &'static str, amphichiral: bool },
    Plain,
}

struct Model {
    name: &'static str,
    blurb: &'static str,
    atoms: Vec<Atom>,
    bonds: Vec<(usize, usize)>,
    swappable: bool,
    legend: Vec<(Color, &'static str)>,
    info: Info,
    group: PointGroup,
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

fn rainbow(t: f32) -> Color {
    let h = t * TAU;
    Color::new(0.55 + 0.4 * h.cos(), 0.55 + 0.4 * (h - 2.1).cos(), 0.6 + 0.35 * (h + 2.1).cos(), 1.0)
}

fn closed_curve(f: impl Fn(f32) -> Vec3, n: usize, scale: f32) -> (Vec<Atom>, Vec<(usize, usize)>) {
    let atoms = (0..n)
        .map(|i| {
            let t = TAU * i as f32 / n as f32;
            atom(f(t) * scale, 0.12, rainbow(i as f32 / n as f32), 1)
        })
        .collect();
    (atoms, (0..n).map(|i| (i, (i + 1) % n)).collect())
}

fn model(
    name: &'static str,
    blurb: &'static str,
    (atoms, bonds): (Vec<Atom>, Vec<(usize, usize)>),
    swappable: bool,
    legend: Vec<(Color, &'static str)>,
    info: Info,
) -> Model {
    let pts: Vec<(Vec3, u8)> = atoms.iter().map(|a| (a.p, a.kind)).collect();
    let group = point_group(&pts);
    Model { name, blurb, atoms, bonds, swappable, legend, info, group }
}

fn models() -> Vec<Model> {
    let mut out = Vec::new();

    out.push(model(
        "CHFClBr (chiral carbon)",
        "Four different groups on one carbon: a stereocentre. Its mirror image is a different molecule.",
        star_molecule([(0.30, H_WHITE, 1), (0.40, F_YELLOW, 2), (0.48, CL_GREEN, 3), (0.56, BR_RED, 4)]),
        true,
        vec![(H_WHITE, "H"), (F_YELLOW, "F"), (CL_GREEN, "Cl"), (BR_RED, "Br")],
        Info::Stereo { prio: [4, 3, 2, 1], names: ["Br", "Cl", "F", "H"] },
    ));

    out.push(model(
        "CH₂ClBr (achiral)",
        "Two identical groups give it a mirror plane, so it matches its reflection.",
        star_molecule([(0.30, H_WHITE, 1), (0.30, H_WHITE, 1), (0.48, CL_GREEN, 3), (0.56, BR_RED, 4)]),
        false,
        vec![(H_WHITE, "H"), (CL_GREEN, "Cl"), (BR_RED, "Br")],
        Info::NotStereo,
    ));

    // Right-handed helix about the y axis: (a cos t, b t, −a sin t).
    let n = 11;
    let (a, step, rise) = (1.0f32, 0.85f32, 0.4f32);
    let atoms: Vec<Atom> = (0..n)
        .map(|i| {
            let t = i as f32 * step;
            let col = Color::new(0.35 + 0.05 * i as f32, 0.55, 0.95 - 0.04 * i as f32, 1.0);
            atom(vec3(a * t.cos(), -2.0 + i as f32 * rise, -a * t.sin()), 0.3, col, 1)
        })
        .collect();
    out.push(model(
        "Helix",
        "A spiral staircase twists one way. Its reflection twists the other way: a right vs left helix.",
        (atoms, (0..n - 1).map(|i| (i, i + 1)).collect()),
        false,
        vec![],
        Info::Helix { a, b: rise / step },
    ));

    let fan = |twist: f32| {
        let mut atoms = vec![atom(Vec3::ZERO, 0.45, C_GREY, 0)];
        let mut bonds = Vec::new();
        for k in 0..3 {
            let a = k as f32 * TAU / 3.0;
            let dir = vec3(a.cos(), 0.0, a.sin());
            let tilt = vec3(-a.sin(), 0.0, a.cos());
            let base = atoms.len();
            atoms.push(atom(dir * 1.3, 0.35, N_BLUE, 1));
            atoms.push(atom(dir * 2.1 + tilt * twist + vec3(0.0, 0.5, 0.0), 0.28, F_YELLOW, 2));
            bonds.push((0, base));
            bonds.push((base, base + 1));
        }
        (atoms, bonds)
    };
    out.push(model(
        "Propeller",
        "Three identical twisted blades: rotation symmetry, but no mirror plane.",
        fan(0.55),
        false,
        vec![],
        Info::Plain,
    ));
    out.push(model(
        "Flat fan (achiral)",
        "The same fan with untwisted blades has mirror planes: it is achiral.",
        fan(0.0),
        false,
        vec![],
        Info::Plain,
    ));

    out.push(model(
        "Trefoil knot",
        "The simplest knot is chiral: no deformation, let alone rotation, turns it into its mirror image.",
        closed_curve(|t| vec3(t.sin() + 2.0 * (2.0 * t).sin(), t.cos() - 2.0 * (2.0 * t).cos(), -(3.0 * t).sin()), 96, 0.55),
        false,
        vec![],
        Info::Knot { jones: "t + t³ − t⁴", jones_mirror: "t⁻¹ + t⁻³ − t⁻⁴", amphichiral: false },
    ));
    out.push(model(
        "Figure-eight knot",
        "Amphichiral: it can be deformed into its mirror image, even though this rigid shape can't be rotated into it.",
        closed_curve(
            |t| {
                let r = 2.0 + (2.0 * t).cos();
                vec3(r * (3.0 * t).cos(), r * (3.0 * t).sin(), (4.0 * t).sin())
            },
            96,
            0.55,
        ),
        false,
        vec![],
        Info::Knot { jones: "t² − t + 1 − t⁻¹ + t⁻²", jones_mirror: "t⁻² − t⁻¹ + 1 − t + t²", amphichiral: true },
    ));

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
    let n_starts = if o.len() > 30 { 30 } else { 80 };
    let mut starts = vec![Quat::IDENTITY];
    for _ in 0..n_starts {
        // Uniform random rotation (Shoemake).
        let (u1, u2, u3) = (rnd(), rnd() * TAU, rnd() * TAU);
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
    cam: Cam,
    drag: Drag3,
    last: Vec2,
    overlay: Option<Overlay>,
    show_mirror: bool,
    swap: bool,
    verdict: Option<(bool, f32)>,
}

fn default_q() -> Quat {
    Quat::from_rotation_y(0.4) * Quat::from_rotation_x(-0.2)
}

impl App3D {
    pub fn new() -> App3D {
        App3D {
            models: models(),
            cur: 0,
            q_orig: default_q(),
            q_copy: Quat::IDENTITY,
            cam: Cam::new(-0.5, 0.28, 13.0),
            drag: Drag3::None,
            last: Vec2::ZERO,
            overlay: None,
            show_mirror: true,
            swap: false,
            verdict: None,
        }
    }

    pub fn model_index(&self, name: &str) -> usize {
        self.models.iter().position(|m| m.name.starts_with(name)).unwrap_or(0)
    }

    fn model(&self) -> &Model {
        &self.models[self.cur]
    }

    fn orig_local(&self) -> Vec<Vec3> {
        self.model().atoms.iter().map(|a| self.q_orig * a.p).collect()
    }

    /// Local positions of the mirror image (before q_copy / overlay rotation).
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
                let s = o.t * o.t * (3.0 - 2.0 * o.t);
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

    pub fn set_model(&mut self, i: usize) {
        self.cur = i % self.models.len();
        self.overlay = None;
        self.verdict = None;
        self.swap = false;
        self.q_copy = Quat::IDENTITY;
    }

    fn handle_input(&mut self) {
        let m = mouse();
        if is_mouse_button_pressed(MouseButton::Left) && in_canvas(m) {
            let split = self.cam.project(Vec3::ZERO).map_or(self.cam.center2d().x, |(p, _)| p.x);
            self.drag = if m.x < split { Drag3::Orig } else { Drag3::Copy };
            self.overlay = None;
            self.verdict = None;
        }
        if is_mouse_button_pressed(MouseButton::Right) && in_canvas(m) {
            self.drag = Drag3::Camera;
        }
        if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
            let d = m - self.last;
            match self.drag {
                Drag3::Orig => self.q_orig = (self.cam.drag_rotation(d) * self.q_orig).normalize(),
                Drag3::Copy => self.q_copy = (self.cam.drag_rotation(d) * self.q_copy).normalize(),
                Drag3::Camera => self.cam.orbit(d),
                Drag3::None => {}
            }
        } else {
            self.drag = Drag3::None;
        }
        self.last = m;
        self.cam.keys_and_wheel(true);

        if is_key_pressed(KeyCode::Space) {
            if self.overlay.is_some() {
                self.overlay = None;
            } else {
                self.superimpose();
            }
        }
        if is_key_pressed(KeyCode::Tab) {
            self.set_model(self.cur + 1);
        }
        if let Some(o) = &mut self.overlay {
            o.t = (o.t + get_frame_time() / 1.4).min(1.0);
        }
    }

    fn push_molecule(&self, prims: &mut Vec<Prim>, center: Vec3, local: &[Vec3], alpha: f32) {
        let model = self.model();
        for &(i, j) in &model.bonds {
            let w = if model.atoms.len() > 40 { 0.12 } else { 0.16 };
            push_stick(&self.cam, prims, center + local[i], center + local[j], w, Color::new(0.62, 0.65, 0.70, alpha));
        }
        for (k, a) in model.atoms.iter().enumerate() {
            push_ball(&self.cam, prims, center + local[k], a.r, with_alpha(a.col, alpha));
        }
    }

    pub fn update_and_draw(&mut self) {
        self.handle_input();
        draw_floor(&self.cam, -3.2);

        let orig = self.orig_local();
        let (cc, cq) = self.copy_pose();
        let copy: Vec<Vec3> = self.copy_local_unrotated().into_iter().map(|p| cq * p).collect();
        let overlaid = self.overlay.as_ref().map_or(false, |o| o.t >= 1.0);
        let in_flight = self.overlay.is_some();

        // Layer relative to the translucent mirror plane x = 0.
        let cam_side = self.cam.eye().x.signum();
        let mut back = Vec::new();
        let mut front = Vec::new();
        let mut po = Vec::new();
        self.push_molecule(&mut po, ORIG_C, &orig, 1.0);
        let mut pc = Vec::new();
        self.push_molecule(&mut pc, cc, &copy, if overlaid { 0.55 } else { 1.0 });
        if ORIG_C.x.signum() == cam_side || in_flight { front.extend(po) } else { back.extend(po) }
        if cc.x.signum() == cam_side || in_flight { front.extend(pc) } else { back.extend(pc) }

        if overlaid && !self.verdict.map_or(true, |v| v.0) {
            let kinds: Vec<u8> = self.model().atoms.iter().map(|a| a.kind).collect();
            let stride = if copy.len() > 40 { 4 } else { 1 };
            for (i, &p) in copy.iter().enumerate().step_by(stride) {
                let best = orig
                    .iter()
                    .zip(&kinds)
                    .filter(|(_, &k)| k == kinds[i])
                    .map(|(&o, _)| o)
                    .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)));
                if let Some(o) = best {
                    if o.distance(p) > 0.25 {
                        push_dash(&self.cam, &mut front, cc + p, ORIG_C + o, Color::new(1.0, 0.3, 0.3, 0.95));
                    }
                }
            }
        }

        draw_prims(&mut back);
        if self.show_mirror {
            let s = 3.2;
            draw_quad(&self.cam, [vec3(0.0, -s, -s), vec3(0.0, -s, s), vec3(0.0, s, s), vec3(0.0, s, -s)], MIRROR);
        }
        draw_prims(&mut front);

        if !in_flight {
            label3(&self.cam, ORIG_C + vec3(0.0, -3.0, 0.0), "Original", 18.0, SAME);
            let copy_name = if self.swap && self.model().swappable {
                "Image (H and F swapped)"
            } else if self.q_copy == Quat::IDENTITY {
                "Mirror image"
            } else {
                "Mirror image (rotated)"
            };
            label3(&self.cam, COPY_C + vec3(0.0, -3.0, 0.0), copy_name, 18.0, OPP);
        }
        if let (Some((ok, rms)), true) = (self.verdict, overlaid) {
            let (msg, col) = if ok {
                ("Perfect overlap: superimposable", GOOD)
            } else {
                ("Best possible fit still misses: NOT superimposable", OPP)
            };
            let c = self.cam.center2d();
            let w = text_width(msg, 24.0);
            text(msg, c.x - w / 2.0, screen_height() - 58.0, 24.0, col);
            let sub = format!("best-fit RMS distance = {rms:.2}   (dashed red = atoms that can't line up)");
            let w = text_width(&sub, 16.0);
            text(&sub, c.x - w / 2.0, screen_height() - 34.0, 16.0, DIM);
        }

        self.draw_math(&orig, &copy);
        hint("L-drag left: rotate original (reflection follows)  ·  L-drag right: rotate image  ·  R-drag / arrows: orbit  ·  Wheel: zoom  ·  Space: superimpose");
    }

    fn draw_math(&self, orig: &[Vec3], copy: &[Vec3]) {
        let m = self.model();
        let g = &m.group;
        let mut lines: Vec<(String, Color)> = vec![
            (format!("Point group: {}   ({} proper, {} improper ops)", g.name, g.proper, g.improper), TEXT),
            if g.chiral() {
                ("Only rotations in G  →  CHIRAL".to_string(), OPP)
            } else {
                let what = if g.has_mirror { "a mirror plane σ" } else if g.has_inversion { "an inversion centre i" } else { "an improper axis Sn" };
                (format!("G contains {what}  →  ACHIRAL"), SAME)
            },
            ("Mirror: (x, y, z) → (−x, y, z),  det = −1".to_string(), DIM),
        ];
        match m.info {
            Info::Stereo { prio, names } => {
                lines.push((format!("Priorities: {} > {} > {} > {}", names[0], names[1], names[2], names[3]), DIM));
                lines.push(("χ = sign det[v₁−v₄, v₂−v₄, v₃−v₄]".to_string(), TEXT));
                for (label, pts, col) in [("Original", orig, SAME), ("Image", copy, OPP)] {
                    let chi = signed_volume(pts[prio[0]], pts[prio[1]], pts[prio[2]], pts[prio[3]]);
                    let rs = if chi < 0.0 { "R" } else { "S" };
                    lines.push((format!("{label}:  χ = {}  →  ({rs})", pm1(chi)), col));
                }
            }
            Info::NotStereo => {
                lines.push(("Two identical groups: not a stereocentre,".to_string(), DIM));
                lines.push(("so there is no R/S label.".to_string(), DIM));
            }
            Info::Helix { a, b } => {
                let tau = b / (a * a + b * b);
                lines.push((format!("Torsion τ = b/(a² + b²),  a = {a:.2}, b = {b:.2}"), TEXT));
                for (label, pts, col) in [("Original", orig, SAME), ("Image", copy, OPP)] {
                    let s = torsion_sign(pts);
                    let hand = if s > 0.0 { "right-handed" } else { "left-handed" };
                    lines.push((format!("{label}:  τ = {}  ({hand})", sgn(tau * s.signum(), 3)), col));
                }
            }
            Info::Knot { jones, jones_mirror, amphichiral } => {
                for (label, pts, col) in [("Original", orig, SAME), ("Image", copy, OPP)] {
                    let w = writhe(pts);
                    let hand = if amphichiral {
                        ""
                    } else if w > 0.0 {
                        "  (right-handed)"
                    } else {
                        "  (left-handed)"
                    };
                    lines.push((format!("{label}:  writhe Wr = {}{hand}", sgn(w, 2)), col));
                }
                lines.push(("Wr is a pseudoscalar: the mirror flips its sign.".to_string(), DIM));
                lines.push((format!("Jones  V(t) = {jones}"), TEXT));
                lines.push((format!("mirror: V(t⁻¹) = {jones_mirror}"), TEXT));
                if amphichiral {
                    lines.push(("Same polynomial → consistent with amphichiral".to_string(), SAME));
                    lines.push((format!("This rigid shape is {}: chiral as a solid,", g.name), DIM));
                    lines.push(("yet amphichiral as a knot (deform, don't rotate).".to_string(), DIM));
                } else {
                    lines.push(("V(t) ≠ V(t⁻¹)  →  topologically chiral".to_string(), OPP));
                    lines.push(("(which V belongs to which hand depends on convention)".to_string(), DIM));
                }
            }
            Info::Plain => {}
        }
        info_box("Mathematics", &lines);
    }

    pub fn sidebar(&mut self, ui: &mut Ui) {
        ui.heading("Model");
        if let Some(i) = ui.row(&["< Prev", "Next > [Tab]"], None) {
            let n = self.models.len();
            self.set_model(if i == 0 { self.cur + n - 1 } else { self.cur + 1 });
        }
        let (name, blurb, swappable) = (self.model().name, self.model().blurb, self.model().swappable);
        ui.label(&format!("{} / {}  {}", self.cur + 1, self.models.len(), name), 18.0, TEXT);
        ui.wrapped(blurb, 15.0, DIM);
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
                self.overlay = None;
            }
        } else if ui.button("Try to superimpose [Space]") {
            self.superimpose();
        }
        if let Some((ok, rms)) = self.verdict {
            if ok {
                ui.label("Superimposable  →  ACHIRAL", 16.0, GOOD);
            } else {
                ui.label("Not superimposable  →  CHIRAL (rigidly)", 16.0, OPP);
            }
            ui.label(&format!("best-fit RMS = {rms:.3}"), 14.0, DIM);
        }
        if ui.button("Reset rotations") {
            self.q_orig = default_q();
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
            ui.wrapped("Swapping any two groups converts one enantiomer into the other (watch χ flip back).", 14.0, DIM);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn molecule_chirality_and_groups() {
        // (chiral by rigid fit, point group)
        let expect = [
            (true, "C1"),
            (false, "Cs"),
            (true, "C2"),
            (true, "C3"),
            (false, "C3v"),
            (true, "D3"),
        ];
        let ms = models();
        for (m, (chiral, group)) in ms.iter().zip(expect) {
            let q = Quat::from_rotation_y(0.7) * Quat::from_rotation_x(0.3);
            let o: Vec<(Vec3, u8)> = m.atoms.iter().map(|a| (q * a.p, a.kind)).collect();
            let c: Vec<(Vec3, u8)> = o.iter().map(|&(p, k)| (Quat::from_rotation_z(1.1) * reflect_x(p), k)).collect();
            let (_, rms) = best_rotation(&o, &c);
            assert_eq!(rms >= 0.05, chiral, "{} rms={rms}", m.name);
            assert_eq!(m.group.name, group, "{}", m.name);
            assert_eq!(m.group.chiral(), chiral, "{}", m.name);
        }
    }

    #[test]
    fn pseudoscalars_flip_under_reflection() {
        let ms = models();
        // CHFClBr: original is one hand, its mirror the other.
        let m = &ms[0];
        let p: Vec<Vec3> = m.atoms.iter().map(|a| a.p).collect();
        let r: Vec<Vec3> = p.iter().map(|&v| reflect_x(v)).collect();
        let chi = |v: &[Vec3]| signed_volume(v[4], v[3], v[2], v[1]);
        assert!(chi(&p) * chi(&r) < 0.0);
        // Helix is right-handed (positive torsion), its mirror left-handed.
        let h: Vec<Vec3> = ms[2].atoms.iter().map(|a| a.p).collect();
        assert!(torsion_sign(&h) > 0.99);
        assert!(torsion_sign(&h.iter().map(|&v| reflect_x(v)).collect::<Vec<_>>()) < -0.99);
        // Trefoil writhe: clearly non-zero and flips sign.
        let k: Vec<Vec3> = ms[5].atoms.iter().map(|a| a.p).collect();
        let w = writhe(&k);
        assert!(w.abs() > 2.0, "writhe {w}");
        let wm = writhe(&k.iter().map(|&v| reflect_x(v)).collect::<Vec<_>>());
        assert!((w + wm).abs() < 1e-3);
    }
}
