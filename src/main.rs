//! Mirror Lab: play with mirrors, chirality and enantiomorphs.

mod app2d;
mod app3d;
mod apphands;
mod geom;
mod handmesh;
mod render3d;
mod shapes;
mod symmetry3d;
mod theory;
mod ui;

use macroquad::prelude::*;
use theory::Jump;
use ui::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Mirror Lab - chirality & enantiomorphs".to_owned(),
        window_width: 1360,
        window_height: 800,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Shapes2D,
    Molecules3D,
    Hands,
    Theory,
}

#[macroquad::main(window_conf)]
async fn main() {
    load_font();
    let mut app2d = app2d::App2D::new();
    let mut app3d = app3d::App3D::new();
    let mut hands = apphands::AppHands::new();
    let mut theory = theory::Theory::new();
    let mut mode = Mode::Shapes2D;
    let mut side_scroll = 0.0f32;
    let mut side_h = 0.0f32;

    // Optional start-up arguments: mirror-lab [2d|3d|hands|theory] [index] [seconds]
    let args: Vec<String> = std::env::args().collect();
    let index: Option<usize> = args.get(2).and_then(|a| a.parse().ok());
    match args.get(1).map(|s| s.as_str()) {
        Some("3d") => {
            mode = Mode::Molecules3D;
            if let Some(i) = index {
                app3d.set_model(i);
            }
        }
        Some("hands") => {
            mode = Mode::Hands;
            hands.goto(index.unwrap_or(0));
            if let Some(t) = args.get(3).and_then(|a| a.parse().ok()) {
                hands.set_time(t);
            }
        }
        Some("theory") => mode = Mode::Theory,
        Some("2d") => {
            if let Some(i) = index {
                app2d.select_shape(i);
            }
        }
        _ => {}
    }

    loop {
        clear_background(BG);

        for (key, m) in [
            (KeyCode::Key1, Mode::Shapes2D),
            (KeyCode::Key2, Mode::Molecules3D),
            (KeyCode::Key3, Mode::Hands),
            (KeyCode::Key4, Mode::Theory),
        ] {
            if is_key_pressed(key) {
                mode = m;
            }
        }

        let mut jump = None;
        match mode {
            Mode::Shapes2D => app2d.update_and_draw(),
            Mode::Molecules3D => app3d.update_and_draw(),
            Mode::Hands => hands.update_and_draw(),
            Mode::Theory => jump = theory.update_and_draw(),
        }

        // Sidebar.
        draw_rectangle(0.0, 0.0, SIDEBAR_W, screen_height(), PANEL);
        draw_line(SIDEBAR_W, 0.0, SIDEBAR_W, screen_height(), 1.0, with_alpha(MIRROR, 0.2));
        // The sidebar scrolls with the wheel when it is taller than the window.
        let over = (side_h - screen_height() + 10.0).max(0.0);
        if mouse().x < SIDEBAR_W {
            side_scroll -= mouse_wheel().1.signum() * 40.0;
        }
        side_scroll = side_scroll.clamp(0.0, over);
        let mut ui = Ui::new(16.0, 14.0 - side_scroll, SIDEBAR_W - 32.0);
        ui.label("MIRROR LAB", 28.0, TEXT);
        ui.label("chirality & enantiomorphs", 15.0, DIM);
        ui.space(6.0);
        let modes = [Mode::Shapes2D, Mode::Molecules3D, Mode::Hands, Mode::Theory];
        let active = modes.iter().position(|&m| m == mode);
        if let Some(i) = ui.row(&["2D [1]", "3D [2]", "Hands [3]", "Theory [4]"], active) {
            mode = modes[i];
        }
        match mode {
            Mode::Shapes2D => app2d.sidebar(&mut ui),
            Mode::Molecules3D => app3d.sidebar(&mut ui),
            Mode::Hands => hands.sidebar(&mut ui),
            Mode::Theory => theory.sidebar(&mut ui),
        }
        side_h = ui.y + side_scroll;
        if over > 0.0 {
            let sh = screen_height();
            let bh = sh * sh / side_h;
            let by = (sh - bh) * side_scroll / over;
            draw_rectangle(SIDEBAR_W - 5.0, by, 3.0, bh, with_alpha(MIRROR, 0.4));
        }

        match jump {
            Some(Jump::Flip2D) => {
                mode = Mode::Shapes2D;
                app2d.flip_demo();
            }
            Some(Jump::Shape(i)) => {
                mode = Mode::Shapes2D;
                app2d.select_shape(i);
            }
            Some(Jump::Model(name)) => {
                mode = Mode::Molecules3D;
                let i = app3d.model_index(name);
                app3d.set_model(i);
            }
            Some(Jump::Hand(i)) => {
                mode = Mode::Hands;
                hands.goto(i);
            }
            None => {}
        }

        next_frame().await
    }
}
