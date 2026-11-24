//! Theory mode: a scrollable primer on the mathematics of chirality, with
//! buttons that jump to the matching live demo.

use crate::ui::*;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub enum Jump {
    Flip2D,
    Shape(usize),
    Model(&'static str),
    Hand(usize),
}

enum Item {
    P(&'static str),
    F(&'static str),
    Try(&'static str, Jump),
}

struct Section {
    title: &'static str,
    items: &'static [Item],
}

use Item::*;

const INTRO: &str = "An object is chiral if no rotation or translation can make it coincide with its mirror image; otherwise it is achiral. Your hands are the standard example. Mathematically, chirality is about orientation: a mirror image is an image whose orientation has been reversed.";

const SECTIONS: &[Section] = &[
    Section {
        title: "1. Geometry and group theory",
        items: &[
            P("The rigid motions of 3D space form the Euclidean group E(3). Each element has a linear part, an orthogonal matrix A with det A = ±1."),
            F("det A = +1: proper motions (rotations and translations, SO(3)). These can be done physically."),
            F("det A = −1: improper motions (reflections, inversion, rotoreflections). These cannot."),
            P("Formal definition: X is chiral if no proper isometry maps X onto its mirror image. Equivalently, the symmetry group of X contains no improper element."),
            P("Point groups: a molecule is chiral exactly when its point group contains only rotations, which leaves the families Cn, Dn, T, O and I. Any mirror plane σ, inversion centre i or improper axis Sn makes it achiral. Strictly the test is 'no Sn axis of any order', since σ = S1 and i = S2."),
            P("Crystals: 65 of the 230 space groups (the Sohncke groups) contain only proper operations, so only these can hold chiral crystals. 22 of them form 11 mirror-image pairs, such as P3₁21 and P3₂21 (left- and right-handed quartz)."),
            P("Dimension matters: a 2D 'F' is chiral in the plane but can be flipped by rotating it through 3D. Likewise a 3D hand could be turned into its mirror image by a rotation through a fourth dimension."),
            Try("2D: flip an F through 3D", Jump::Flip2D),
            Try("Hands: rotate through 4D", Jump::Hand(3)),
            Try("Hands: why rotations fail", Jump::Hand(1)),
        ],
    },
    Section {
        title: "2. The sign of a determinant",
        items: &[
            P("The simplest way to compute handedness is a signed volume. For a tetrahedral stereocentre with substituents at v₁ … v₄:"),
            F("χ = sign det[v₁ − v₄,  v₂ − v₄,  v₃ − v₄] = sign((a × b) · c)"),
            P("A reflection flips this sign. Once the substituents are ranked by priority, the sign is the R/S label of the Cahn–Ingold–Prelog system. A molecule with n stereocentres has at most 2ⁿ stereoisomers; symmetric (meso) cases reduce that number."),
            Try("3D: R/S of CHFClBr", Jump::Model("CHFClBr")),
            Try("Hands: the handedness triad", Jump::Hand(2)),
        ],
    },
    Section {
        title: "3. Pseudoscalars and measuring chirality",
        items: &[
            P("A pseudoscalar behaves like a number under rotations but flips sign under a reflection. Examples: the triple product, the helicity of a vector field ∫ u · (curl u) dV, and optical rotation."),
            P("Continuous chirality measures ask 'how chiral' rather than yes/no. The Zabrodsky–Avnir CCM is the minimal distance from a shape to the nearest achiral shape. The 2D tab shows a vertex-based version, S, for every shape (S = 0 exactly when the shape is achiral)."),
            P("The chiral-zero theorem (Weinberg, Mislow and others): no continuous pseudoscalar can detect every chiral object; some chiral shapes always score zero. Chirality (yes/no) and handedness (a sign) are different things."),
            Try("2D: compare chirality measures", Jump::Shape(0)),
        ],
    },
    Section {
        title: "4. Curves and helices",
        items: &[
            P("The torsion τ of a space curve is a pseudoscalar, so its sign gives the curve's handedness. For the helix r(t) = (a cos t, a sin t, bt):"),
            F("τ = b / (a² + b²)"),
            P("τ > 0 means right-handed (like most DNA and screw threads); τ < 0 means left-handed."),
            Try("3D: helix and its torsion", Jump::Model("Helix")),
        ],
    },
    Section {
        title: "5. Topology: knots and manifolds",
        items: &[
            P("The trefoil is chiral: no deformation turns it into its mirror image. The figure-eight knot is amphichiral."),
            F("Mirror image of a knot:  V_K*(t) = V_K(t⁻¹)   (Jones polynomial)"),
            P("If the Jones polynomial is not symmetric under t ↔ t⁻¹, the knot is chiral. The knot signature also flips sign under mirroring."),
            F("Lk = Tw + Wr   (Călugăreanu–White–Fuller)"),
            P("This splits the integer linking number of a closed ribbon, such as circular DNA, into twist and writhe, both chiral quantities: the mathematics of DNA supercoiling. The 3D tab computes the writhe of each knot live."),
            P("A manifold is chiral if it has no orientation-reversing self-map. CP² has signature +1 and its reversed-orientation copy −1; signature is a topological invariant, so CP² is chiral."),
            Try("3D: trefoil knot", Jump::Model("Trefoil")),
            Try("3D: figure-eight knot", Jump::Model("Figure-eight")),
        ],
    },
    Section {
        title: "6. Physics: parity, helicity and γ⁵",
        items: &[
            F("Parity P: x → −x  (improper)       Helicity h = S · p / |p|  (pseudoscalar)"),
            P("Chirality in the Dirac equation is the eigenvalue of γ⁵. The projectors P_L = ½(1 − γ⁵), P_R = ½(1 + γ⁵) split a fermion into left- and right-handed parts; for massless particles chirality and helicity coincide."),
            P("Parity violation: the weak force couples only to left-handed fermions (the V − A structure), so nature itself is chiral at a fundamental level (Wu experiment, 1957)."),
            P("Optics: chiral media respond differently to left- and right-circularly polarized light, producing optical rotation and circular dichroism, which is how enantiomers are told apart in the lab."),
            Try("Hands: helicity animation", Jump::Hand(4)),
        ],
    },
];

const OUTRO: &str = "The common thread: a mirror image is the action of an element with determinant −1. Chirality is the failure of an object's symmetry group to include any such element. Handedness is then read off whichever pseudoscalar suits the setting: a signed volume, torsion, a knot invariant or the γ⁵ eigenvalue.";

pub struct Theory {
    scroll: f32,
    content_h: f32,
    section_y: Vec<f32>,
}

impl Theory {
    pub fn new() -> Theory {
        Theory { scroll: 0.0, content_h: 2000.0, section_y: vec![0.0; SECTIONS.len()] }
    }

    /// Draws the page; returns a jump request if a "Try it" button was clicked.
    pub fn update_and_draw(&mut self) -> Option<Jump> {
        let wheel = mouse_wheel().1;
        if wheel != 0.0 && in_canvas(mouse()) {
            self.scroll -= wheel.signum() * 60.0;
        }
        if is_key_down(KeyCode::Down) {
            self.scroll += 600.0 * get_frame_time();
        }
        if is_key_down(KeyCode::Up) {
            self.scroll -= 600.0 * get_frame_time();
        }
        if is_key_pressed(KeyCode::PageDown) {
            self.scroll += screen_height() * 0.8;
        }
        if is_key_pressed(KeyCode::PageUp) {
            self.scroll -= screen_height() * 0.8;
        }
        let max = (self.content_h - screen_height() + 60.0).max(0.0);
        self.scroll = self.scroll.clamp(0.0, max);

        let avail = screen_width() - SIDEBAR_W;
        let w = avail.min(860.0) - 60.0;
        let x = SIDEBAR_W + (avail - w) / 2.0;
        let mut ui = Ui::new(x, 30.0 - self.scroll, w);
        let mut jump = None;

        ui.label("The mathematics of chirality", 30.0, TEXT);
        ui.space(6.0);
        ui.wrapped(INTRO, 17.0, DIM);
        for (si, sec) in SECTIONS.iter().enumerate() {
            ui.space(14.0);
            self.section_y[si] = ui.y + self.scroll;
            ui.label(sec.title, 22.0, MIRROR);
            draw_line(x, ui.y, x + w, ui.y, 1.0, with_alpha(MIRROR, 0.25));
            ui.space(8.0);
            let mut tries: Vec<(&str, Jump)> = Vec::new();
            for it in sec.items {
                match it {
                    P(t) => {
                        ui.wrapped(t, 16.0, TEXT);
                        ui.space(6.0);
                    }
                    F(t) => {
                        let h = 30.0;
                        draw_rectangle(x, ui.y - 2.0, w, h, with_alpha(MIRROR, 0.08));
                        draw_rectangle(x, ui.y - 2.0, 3.0, h, MIRROR);
                        text(t, x + 14.0, ui.y + 18.0, 17.0, Color::new(0.85, 0.92, 1.0, 1.0));
                        ui.y += h + 6.0;
                    }
                    Try(label, j) => tries.push((label, *j)),
                }
            }
            if !tries.is_empty() {
                let labels: Vec<String> = tries.iter().map(|(l, _)| format!("→ {l}")).collect();
                let refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
                if let Some(i) = ui.row(&refs, None) {
                    // Ignore clicks that land under the sidebar or off-screen.
                    if in_canvas(mouse()) {
                        jump = Some(tries[i].1);
                    }
                }
            }
        }
        ui.space(16.0);
        ui.wrapped(OUTRO, 17.0, GOOD);
        ui.space(40.0);
        self.content_h = ui.y + self.scroll;

        // Scrollbar.
        if max > 0.0 {
            let sh = screen_height();
            let bh = sh * sh / self.content_h;
            let by = (sh - bh) * self.scroll / max;
            draw_rectangle(screen_width() - 8.0, by, 4.0, bh, with_alpha(MIRROR, 0.4));
        }
        hint("Wheel / arrow keys / PgUp PgDn: scroll  ·  → buttons open the live demo");
        jump
    }

    pub fn sidebar(&mut self, ui: &mut Ui) {
        ui.heading("Contents");
        if ui.button("Introduction") {
            self.scroll = 0.0;
        }
        for (i, s) in SECTIONS.iter().enumerate() {
            if ui.button(s.title) {
                self.scroll = self.section_y[i] - 20.0;
            }
        }
        ui.space(6.0);
        ui.wrapped(
            "Each section links to the demo that shows it live: determinants in the 2D tab, point groups, R/S, torsion and writhe in the 3D tab, and animations in the Hands tab.",
            14.0,
            DIM,
        );
    }
}
