//! Tiny immediate-mode sidebar UI drawn with macroquad primitives.

use macroquad::prelude::*;

pub const SIDEBAR_W: f32 = 310.0;

pub const BG: Color = Color::new(0.059, 0.071, 0.110, 1.0);
pub const PANEL: Color = Color::new(0.090, 0.106, 0.157, 1.0);
pub const TEXT: Color = Color::new(0.886, 0.910, 0.941, 1.0);
pub const DIM: Color = Color::new(0.580, 0.639, 0.722, 1.0);
pub const SAME: Color = Color::new(0.176, 0.831, 0.749, 1.0); // teal: same hand
pub const OPP: Color = Color::new(0.984, 0.573, 0.235, 1.0); // orange: opposite hand
pub const MIRROR: Color = Color::new(0.576, 0.773, 0.992, 1.0);
pub const COPY: Color = Color::new(0.769, 0.710, 0.992, 1.0);
pub const GOOD: Color = Color::new(0.290, 0.871, 0.502, 1.0);

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

pub fn mouse() -> Vec2 {
    let (x, y) = mouse_position();
    vec2(x, y)
}

pub fn in_canvas(p: Vec2) -> bool {
    p.x > SIDEBAR_W
}

pub fn dashed_line(a: Vec2, b: Vec2, dash: f32, thick: f32, col: Color) {
    let len = a.distance(b);
    if len < 1e-3 {
        return;
    }
    let d = (b - a) / len;
    let mut t = 0.0;
    while t < len {
        let e = (t + dash).min(len);
        let (p, q) = (a + d * t, a + d * e);
        draw_line(p.x, p.y, q.x, q.y, thick, col);
        t += dash * 2.0;
    }
}

pub struct Ui {
    x: f32,
    pub y: f32,
    w: f32,
    m: Vec2,
    clicked: bool,
}

impl Ui {
    pub fn new(x: f32, y: f32, w: f32) -> Ui {
        Ui { x, y, w, m: mouse(), clicked: is_mouse_button_pressed(MouseButton::Left) }
    }

    pub fn space(&mut self, h: f32) {
        self.y += h;
    }

    pub fn label(&mut self, s: &str, size: f32, col: Color) {
        draw_text(s, self.x, self.y + size * 0.75, size, col);
        self.y += size * 0.95;
    }

    pub fn heading(&mut self, s: &str) {
        self.space(6.0);
        draw_text(s, self.x, self.y + 14.0, 19.0, MIRROR);
        draw_line(self.x, self.y + 20.0, self.x + self.w, self.y + 20.0, 1.0, with_alpha(MIRROR, 0.25));
        self.y += 26.0;
    }

    pub fn wrapped(&mut self, s: &str, size: f32, col: Color) {
        let mut line = String::new();
        for word in s.split_whitespace() {
            let trial = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if measure_text(&trial, None, size as u16, 1.0).width > self.w && !line.is_empty() {
                self.label(&line, size, col);
                line = word.to_string();
            } else {
                line = trial;
            }
        }
        if !line.is_empty() {
            self.label(&line, size, col);
        }
    }

    fn button_rect(&self, r: Rect, label: &str, active: bool) -> bool {
        let hover = r.contains(self.m);
        let fill = if active {
            with_alpha(MIRROR, 0.35)
        } else if hover {
            Color::new(0.18, 0.21, 0.29, 1.0)
        } else {
            Color::new(0.13, 0.15, 0.21, 1.0)
        };
        draw_rectangle(r.x, r.y, r.w, r.h, fill);
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, with_alpha(MIRROR, if hover { 0.7 } else { 0.3 }));
        let size = 18.0;
        let tw = measure_text(label, None, size as u16, 1.0).width;
        draw_text(label, r.x + (r.w - tw) / 2.0, r.y + r.h / 2.0 + 5.0, size, TEXT);
        hover && self.clicked
    }

    pub fn button(&mut self, label: &str) -> bool {
        let r = Rect::new(self.x, self.y, self.w, 26.0);
        self.y += 31.0;
        self.button_rect(r, label, false)
    }

    /// A row of equal-width buttons; returns the index clicked. `active` highlights one.
    pub fn row(&mut self, labels: &[&str], active: Option<usize>) -> Option<usize> {
        let gap = 6.0;
        let n = labels.len() as f32;
        let bw = (self.w - gap * (n - 1.0)) / n;
        let mut hit = None;
        for (i, l) in labels.iter().enumerate() {
            let r = Rect::new(self.x + i as f32 * (bw + gap), self.y, bw, 26.0);
            if self.button_rect(r, l, active == Some(i)) {
                hit = Some(i);
            }
        }
        self.y += 31.0;
        hit
    }

    pub fn toggle(&mut self, label: &str, v: &mut bool) {
        let r = Rect::new(self.x, self.y, self.w, 22.0);
        let hover = r.contains(self.m);
        draw_rectangle_lines(self.x, self.y + 3.0, 16.0, 16.0, 1.5, if hover { TEXT } else { DIM });
        if *v {
            draw_rectangle(self.x + 4.0, self.y + 7.0, 8.0, 8.0, SAME);
        }
        draw_text(label, self.x + 24.0, self.y + 16.0, 18.0, TEXT);
        if hover && self.clicked {
            *v = !*v;
        }
        self.y += 25.0;
    }

    pub fn swatch(&mut self, col: Color, label: &str) {
        draw_rectangle(self.x, self.y + 3.0, 16.0, 14.0, with_alpha(col, 0.55));
        draw_rectangle_lines(self.x, self.y + 3.0, 16.0, 14.0, 1.5, col);
        draw_text(label, self.x + 24.0, self.y + 15.0, 17.0, TEXT);
        self.y += 21.0;
    }
}
