//! Mirror Lab: play with mirrors, chirality and enantiomorphs.

mod app2d;
mod app3d;
mod geom;
mod shapes;
mod ui;

use macroquad::prelude::*;
use ui::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Mirror Lab - chirality & enantiomorphs".to_owned(),
        window_width: 1440,
        window_height: 880,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Shapes2D,
    Molecules3D,
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app2d = app2d::App2D::new();
    let mut app3d = app3d::App3D::new();
    let mut mode = Mode::Shapes2D;

    loop {
        clear_background(BG);

        if is_key_pressed(KeyCode::Key1) {
            mode = Mode::Shapes2D;
        }
        if is_key_pressed(KeyCode::Key2) {
            mode = Mode::Molecules3D;
        }

        match mode {
            Mode::Shapes2D => app2d.update_and_draw(),
            Mode::Molecules3D => app3d.update_and_draw(),
        }

        // Sidebar.
        draw_rectangle(0.0, 0.0, SIDEBAR_W, screen_height(), PANEL);
        draw_line(SIDEBAR_W, 0.0, SIDEBAR_W, screen_height(), 1.0, with_alpha(MIRROR, 0.2));
        let mut ui = Ui::new(16.0, 14.0, SIDEBAR_W - 32.0);
        ui.label("MIRROR LAB", 30.0, TEXT);
        ui.label("chirality & enantiomorphs", 17.0, DIM);
        ui.space(6.0);
        let active = if mode == Mode::Shapes2D { 0 } else { 1 };
        match ui.row(&["2D shapes [1]", "3D molecules [2]"], Some(active)) {
            Some(0) => mode = Mode::Shapes2D,
            Some(1) => mode = Mode::Molecules3D,
            _ => {}
        }
        match mode {
            Mode::Shapes2D => app2d.sidebar(&mut ui),
            Mode::Molecules3D => app3d.sidebar(&mut ui),
        }

        next_frame().await
    }
}
