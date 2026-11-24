//! 2D mode: drag shapes and mirror lines, see reflections, kaleidoscopes,
//! and try to superimpose a shape on its mirror image.

use crate::geom::*;
use crate::shapes::{library, ShapeDef, SHAPE_RADIUS};
use crate::ui::*;
use macroquad::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

const MAX_IMAGES: usize = 400;
const KNOB_DIST: f32 = SHAPE_RADIUS + 26.0;

#[derive(Clone, Copy)]
struct Piece {
    pos: Vec2,
    angle: f32,
    flipped: bool,
}

impl Piece {
    fn mat(&self) -> Mat2 {
        let r = Mat2::from_angle(self.angle);
        if self.flipped {
            r * flip()
        } else {
            r
        }
    }
    fn xf(&self) -> Xf {
        Xf { m: self.mat(), t: self.pos }
    }
    fn knob(&self) -> Vec2 {
        self.pos + Mat2::from_angle(self.angle) * vec2(0.0, -KNOB_DIST)
    }
}

#[derive(Clone, Copy)]
struct Mirror {
    a: Vec2,
    b: Vec2,
}

#[derive(Clone, Copy, PartialEq)]
enum Which {
    Orig,
    Copy,
}

#[derive(Clone, Copy)]
enum Drag {
    None,
    Body(Which, Vec2),
    Knob(Which),
    HandleA(usize),
    HandleB(usize),
    Line(usize, Vec2),
}

pub struct App2D {
    shapes: Vec<ShapeDef>,
    cur: usize,
    orig: Piece,
    copy: Piece,
    copy_on: bool,
    mirrors: Vec<Mirror>,
    depth: usize,
    show_sym: bool,
    show_arrows: bool,
    drag: Drag,
    selected: Which,
    sel_mirror: Option<usize>,
    flash: Option<(String, Color, f64)>,
    image_count: usize,
    /// (piece, start time, matrix before the flip) for the flip-through-3D animation.
    flip_anim: Option<(Which, f64, Mat2)>,
}

fn canvas_center() -> Vec2 {
    vec2(SIDEBAR_W + (screen_width() - SIDEBAR_W) / 2.0, screen_height() / 2.0)
}

impl App2D {
    pub fn new() -> App2D {
        let mut app = App2D {
            shapes: library(),
            cur: 0,
            orig: Piece { pos: Vec2::ZERO, angle: 0.0, flipped: false },
            copy: Piece { pos: Vec2::ZERO, angle: 0.0, flipped: true },
            copy_on: false,
            mirrors: vec![],
            depth: 1,
            show_sym: false,
            show_arrows: true,
            drag: Drag::None,
            selected: Which::Orig,
            sel_mirror: None,
            flash: None,
            flip_anim: None,
            image_count: 0,
        };
        app.preset_single();
        app
    }

    fn shape(&self) -> &ShapeDef {
        &self.shapes[self.cur]
    }

    fn piece(&mut self, w: Which) -> &mut Piece {
        match w {
            Which::Orig => &mut self.orig,
            Which::Copy => &mut self.copy,
        }
    }

    fn say(&mut self, s: &str, col: Color) {
        self.flash = Some((s.to_string(), col, get_time()));
    }

    // ---------- presets ----------

    fn preset_single(&mut self) {
        let c = canvas_center();
        self.mirrors = vec![Mirror { a: vec2(c.x, c.y - 120.0), b: vec2(c.x, c.y + 120.0) }];
        self.orig = Piece { pos: vec2(c.x - 220.0, c.y), angle: 0.0, flipped: false };
        self.depth = 1;
        self.sel_mirror = None;
        self.place_copy();
    }

    /// Two mirrors meeting at angle PI/n: a kaleidoscope with 2n images.
    fn preset_wedge(&mut self, n: usize) {
        let c = canvas_center() + vec2(0.0, 40.0);
        let th = PI / n as f32;
        let dir = |a: f32| vec2(a.cos(), a.sin());
        let a0 = -FRAC_PI_2 - th / 2.0;
        self.mirrors = vec![
            Mirror { a: c, b: c + dir(a0) * 200.0 },
            Mirror { a: c, b: c + dir(a0 + th) * 200.0 },
        ];
        let dist = (SHAPE_RADIUS * 1.15 / (th / 2.0).sin()).max(170.0).min(330.0);
        self.orig = Piece { pos: c + dir(-FRAC_PI_2) * dist, angle: 0.0, flipped: false };
        self.depth = n.min(10);
        self.sel_mirror = None;
        self.copy_on = false;
    }

    fn preset_parallel(&mut self) {
        let c = canvas_center();
        let gap = 130.0;
        self.mirrors = vec![
            Mirror { a: vec2(c.x - gap, c.y - 120.0), b: vec2(c.x - gap, c.y + 120.0) },
            Mirror { a: vec2(c.x + gap, c.y - 120.0), b: vec2(c.x + gap, c.y + 120.0) },
        ];
        self.orig = Piece { pos: vec2(c.x - 15.0, c.y), angle: 0.3, flipped: false };
        self.depth = 4;
        self.sel_mirror = None;
        self.copy_on = false;
    }

    fn preset_triangle(&mut self) {
        // Equilateral triangle of mirrors: an infinite kaleidoscope.
        let c = canvas_center();
        let r = 150.0;
        let p: Vec<Vec2> = (0..3)
            .map(|i| {
                let a = -FRAC_PI_2 + i as f32 * 2.0 * PI / 3.0;
                c + vec2(a.cos(), a.sin()) * r
            })
            .collect();
        self.mirrors = (0..3).map(|i| Mirror { a: p[i], b: p[(i + 1) % 3] }).collect();
        self.orig = Piece { pos: c, angle: 0.2, flipped: false };
        self.depth = 6;
        self.sel_mirror = None;
        self.copy_on = false;
    }

    /// Put the copy where the first mirror would show its reflection, but off to the side.
    fn place_copy(&mut self) {
        let c = canvas_center();
        self.copy = Piece {
            pos: vec2(c.x + 330.0, c.y + 170.0),
            angle: PI - self.orig.angle,
            flipped: !self.orig.flipped,
        };
    }

    fn add_mirror(&mut self) {
        let c = canvas_center();
        let k = self.mirrors.len() as f32;
        let a = 0.6 + k * 0.7;
        let d = vec2(a.cos(), a.sin()) * 120.0;
        let o = vec2(k * 30.0, -k * 20.0);
        self.mirrors.push(Mirror { a: c + o - d, b: c + o + d });
        self.sel_mirror = Some(self.mirrors.len() - 1);
    }

    fn remove_mirror(&mut self) {
        if self.mirrors.is_empty() {
            return;
        }
        let i = self.sel_mirror.unwrap_or(self.mirrors.len() - 1);
        self.mirrors.remove(i.min(self.mirrors.len() - 1));
        self.sel_mirror = None;
    }

    fn set_shape(&mut self, i: usize) {
        self.cur = i % self.shapes.len();
    }

    // ---------- geometry ----------

    fn world_pts(&self, xf: &Xf) -> Vec<Vec2> {
        self.shape().pts.iter().map(|&p| xf.apply(p)).collect()
    }

    /// Every distinct composition of mirror reflections up to `depth`.
    fn images(&self) -> Vec<(Xf, usize)> {
        let refl: Vec<Xf> = self.mirrors.iter().map(|m| reflection(m.a, m.b)).collect();
        let base = self.display_xf(Which::Orig);
        let mut all: Vec<(Xf, usize)> = vec![(base, 0)];
        let mut frontier = vec![base];
        let (sw, sh) = (screen_width(), screen_height());
        let visible = |p: Vec2| p.x > SIDEBAR_W - 400.0 && p.x < sw + 400.0 && p.y > -400.0 && p.y < sh + 400.0;
        for d in 1..=self.depth {
            let mut next = Vec::new();
            for xf in &frontier {
                for r in &refl {
                    let n = xf.then(r);
                    if !visible(n.t) || all.iter().any(|(e, _)| e.approx_eq(&n)) {
                        continue;
                    }
                    all.push((n, d));
                    next.push(n);
                    if all.len() >= MAX_IMAGES {
                        return all;
                    }
                }
            }
            frontier = next;
        }
        all
    }

    /// If the copy (as currently placed) coincides with the original up to a
    /// symmetry of the shape, return the exact matrix it should snap to.
    fn copy_match(&self) -> Option<Mat2> {
        if self.copy.pos.distance(self.orig.pos) > 16.0 {
            return None;
        }
        let ac = self.copy.mat();
        let ao = self.orig.mat();
        self.shape()
            .group
            .iter()
            .map(|&s| ao * s)
            .find(|t| t.determinant() * ac.determinant() > 0.0 && t.abs_diff_eq(ac, 0.12))
    }

    fn exactly_matched(&self) -> bool {
        self.copy_on
            && self.copy.pos.distance(self.orig.pos) < 0.5
            && self.copy_match().map_or(false, |t| t.abs_diff_eq(self.copy.mat(), 1e-3))
    }

    // ---------- input ----------

    fn handle_input(&mut self) {
        let m = mouse();
        let dt = get_frame_time();

        if is_mouse_button_pressed(MouseButton::Left) && in_canvas(m) {
            self.drag = self.pick(m);
        }
        if is_mouse_button_down(MouseButton::Left) {
            let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            match self.drag {
                Drag::None => {}
                Drag::Body(w, off) => self.piece(w).pos = m + off,
                Drag::Knob(w) => {
                    let p = self.piece(w);
                    let v = m - p.pos;
                    let mut a = v.y.atan2(v.x) + FRAC_PI_2;
                    if shift {
                        let s = PI / 12.0;
                        a = (a / s).round() * s;
                    }
                    p.angle = a;
                }
                Drag::HandleA(i) => self.mirrors[i].a = snap_dir(self.mirrors[i].b, m, shift),
                Drag::HandleB(i) => self.mirrors[i].b = snap_dir(self.mirrors[i].a, m, shift),
                Drag::Line(i, last) => {
                    let d = m - last;
                    self.mirrors[i].a += d;
                    self.mirrors[i].b += d;
                    self.drag = Drag::Line(i, m);
                }
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            if let Drag::Body(Which::Copy, _) | Drag::Knob(Which::Copy) = self.drag {
                self.try_snap();
            }
            self.drag = Drag::None;
        }

        // Wheel rotates whichever piece is under the mouse.
        let wheel = mouse_wheel().1;
        if wheel != 0.0 && in_canvas(m) {
            if let Some(w) = self.piece_at(m) {
                self.selected = w;
                self.piece(w).angle += wheel.signum() * PI / 24.0;
                if w == Which::Copy {
                    self.try_snap();
                }
            }
        }

        let sel = self.selected;
        if is_key_down(KeyCode::Q) {
            self.piece(sel).angle -= 1.6 * dt;
        }
        if is_key_down(KeyCode::E) {
            self.piece(sel).angle += 1.6 * dt;
        }
        if (is_key_released(KeyCode::Q) || is_key_released(KeyCode::E)) && sel == Which::Copy {
            self.try_snap();
        }
        if is_key_pressed(KeyCode::F) {
            self.flip(sel);
        }
        if is_key_pressed(KeyCode::M) {
            self.add_mirror();
        }
        if is_key_pressed(KeyCode::Delete) || is_key_pressed(KeyCode::Backspace) {
            self.remove_mirror();
        }
        if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Right) {
            self.set_shape(self.cur + 1);
        }
        if is_key_pressed(KeyCode::Left) {
            self.set_shape(self.cur + self.shapes.len() - 1);
        }
        if is_key_pressed(KeyCode::C) {
            self.toggle_copy();
        }
        if is_key_pressed(KeyCode::S) {
            self.show_sym = !self.show_sym;
        }
        if is_key_pressed(KeyCode::A) {
            self.show_arrows = !self.show_arrows;
        }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            self.depth = (self.depth + 1).min(12);
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            self.depth = self.depth.saturating_sub(1).max(1);
        }
    }

    /// Flip a piece over by "rotating it through 3D" (animated).
    fn flip(&mut self, w: Which) {
        let old = self.piece(w).mat();
        let p = self.piece(w);
        p.flipped = !p.flipped;
        p.angle = -p.angle;
        self.flip_anim = Some((w, get_time(), old));
        self.say("Flipped through 3D! Inside the plane, a flip can only be done by a reflection.", OPP);
    }

    /// Public hooks used by the Theory page.
    pub fn select_shape(&mut self, i: usize) {
        self.set_shape(i);
    }

    pub fn flip_demo(&mut self) {
        self.set_shape(0);
        self.preset_single();
        self.selected = Which::Orig;
        self.flip(Which::Orig);
    }

    /// Progress of the flip animation for piece `w` (None if not animating).
    fn flip_progress(&self, w: Which) -> Option<(f32, Mat2)> {
        match self.flip_anim {
            Some((fw, t0, old)) if fw == w => {
                let s = ((get_time() - t0) / 0.9) as f32;
                if s < 1.0 {
                    Some((s, old))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// The transform used for drawing, including any in-progress flip.
    fn display_xf(&self, w: Which) -> Xf {
        let p = match w {
            Which::Orig => self.orig,
            Which::Copy => self.copy,
        };
        match self.flip_progress(w) {
            // Rotating about the world horizontal axis through 3D: its shadow
            // in the plane scales y by cos(pi s), passing through det = 0.
            Some((s, old)) => {
                let sq = Mat2::from_cols(vec2(1.0, 0.0), vec2(0.0, (PI * s).cos()));
                Xf { m: sq * old, t: p.pos }
            }
            None => p.xf(),
        }
    }

    fn toggle_copy(&mut self) {
        self.copy_on = !self.copy_on;
        if self.copy_on {
            self.place_copy();
            self.selected = Which::Copy;
            self.say("Here is the mirror image. Drag & rotate it onto the original!", COPY);
        } else {
            self.selected = Which::Orig;
        }
    }

    fn try_snap(&mut self) {
        if let Some(t) = self.copy_match() {
            let was = self.exactly_matched();
            self.copy.pos = self.orig.pos;
            self.copy.angle = mat_angle(t);
            if !was {
                let msg = if self.copy.flipped != self.orig.flipped && !self.shape().is_chiral() {
                    "It fits! The mirror image of an achiral shape is the same shape."
                } else if self.copy.flipped != self.orig.flipped {
                    "It fits... but only because you flipped it (F). That's cheating!"
                } else {
                    "It fits! Same hand, so a rotation and a slide are enough."
                };
                self.say(msg, GOOD);
            }
        }
    }

    fn piece_at(&self, m: Vec2) -> Option<Which> {
        if self.copy_on && point_in_polygon(m, &self.world_pts(&self.copy.xf())) {
            return Some(Which::Copy);
        }
        if point_in_polygon(m, &self.world_pts(&self.orig.xf())) {
            return Some(Which::Orig);
        }
        None
    }

    fn pick(&mut self, m: Vec2) -> Drag {
        // Mirror handles (selected mirror first).
        let mut order: Vec<usize> = (0..self.mirrors.len()).collect();
        if let Some(s) = self.sel_mirror {
            order.retain(|&i| i != s);
            order.insert(0, s);
        }
        for &i in &order {
            let mi = self.mirrors[i];
            if m.distance(mi.a) < 11.0 {
                self.sel_mirror = Some(i);
                return Drag::HandleA(i);
            }
            if m.distance(mi.b) < 11.0 {
                self.sel_mirror = Some(i);
                return Drag::HandleB(i);
            }
        }
        // Rotation knobs.
        let mut pieces = vec![Which::Orig];
        if self.copy_on {
            pieces.insert(0, Which::Copy);
        }
        for &w in &pieces {
            if m.distance(self.piece(w).knob()) < 11.0 {
                self.selected = w;
                return Drag::Knob(w);
            }
        }
        if let Some(w) = self.piece_at(m) {
            self.selected = w;
            return Drag::Body(w, self.piece(w).pos - m);
        }
        for &i in &order {
            let mi = self.mirrors[i];
            if dist_to_line(m, mi.a, mi.b) < 7.0 {
                self.sel_mirror = Some(i);
                return Drag::Line(i, m);
            }
        }
        self.sel_mirror = None;
        Drag::None
    }

    // ---------- drawing ----------

    fn draw_piece(&self, xf: &Xf, fill: Color, edge: Color, thick: f32) {
        let pts = self.world_pts(xf);
        for t in &self.shape().tris {
            draw_triangle(pts[t[0]], pts[t[1]], pts[t[2]], fill);
        }
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            draw_line(a.x, a.y, b.x, b.y, thick, edge);
        }
        if self.show_arrows {
            self.draw_spin_arrow(xf, edge);
        }
    }

    /// A curled arrow carried along by the transform: it reverses under reflection,
    /// making the "handedness" of each copy visible.
    fn draw_spin_arrow(&self, xf: &Xf, col: Color) {
        let r = SHAPE_RADIUS * 0.26;
        let n = 20;
        let a0 = -0.3f32;
        let a1 = a0 + 1.5 * PI;
        let pt = |a: f32| xf.apply(vec2(a.cos(), a.sin()) * r);
        let col = with_alpha(Color::new(0.05, 0.06, 0.09, 1.0), 0.85 * col.a.max(0.6));
        for i in 0..n {
            let t0 = a0 + (a1 - a0) * i as f32 / n as f32;
            let t1 = a0 + (a1 - a0) * (i + 1) as f32 / n as f32;
            let (p, q) = (pt(t0), pt(t1));
            draw_line(p.x, p.y, q.x, q.y, 3.0, col);
        }
        // Arrow head at a1, pointing along increasing angle.
        let tip_local = vec2(a1.cos(), a1.sin()) * r;
        let tangent = vec2(-a1.sin(), a1.cos());
        let normal = vec2(a1.cos(), a1.sin());
        let tip = xf.apply(tip_local + tangent * 6.0);
        let l = xf.apply(tip_local - tangent * 5.0 + normal * 7.0);
        let rr = xf.apply(tip_local - tangent * 5.0 - normal * 7.0);
        draw_triangle(tip, l, rr, col);
    }

    fn draw_mirror(&self, i: usize, mi: &Mirror) {
        let sel = self.sel_mirror == Some(i);
        let d = (mi.b - mi.a).normalize_or_zero();
        if d == Vec2::ZERO {
            return;
        }
        let n = vec2(-d.y, d.x);
        let far = 4000.0;
        let (p, q) = (mi.a - d * far, mi.a + d * far);
        // Hatching on the "back" of the mirror, like a technical drawing.
        let mut t = -far;
        while t < far {
            let s = mi.a + d * t;
            let e = s + n * 9.0 - d * 6.0;
            draw_line(s.x, s.y, e.x, e.y, 1.0, with_alpha(MIRROR, 0.22));
            t += 14.0;
        }
        draw_line(p.x, p.y, q.x, q.y, 7.0, with_alpha(MIRROR, 0.12));
        draw_line(p.x, p.y, q.x, q.y, if sel { 3.5 } else { 2.5 }, with_alpha(MIRROR, if sel { 1.0 } else { 0.8 }));
        for h in [mi.a, mi.b] {
            draw_circle(h.x, h.y, 8.0, BG);
            draw_circle_lines(h.x, h.y, 8.0, 2.0, MIRROR);
        }
        let mid = (mi.a + mi.b) / 2.0;
        draw_circle(mid.x, mid.y, 3.0, MIRROR);
    }

    fn draw_knob(&self, p: &Piece, col: Color) {
        let k = p.knob();
        let edge = p.pos + Mat2::from_angle(p.angle) * vec2(0.0, -SHAPE_RADIUS * 0.6);
        dashed_line(edge, k, 4.0, 1.0, with_alpha(col, 0.5));
        draw_circle(k.x, k.y, 7.0, BG);
        draw_circle_lines(k.x, k.y, 7.0, 2.0, col);
    }

    fn draw_symmetry_lines(&self) {
        let o = self.orig;
        let base = mat_angle(o.mat());
        let sign = if o.flipped { -1.0 } else { 1.0 };
        for a in self.shape().mirror_line_angles() {
            let ang = base + sign * a;
            let d = vec2(ang.cos(), ang.sin()) * (SHAPE_RADIUS + 30.0);
            dashed_line(o.pos - d, o.pos + d, 6.0, 2.0, GOOD);
        }
        let order = self.shape().rotation_order();
        if order > 1 {
            draw_circle_lines(o.pos.x, o.pos.y, 5.0, 2.0, GOOD);
            text(&format!("{order}-fold"), o.pos.x + 8.0, o.pos.y - 8.0, 16.0, GOOD);
        }
    }

    pub fn update_and_draw(&mut self) {
        self.handle_input();

        // Faint grid.
        let (sw, sh) = (screen_width(), screen_height());
        let g = 40.0;
        let gc = Color::new(1.0, 1.0, 1.0, 0.03);
        let mut x = SIDEBAR_W;
        while x < sw {
            draw_line(x, 0.0, x, sh, 1.0, gc);
            x += g;
        }
        let mut y = 0.0;
        while y < sh {
            draw_line(SIDEBAR_W, y, sw, y, 1.0, gc);
            y += g;
        }

        let imgs = self.images();
        self.image_count = imgs.len();
        let base_det = self.orig.xf().det();
        for (xf, d) in imgs.iter().skip(1).rev() {
            let same = xf.det() * base_det > 0.0;
            let c = if same { SAME } else { OPP };
            let fade = (1.0 - (*d as f32 - 1.0) * 0.07).max(0.35);
            self.draw_piece(xf, with_alpha(c, 0.22 * fade), with_alpha(c, 0.85 * fade), 1.5);
        }
        for (i, mi) in self.mirrors.iter().enumerate() {
            self.draw_mirror(i, mi);
        }

        // Original.
        self.draw_piece(&self.display_xf(Which::Orig), with_alpha(SAME, 0.55), SAME, 2.5);
        self.draw_knob(&self.orig, SAME);
        if self.show_sym {
            self.draw_symmetry_lines();
        }

        // Movable mirror-image copy.
        if self.copy_on {
            let same = self.copy.mat().determinant() * base_det > 0.0;
            let c = if same { SAME } else { OPP };
            let matched = self.exactly_matched();
            let fill = with_alpha(c, if matched { 0.25 } else { 0.4 });
            self.draw_piece(&self.display_xf(Which::Copy), fill, if matched { GOOD } else { COPY }, 2.5);
            self.draw_knob(&self.copy, COPY);
            if matched {
                let p = self.orig.pos;
                text("SUPERIMPOSED", p.x - 60.0, p.y + SHAPE_RADIUS + 30.0, 22.0, GOOD);
            } else if self.copy.pos.distance(self.orig.pos) < SHAPE_RADIUS && !same && self.shape().is_chiral() {
                let p = self.orig.pos;
                text(
                    "Opposite hands: no slide or turn can make these coincide",
                    p.x - 210.0,
                    p.y + SHAPE_RADIUS + 30.0,
                    18.0,
                    OPP,
                );
            }
        }

        // Selection ring.
        if self.selected == Which::Copy && self.copy_on || self.selected == Which::Orig {
            let p = if self.selected == Which::Copy { self.copy.pos } else { self.orig.pos };
            draw_circle_lines(p.x, p.y, 4.0, 1.5, TEXT);
        }

        self.draw_math();
        hint("Drag: move  ·  Wheel / Q E / knob: rotate  ·  F: flip through 3D  ·  M / Del: mirrors  ·  + −: bounces  ·  Tab: shape  ·  C: copy  ·  S: symmetry lines");

        // Flash message.
        if let Some((msg, col, t0)) = &self.flash {
            let age = get_time() - t0;
            if age < 4.0 {
                let a = (1.0 - ((age - 3.0).max(0.0)) as f32).clamp(0.0, 1.0);
                let tw = text_width(msg, 20.0);
                let x = SIDEBAR_W + (sw - SIDEBAR_W - tw) / 2.0;
                draw_rectangle(x - 14.0, 14.0, tw + 28.0, 36.0, with_alpha(PANEL, 0.92 * a));
                draw_rectangle_lines(x - 14.0, 14.0, tw + 28.0, 36.0, 1.5, with_alpha(*col, a));
                text(msg, x, 38.0, 20.0, with_alpha(*col, a));
            }
        }
    }

    fn draw_math(&self) {
        let s = self.shape();
        let w = if self.copy_on { self.selected } else { Which::Orig };
        let a = self.display_xf(w).m;
        let det = a.determinant();
        let anim = self.flip_progress(w).is_some();
        let f = |v: f32| format!("{:>6}", sgn(v, 2));
        let who = if w == Which::Orig { "original" } else { "copy" };
        let mut lines: Vec<(String, Color)> = vec![
            (format!("Linear part of the {who}:"), DIM),
            (format!("A = [ {}  {} ]", f(a.x_axis.x), f(a.y_axis.x)), TEXT),
            (format!("      [ {}  {} ]", f(a.x_axis.y), f(a.y_axis.y)), TEXT),
            if anim {
                (format!("det A = {}   (passing through 3D!)", sgn(det, 2)), DIM)
            } else if det > 0.0 {
                ("det A = +1  →  proper (rotation)".to_string(), SAME)
            } else {
                ("det A = −1  →  improper (reflected)".to_string(), OPP)
            },
            (format!("Symmetry group G = {}   |G| = {}", s.group_name(), s.group.len()), TEXT),
            if s.is_chiral() {
                ("G has no reflections  →  CHIRAL".to_string(), OPP)
            } else {
                (format!("G has {} reflection(s)  →  ACHIRAL", s.mirror_line_angles().len()), SAME)
            },
            (format!("Chirality measure S = {:.2}   (0 = achiral)", s.ccm), TEXT),
        ];
        if !self.mirrors.is_empty() {
            lines.push(("Mirror at angle θ:  M = [cos 2θ  sin 2θ ; sin 2θ  −cos 2θ]".to_string(), DIM));
            lines.push(("det M = −1;  k reflections  →  det = (−1)^k".to_string(), DIM));
        }
        if self.copy_on {
            let rel = self.copy.mat().determinant() * self.orig.mat().determinant();
            lines.push((
                format!("det(A_copy) · det(A_orig) = {}", pm1(rel)),
                if rel < 0.0 { OPP } else { SAME },
            ));
        }
        info_box("Mathematics", &lines);
    }

    pub fn sidebar(&mut self, ui: &mut Ui) {
        ui.heading("Shape");
        if let Some(i) = ui.row(&["< Prev", "Next >"], None) {
            let n = self.shapes.len();
            self.set_shape(if i == 0 { self.cur + n - 1 } else { self.cur + 1 });
        }
        let (name, blurb, chiral, rot, lines) = {
            let s = self.shape();
            (s.name, s.blurb, s.is_chiral(), s.rotation_order(), s.mirror_line_angles().len())
        };
        ui.label(&format!("{} / {}  {}", self.cur + 1, self.shapes.len(), name), 20.0, TEXT);
        if chiral {
            ui.label("CHIRAL", 20.0, OPP);
        } else {
            ui.label("ACHIRAL", 20.0, SAME);
        }
        ui.label(&format!("Rotation: {rot}-fold   Mirror lines: {lines}"), 15.0, DIM);
        ui.wrapped(blurb, 15.0, DIM);
        ui.space(4.0);
        ui.toggle("Show own symmetry lines [S]", &mut self.show_sym);
        ui.toggle("Show handedness arrows [A]", &mut self.show_arrows);

        ui.heading("Mirrors");
        match ui.row(&["+ Add [M]", "- Remove [Del]"], None) {
            Some(0) => self.add_mirror(),
            Some(1) => self.remove_mirror(),
            _ => {}
        }
        let depth_label = format!("Bounces: {}", self.depth);
        match ui.row(&["-", &depth_label, "+"], None) {
            Some(0) => self.depth = self.depth.saturating_sub(1).max(1),
            Some(2) => self.depth = (self.depth + 1).min(12),
            _ => {}
        }
        ui.label("Presets:", 15.0, DIM);
        match ui.row(&["Single", "90 deg", "60 deg", "45 deg"], None) {
            Some(0) => self.preset_single(),
            Some(1) => self.preset_wedge(2),
            Some(2) => self.preset_wedge(3),
            Some(3) => self.preset_wedge(4),
            _ => {}
        }
        match ui.row(&["Parallel", "Triangle", "None"], None) {
            Some(0) => self.preset_parallel(),
            Some(1) => self.preset_triangle(),
            Some(2) => {
                self.mirrors.clear();
                self.sel_mirror = None;
            }
            _ => {}
        }
        ui.label(&format!("{} images visible", self.image_count.saturating_sub(1)), 15.0, DIM);

        ui.heading("Superimpose challenge");
        let mut on = self.copy_on;
        ui.toggle("Free mirror-image copy [C]", &mut on);
        if on != self.copy_on {
            self.toggle_copy();
        }
        let sel = self.selected;
        match ui.row(&["Flip through 3D [F]", "Reset copy"], None) {
            Some(0) => self.flip(sel),
            Some(1) => {
                if !self.copy_on {
                    self.toggle_copy();
                }
                self.place_copy();
            }
            _ => {}
        }

        ui.heading("Legend");
        ui.swatch(SAME, "Same hand (even # of reflections)");
        ui.swatch(OPP, "Opposite hand (odd #) = enantiomorph");
        ui.swatch(COPY, "Your movable copy (outline)");
    }
}

fn snap_dir(anchor: Vec2, m: Vec2, shift: bool) -> Vec2 {
    if !shift {
        return m;
    }
    let v = m - anchor;
    let s = PI / 12.0;
    let a = (v.y.atan2(v.x) / s).round() * s;
    anchor + vec2(a.cos(), a.sin()) * v.length()
}
