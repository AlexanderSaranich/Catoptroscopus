//! Tiny immediate-mode sidebar UI drawn with macroquad primitives.

use macroquad::prelude::*;
use std::cell::RefCell;

thread_local! {
    static FONT: RefCell<Option<Font>> = const { RefCell::new(None) };
}

/// Load a system font with Greek/math glyphs; falls back to macroquad's ASCII font.
pub fn load_font() {
    let candidates = [
        "C:/Windows/Fonts/segoeui.ttf",
        "C:/Windows/Fonts/arial.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/Library/Fonts/Arial Unicode.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(f) = load_ttf_font_from_bytes(&bytes) {
                FONT.with(|cell| *cell.borrow_mut() = Some(f));
                return;
            }
        }
    }
}

/// Draw text with the UI font; returns its width.
pub fn text(s: &str, x: f32, y: f32, size: f32, col: Color) -> f32 {
    FONT.with(|cell| {
        let f = cell.borrow();
        draw_text_ex(s, x, y, TextParams { font: f.as_ref(), font_size: size as u16, color: col, ..Default::default() })
            .width
    })
}

pub fn text_width(s: &str, size: f32) -> f32 {
    FONT.with(|cell| measure_text(s, cell.borrow().as_ref(), size as u16, 1.0).width)
}

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
    pub x: f32,
    pub y: f32,
    pub w: f32,
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
        text(s, self.x, self.y + size * 0.8, size, col);
        self.y += size * 1.12;
    }

    pub fn heading(&mut self, s: &str) {
        self.space(6.0);
        text(s, self.x, self.y + 15.0, 18.0, MIRROR);
        draw_line(self.x, self.y + 20.0, self.x + self.w, self.y + 20.0, 1.0, with_alpha(MIRROR, 0.25));
        self.y += 26.0;
    }

    pub fn wrapped(&mut self, s: &str, size: f32, col: Color) {
        let mut line = String::new();
        for word in s.split_whitespace() {
            let trial = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if text_width(&trial, size) > self.w && !line.is_empty() {
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
        let mut size = 16.0;
        while text_width(label, size) > r.w - 6.0 && size > 11.0 {
            size -= 1.0;
        }
        let tw = text_width(label, size);
        text(label, r.x + (r.w - tw) / 2.0, r.y + r.h / 2.0 + size * 0.35, size, TEXT);
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
        text(label, self.x + 24.0, self.y + 16.0, 16.0, TEXT);
        if hover && self.clicked {
            *v = !*v;
        }
        self.y += 25.0;
    }

    pub fn swatch(&mut self, col: Color, label: &str) {
        draw_rectangle(self.x, self.y + 3.0, 16.0, 14.0, with_alpha(col, 0.55));
        draw_rectangle_lines(self.x, self.y + 3.0, 16.0, 14.0, 1.5, col);
        text(label, self.x + 24.0, self.y + 15.0, 15.0, TEXT);
        self.y += 21.0;
    }
}

/// A translucent info box with lines of text, anchored at the canvas top-right.
pub fn info_box(title: &str, lines: &[(String, Color)]) {
    let size = 16.0;
    let w = lines
        .iter()
        .map(|(l, _)| text_width(l, size))
        .fold(text_width(title, 17.0), f32::max)
        + 24.0;
    let h = 30.0 + lines.len() as f32 * 20.0;
    let x = screen_width() - w - 14.0;
    let y = 14.0;
    draw_rectangle(x, y, w, h, with_alpha(PANEL, 0.88));
    draw_rectangle_lines(x, y, w, h, 1.0, with_alpha(MIRROR, 0.35));
    text(title, x + 12.0, y + 20.0, 17.0, MIRROR);
    for (i, (l, c)) in lines.iter().enumerate() {
        text(l, x + 12.0, y + 42.0 + i as f32 * 20.0, size, *c);
    }
}

/// One-line hint at the bottom-left of the canvas.
pub fn hint(s: &str) {
    draw_rectangle(SIDEBAR_W, screen_height() - 30.0, screen_width() - SIDEBAR_W, 30.0, with_alpha(BG, 0.9));
    text(s, SIDEBAR_W + 14.0, screen_height() - 12.0, 15.0, with_alpha(DIM, 0.85));
}

/// Format a signed number with a typographic minus sign.
pub fn sgn(v: f32, prec: usize) -> String {
    let v = if v.abs() < 0.5 * 10f32.powi(-(prec as i32)) { 0.0 } else { v };
    format!("{:+.*}", prec, v).replace('-', "−")
}

/// "+1" / "−1" for a sign.
pub fn pm1(v: f32) -> &'static str {
    if v < 0.0 {
        "−1"
    } else {
        "+1"
    }
}
