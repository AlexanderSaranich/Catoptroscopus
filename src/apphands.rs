//! Hands mode: an animated, narrated sequence about the most famous chiral
//! objects of all, plus a particle-physics coda on helicity.

use crate::render3d::*;
use crate::ui::*;
use macroquad::prelude::*;
use std::f32::consts::PI;

const SKIN: Color = Color::new(0.96, 0.78, 0.64, 1.0);
const NAIL: Color = Color::new(1.0, 0.88, 0.88, 1.0);
const LEFT_C: Vec3 = Vec3::new(-2.2, 0.0, 0.0);
const RIGHT_C: Vec3 = Vec3::new(2.2, 0.0, 0.0);

#[derive(Clone, Copy, PartialEq)]
enum Part {
    Skin,
    Nail,
    Sleeve,
}

#[derive(Clone, Copy)]
struct Bead {
    p: Vec3,
    r: f32,
    part: Part,
}

struct HandModel {
    beads: Vec<Bead>,
    tips: Vec<usize>,
    palm: Vec3,
}

/// A LEFT hand: palm faces +z, fingers point +y, thumb on the +x side.
/// Handedness triad: a = fingers (+y), b = thumb (+x), c = palm normal (+z)
/// gives (a × b) · c = −1.
fn hand_model() -> HandModel {
    let mut beads = Vec::new();
    let mut tips = Vec::new();
    for ix in 0..4 {
        for iy in 0..4 {
            let p = vec3(-0.45 + 0.3 * ix as f32, -0.45 + 0.3 * iy as f32, 0.0);
            beads.push(Bead { p, r: 0.27, part: Part::Skin });
        }
    }
    let bases = [0.45f32, 0.15, -0.15, -0.45];
    let lens = [1.0f32, 1.12, 1.02, 0.8];
    let c = 0.7;
    for (xb, len) in bases.iter().zip(lens) {
        let steps = (len / 0.09).round() as usize;
        for i in 0..=steps {
            let s = len * i as f32 / steps as f32;
            let p = vec3(xb + xb * 0.18 * s, 0.6 + (c * s).sin() / c, (1.0 - (c * s).cos()) / c);
            beads.push(Bead { p, r: 0.14 - 0.03 * s / len, part: Part::Skin });
        }
        tips.push(beads.len() - 1);
        let s = len - 0.1;
        let n = vec3(0.0, (c * s).sin(), -(c * s).cos());
        let p = vec3(xb + xb * 0.18 * s, 0.6 + (c * s).sin() / c, (1.0 - (c * s).cos()) / c);
        beads.push(Bead { p: p + n * 0.075, r: 0.085, part: Part::Nail });
    }
    // Thumb.
    let base = vec3(0.5, -0.35, 0.1);
    let d = vec3(0.8, 0.6, 0.45).normalize();
    let len = 0.85;
    let steps = 10;
    for i in 0..=steps {
        let s = len * i as f32 / steps as f32;
        let p = base + d * s + vec3(0.0, 0.25 * s * s, 0.0);
        beads.push(Bead { p, r: 0.17 - 0.04 * s / len, part: Part::Skin });
    }
    tips.push(beads.len() - 1);
    let s = len - 0.1;
    let p = base + d * s + vec3(0.0, 0.25 * s * s, 0.0);
    beads.push(Bead { p: p + vec3(0.45, 0.0, -0.8).normalize() * 0.08, r: 0.09, part: Part::Nail });
    // Wrist and sleeve.
    let mut y = -0.65;
    while y > -1.8 {
        let sleeve = y < -0.95;
        beads.push(Bead { p: vec3(0.0, y, 0.0), r: if sleeve { 0.36 } else { 0.29 }, part: if sleeve { Part::Sleeve } else { Part::Skin } });
        y -= 0.12;
    }
    // Centre on the bead centroid.
    let cen = beads.iter().map(|b| b.p).sum::<Vec3>() / beads.len() as f32;
    for b in &mut beads {
        b.p -= cen;
    }
    HandModel { beads, tips, palm: -cen }
}

fn refl_x() -> Mat3 {
    Mat3::from_diagonal(vec3(-1.0, 1.0, 1.0))
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Handedness of a pose: sign of (a × b)·c for the transformed triad.
fn chi(l: Mat3) -> f32 {
    let (a, b, c) = (l * Vec3::Y, l * Vec3::X, l * Vec3::Z);
    a.cross(b).dot(c)
}

struct Scene {
    title: &'static str,
    dur: f32,
}

const SCENES: [Scene; 5] = [
    Scene { title: "1. A hand and its reflection", dur: 10.0 },
    Scene { title: "2. Try to superimpose", dur: 15.0 },
    Scene { title: "3. The handedness triad", dur: 12.0 },
    Scene { title: "4. A detour through the 4th dimension", dur: 11.0 },
    Scene { title: "5. Helicity: chirality in physics", dur: 12.0 },
];

const ATTEMPTS: [&str; 4] = [
    "Slide it over: palms agree, but the thumbs point opposite ways.",
    "Turn 180° about the vertical: thumbs agree, but now the palm faces away.",
    "Turn 180° about the palm normal: palm and thumb agree, but the fingers point down.",
    "Turn 180° about the left–right axis: upside down and back to front.",
];

pub struct AppHands {
    hand: HandModel,
    cam: Cam,
    scene: usize,
    t: f32,
    playing: bool,
    auto_advance: bool,
    show_mirror: bool,
    dragging_cam: bool,
    scrubbing: bool,
    last: Vec2,
}

/// Pose of one hand: world = center + l * local.
#[derive(Clone, Copy)]
struct Pose {
    center: Vec3,
    l: Mat3,
    alpha: f32,
    /// Optional 4D rotation angle for colouring by the hidden w coordinate.
    w_theta: Option<(f32, Mat3)>,
}

impl AppHands {
    pub fn new() -> AppHands {
        AppHands {
            hand: hand_model(),
            cam: Cam::new(0.35, 0.22, 12.0),
            scene: 0,
            t: 0.0,
            playing: true,
            auto_advance: true,
            show_mirror: true,
            dragging_cam: false,
            scrubbing: false,
            last: Vec2::ZERO,
        }
    }

    pub fn goto(&mut self, i: usize) {
        self.scene = i % SCENES.len();
        self.t = 0.0;
        self.playing = true;
    }

    pub fn set_time(&mut self, t: f32) {
        self.t = t.clamp(0.0, SCENES[self.scene].dur);
    }

    fn bar_rect() -> Rect {
        let x0 = SIDEBAR_W + 40.0;
        Rect::new(x0, screen_height() - 34.0, screen_width() - x0 - 40.0, 8.0)
    }

    fn handle_input(&mut self) {
        let m = mouse();
        let dt = get_frame_time();
        let bar = Self::bar_rect();
        let bar_hit = Rect::new(bar.x, bar.y - 8.0, bar.w, bar.h + 16.0);
        if is_mouse_button_pressed(MouseButton::Left) && bar_hit.contains(m) {
            self.scrubbing = true;
        } else if (is_mouse_button_pressed(MouseButton::Left) || is_mouse_button_pressed(MouseButton::Right)) && in_canvas(m) {
            self.dragging_cam = true;
        }
        if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
            if self.scrubbing {
                self.t = ((m.x - bar.x) / bar.w).clamp(0.0, 1.0) * SCENES[self.scene].dur;
            } else if self.dragging_cam {
                self.cam.orbit(m - self.last);
            }
        } else {
            self.dragging_cam = false;
            self.scrubbing = false;
        }
        self.last = m;
        self.cam.keys_and_wheel(false);

        if is_key_pressed(KeyCode::Space) {
            self.playing = !self.playing;
        }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::Tab) {
            self.goto(self.scene + 1);
        }
        if is_key_pressed(KeyCode::Left) {
            self.goto(self.scene + SCENES.len() - 1);
        }
        if self.playing && !self.scrubbing {
            self.t += dt;
            if self.t > SCENES[self.scene].dur {
                if self.auto_advance {
                    self.goto(self.scene + 1);
                } else {
                    self.t = 0.0;
                }
            }
        }
    }

    fn push_hand(&self, out: &mut Vec<Prim>, pose: Pose) {
        let det_sign = chi(pose.l);
        let sleeve = if det_sign.abs() < 0.05 {
            DIM
        } else if det_sign < 0.0 {
            SAME
        } else {
            OPP
        };
        for b in &self.hand.beads {
            let mut col = match b.part {
                Part::Skin => SKIN,
                Part::Nail => NAIL,
                Part::Sleeve => sleeve,
            };
            if let Some((theta, base)) = pose.w_theta {
                let w = (base * b.p).x * theta.sin();
                let k = (w.abs() / 1.3).min(1.0) * 0.75;
                let tint = if w > 0.0 { Color::new(0.35, 0.55, 1.0, 1.0) } else { Color::new(1.0, 0.35, 0.45, 1.0) };
                col = Color::new(
                    col.r + (tint.r - col.r) * k,
                    col.g + (tint.g - col.g) * k,
                    col.b + (tint.b - col.b) * k,
                    1.0,
                );
            }
            push_ball(&self.cam, out, pose.center + pose.l * b.p, b.r, with_alpha(col, pose.alpha));
        }
    }

    fn draw_hands(&self, poses: &[Pose], mismatch: Option<(Pose, Pose)>) {
        let cam_side = self.cam.eye().x.signum();
        let mut back = Vec::new();
        let mut front = Vec::new();
        for p in poses {
            let mut v = Vec::new();
            self.push_hand(&mut v, *p);
            if p.center.x.signum() == cam_side || p.center.x.abs() < 1.5 {
                front.extend(v);
            } else {
                back.extend(v);
            }
        }
        if let Some((a, b)) = mismatch {
            for &i in &self.hand.tips {
                let pa = a.center + a.l * self.hand.beads[i].p;
                let pb = b.center + b.l * self.hand.beads[i].p;
                if pa.distance(pb) > 0.2 {
                    push_dash(&self.cam, &mut front, pa, pb, Color::new(1.0, 0.3, 0.3, 0.95));
                }
            }
        }
        draw_prims(&mut back);
        if self.show_mirror {
            let s = 2.6;
            draw_quad(&self.cam, [vec3(0.0, -s, -s), vec3(0.0, -s, s), vec3(0.0, s, s), vec3(0.0, s, -s)], MIRROR);
        }
        draw_prims(&mut front);
    }

    fn triad(&self, pose: &Pose, lab: bool) {
        let o = pose.center + pose.l * self.hand.palm + pose.l * vec3(0.0, 0.0, 0.45);
        let len = 1.25;
        let arrows = [
            (Vec3::Y, Color::new(1.0, 0.4, 0.4, 1.0), "a fingers"),
            (Vec3::X, Color::new(0.4, 0.95, 0.5, 1.0), "b thumb"),
            (Vec3::Z, Color::new(0.45, 0.65, 1.0, 1.0), "c palm"),
        ];
        for (v, col, name) in arrows {
            let d = (pose.l * v).normalize_or_zero();
            draw_arrow3(&self.cam, o, o + d * len, col, if lab { name } else { "" });
        }
    }

    pub fn update_and_draw(&mut self) {
        self.handle_input();
        draw_floor(&self.cam, -2.4);
        let t = self.t;
        let id = Mat3::IDENTITY;
        let mut caption: String;
        let mut math: Vec<(String, Color)> = Vec::new();

        match self.scene {
            0 => {
                let lo = Mat3::from_quat(Quat::from_rotation_y(0.35 * (0.8 * t).sin()) * Quat::from_rotation_x(0.25 * (0.6 * t).sin()));
                let left = Pose { center: LEFT_C, l: lo, alpha: 1.0, w_theta: None };
                let right = Pose { center: RIGHT_C, l: refl_x() * lo, alpha: smooth(t / 1.5), w_theta: None };
                self.draw_hands(&[left, right], None);
                label3(&self.cam, LEFT_C + vec3(0.0, 2.5, 0.0), "LEFT hand", 18.0, SAME);
                label3(&self.cam, RIGHT_C + vec3(0.0, 2.5, 0.0), "its mirror image: a RIGHT hand", 18.0, OPP);
                caption = "Hold your left hand up to a mirror and a right hand looks back. The reflection follows every move of the original, but with left and right exchanged.".into();
                math.push(("Reflection in the plane x = 0:".into(), TEXT));
                math.push(("M = diag(−1, 1, 1),   (x, y, z) → (−x, y, z)".into(), TEXT));
                math.push(("det M = −1  →  improper: orientation reversed".into(), OPP));
                math.push(("Rotations R in SO(3) have det R = +1.".into(), DIM));
            }
            1 => {
                let left = Pose { center: LEFT_C, l: id, alpha: 1.0, w_theta: None };
                let slide = smooth((t - 1.5) / 1.5);
                let center = RIGHT_C.lerp(LEFT_C, slide);
                let rots = [Quat::IDENTITY, Quat::from_rotation_y(PI), Quat::from_rotation_z(PI), Quat::from_rotation_x(PI)];
                let mut q = Quat::IDENTITY;
                let mut k_cur = None;
                for k in 1..4 {
                    let start = 3.0 + 2.6 * k as f32;
                    if t >= start - 1.0 {
                        q = rots[k - 1].slerp(rots[k], smooth((t - (start - 1.0)) / 1.0));
                    }
                }
                for k in 0..4 {
                    if t >= 3.0 + 2.6 * k as f32 - 1.0 && t >= 3.0 {
                        k_cur = Some(k);
                    }
                }
                let right = Pose { center, l: Mat3::from_quat(q) * refl_x(), alpha: if slide > 0.0 { 0.85 } else { 1.0 }, w_theta: None };
                let overlaid = slide >= 1.0;
                self.draw_hands(&[left, right], if overlaid { Some((left, right)) } else { None });
                caption = match k_cur {
                    _ if t < 1.5 => "Can we move the right hand onto the left one using only rigid motions (slides and turns)?".into(),
                    _ if t < 3.0 => "Slide the right hand over...".into(),
                    Some(k) if t < 3.0 + 2.6 * 4.0 => ATTEMPTS[k].into(),
                    _ => "Every rotation fixes one feature and breaks another. No element of SO(3) works: hands are chiral.".into(),
                };
                if overlaid {
                    let msg = "red dashes: fingertips that don't line up";
                    text(msg, SIDEBAR_W + 20.0, 34.0, 16.0, Color::new(1.0, 0.45, 0.45, 1.0));
                }
                math.push(("Rigid motions: x → R x + t,  R in SO(3)".into(), TEXT));
                math.push(("det(R · M) = det R · det M = (+1)(−1) = −1".into(), TEXT));
                math.push(("No product of rotations has det −1,".into(), DIM));
                math.push(("so no rigid motion maps L onto R.".into(), DIM));
                math.push(("Chiral ⇔ symmetry group has no improper element.".into(), OPP));
            }
            2 => {
                let lo = Mat3::from_quat(Quat::from_rotation_y(0.7 * t) * Quat::from_rotation_x(0.4 * (0.9 * t).sin()));
                let ro = Mat3::from_quat(Quat::from_rotation_z(0.5 * t) * Quat::from_rotation_y(-0.6 * t)) * refl_x();
                let left = Pose { center: LEFT_C, l: lo, alpha: 1.0, w_theta: None };
                let right = Pose { center: RIGHT_C, l: ro, alpha: 1.0, w_theta: None };
                self.draw_hands(&[left, right], None);
                self.triad(&left, true);
                self.triad(&right, false);
                caption = "Fingers (a), thumb (b) and palm normal (c) form a frame. However the hands tumble, the sign of the triple product never changes: it is a pseudoscalar, and only a reflection can flip it.".into();
                math.push(("χ = sign((a × b) · c) = sign det[a b c]".into(), TEXT));
                math.push((format!("Left hand:   χ = {}", pm1(chi(lo))), SAME));
                math.push((format!("Right hand:  χ = {}", pm1(chi(ro))), OPP));
                math.push(("Rotations: det(R[a b c]) = det[a b c]".into(), DIM));
                math.push(("Same idea as R/S for a stereocentre (3D tab).".into(), DIM));
            }
            3 => {
                let left = Pose { center: LEFT_C, l: id, alpha: 1.0, w_theta: None };
                let theta = PI * smooth((t - 1.0) / 4.0);
                let slide = smooth((t - 5.5) / 2.0);
                let shadow = Mat3::from_diagonal(vec3(theta.cos(), 1.0, 1.0));
                let base = refl_x();
                let right = Pose {
                    center: RIGHT_C.lerp(LEFT_C, slide),
                    l: shadow * base,
                    alpha: if slide > 0.0 { 0.8 } else { 1.0 },
                    w_theta: Some((theta, base)),
                };
                self.draw_hands(&[left, right], None);
                if slide >= 1.0 {
                    label3(&self.cam, LEFT_C + vec3(0.0, 2.6, 0.0), "Superimposed!", 24.0, GOOD);
                }
                caption = if t < 1.0 {
                    "A flat 'F' can be flipped by lifting it through 3D. Can a hand be flipped by 'lifting' it through a 4th dimension w?".into()
                } else if t < 5.0 {
                    "Rotate in the (x, w) plane. We only see the 3D shadow: the hand flattens to nothing, then comes back mirrored. Colour shows the hidden w coordinate.".into()
                } else {
                    "It emerges as a LEFT hand, and now slides perfectly onto the original. Chirality depends on the dimension of the space.".into()
                };
                let d = -theta.cos();
                math.push(("4D rotation in the (x, w) plane:".into(), TEXT));
                math.push(("x′ = x cos θ − w sin θ,   w′ = x sin θ + w cos θ".into(), TEXT));
                math.push((format!("θ = {:.0}°   (4D det = +1, a proper rotation)", theta.to_degrees()), DIM));
                math.push((format!("3D shadow: det = −cos θ = {}", sgn(d, 2)), if d.abs() < 0.05 { DIM } else if d > 0.0 { SAME } else { OPP }));
                math.push(("blue: w > 0    red: w < 0".into(), DIM));
            }
            _ => {
                self.draw_helicity(t);
                caption = "Spin is an axial vector: a mirror reverses the sense of rotation but not the direction of motion, so the helicity h = S·p/|p| changes sign. The weak force acts only on left-handed particles (Wu experiment, 1957): nature itself is chiral.".into();
                math.push(("Parity P: x → −x  (improper)".into(), TEXT));
                math.push(("Helicity h = S · p / |p|  is a pseudoscalar".into(), TEXT));
                math.push(("top: h = +½ (right-handed)".into(), OPP));
                math.push(("mirror: h = −½ (left-handed)".into(), SAME));
                math.push(("Dirac: P_L = ½(1 − γ⁵), P_R = ½(1 + γ⁵); massless → chirality = helicity".into(), DIM));
            }
        }

        info_box("Mathematics", &math);

        // Caption panel and scrub bar.
        let x0 = SIDEBAR_W + 40.0;
        let w = screen_width() - x0 - 40.0;
        let sh = screen_height();
        draw_rectangle(x0 - 16.0, sh - 128.0, w + 32.0, 116.0, with_alpha(PANEL, 0.85));
        text(SCENES[self.scene].title, x0, sh - 104.0, 20.0, MIRROR);
        let mut ui = Ui::new(x0, sh - 96.0, w);
        caption = caption.replace("  ", " ");
        ui.wrapped(&caption, 16.0, TEXT);
        let bar = Self::bar_rect();
        draw_rectangle(bar.x, bar.y, bar.w, bar.h, Color::new(1.0, 1.0, 1.0, 0.1));
        let f = self.t / SCENES[self.scene].dur;
        draw_rectangle(bar.x, bar.y, bar.w * f, bar.h, MIRROR);
        draw_circle(bar.x + bar.w * f, bar.y + bar.h / 2.0, 7.0, TEXT);
        if !self.playing {
            text("paused", bar.x + bar.w - 50.0, bar.y - 6.0, 14.0, DIM);
        }
    }

    fn draw_helicity(&self, t: f32) {
        let x = -3.0 + 6.0 * ((t / 6.0).fract());
        let phi = 3.0 * t;
        let top = vec3(x, 1.3, 0.0);
        let mut up = Vec::new();
        let mut down = Vec::new();
        let ring = 10;
        for (center, list, sign) in [(top, &mut up, 1.0f32), (vec3(x, -1.3, 0.0), &mut down, -1.0)] {
            push_ball(&self.cam, list, center, 0.33, Color::new(0.7, 0.5, 0.95, 1.0));
            for i in 0..ring {
                let a = phi + i as f32 * std::f32::consts::TAU / ring as f32;
                // Ring rotating about +x; reflecting y -> -y reverses its sense.
                let off = vec3(0.0, 0.75 * a.cos() * sign, 0.75 * a.sin());
                let col = if i == 0 { Color::new(1.0, 0.85, 0.3, 1.0) } else { Color::new(0.75, 0.8, 0.9, 1.0) };
                push_ball(&self.cam, list, center + off, if i == 0 { 0.13 } else { 0.09 }, col);
            }
        }
        let cam_up = self.cam.eye().y > 0.0;
        let (mut far, mut near) = if cam_up { (down, up) } else { (up, down) };
        draw_prims(&mut far);
        if self.show_mirror {
            draw_quad(&self.cam, [vec3(-4.0, 0.0, -2.0), vec3(4.0, 0.0, -2.0), vec3(4.0, 0.0, 2.0), vec3(-4.0, 0.0, 2.0)], MIRROR);
        }
        draw_prims(&mut near);
        for (y, s, col) in [(1.3f32, 1.0f32, OPP), (-1.3, -1.0, SAME)] {
            let c = vec3(x, y, 0.0);
            draw_arrow3(&self.cam, c + vec3(0.0, 0.0, 1.1), c + vec3(1.5, 0.0, 1.1), TEXT, "p");
            draw_arrow3(&self.cam, c, c + vec3(1.2 * s, 0.0, 0.0), Color::new(1.0, 0.85, 0.3, 1.0), "S");
            let h = if s > 0.0 { "h = +½  right-handed" } else { "h = −½  left-handed" };
            label3(&self.cam, vec3(0.0, y + if y > 0.0 { 1.3 } else { -1.1 }, 0.0), h, 18.0, col);
        }
    }

    pub fn sidebar(&mut self, ui: &mut Ui) {
        ui.heading("Animation");
        for (i, s) in SCENES.iter().enumerate() {
            let r = Rect::new(ui.x, ui.y, ui.w, 24.0);
            let active = i == self.scene;
            let hover = r.contains(mouse());
            draw_rectangle(r.x, r.y, r.w, r.h, if active { with_alpha(MIRROR, 0.3) } else if hover { Color::new(0.18, 0.21, 0.29, 1.0) } else { Color::new(0.13, 0.15, 0.21, 1.0) });
            text(s.title, r.x + 8.0, r.y + 17.0, 15.0, TEXT);
            if hover && is_mouse_button_pressed(MouseButton::Left) {
                self.goto(i);
            }
            ui.y += 28.0;
        }
        ui.space(4.0);
        match ui.row(&["< Prev", if self.playing { "Pause" } else { "Play" }, "Next >"], None) {
            Some(0) => self.goto(self.scene + SCENES.len() - 1),
            Some(1) => self.playing = !self.playing,
            Some(2) => self.goto(self.scene + 1),
            _ => {}
        }
        if ui.button("Restart scene") {
            self.t = 0.0;
        }
        ui.toggle("Auto-advance to next scene", &mut self.auto_advance);
        ui.toggle("Show mirror", &mut self.show_mirror);

        ui.heading("Colours");
        ui.swatch(SAME, "Left-handed sleeve (χ = −1)");
        ui.swatch(OPP, "Right-handed sleeve (χ = +1)");
        ui.swatch(DIM, "Flattened: det = 0");

        ui.heading("Controls");
        for l in ["Space: play/pause   ← →: scene", "Drag: orbit camera   Wheel: zoom", "Click the bar to scrub"] {
            ui.label(l, 14.0, DIM);
        }
    }
}
