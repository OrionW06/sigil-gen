use macroquad::prelude::*;
use miniquad::conf::{Conf, Platform, LinuxBackend};
use std::collections::HashSet;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const CIRCLE_RADIUS: f32 = 250.0;
const ANIMATION_SPEED: f32 = 3.0;

#[derive(Clone)]
struct SigilPoint {
    relative_pos: Vec2,
    number: u8,
}

#[derive(Clone)]
enum State {
    Start,
    Input,
    Display,
    Animating { progress: f32, line: usize },
    Saving,
}

struct SigilApp {
    state: State,
    intention: String,
    points: Vec<SigilPoint>,
    blink_timer: f32,
    save_timer: f32,
    cursor_pos: usize,
    selection_start: Option<usize>,
}

impl SigilApp {
    fn new() -> Self {
        Self {
            state: State::Start,
            intention: String::new(),
            points: Vec::new(),
            blink_timer: 0.0,
            save_timer: 0.0,
            cursor_pos: 0,
            selection_start: None,
        }
    }

    fn get_center(&self) -> Vec2 {
        vec2(screen_width() / 2.0, screen_height() / 2.0)
    }

    fn get_absolute_pos(&self, point: &SigilPoint) -> Vec2 {
        self.get_center() + point.relative_pos
    }

    fn generate_sigil(&mut self) {
        if self.intention.trim().is_empty() {
            return;
        }

        let vowels = "aeiouAEIOU";
        let mut seen = HashSet::new();
        let filtered: String = self.intention
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() && !vowels.contains(*c))
            .map(|c| c.to_ascii_lowercase())
            .filter(|c| seen.insert(*c))
            .collect();

        if filtered.is_empty() {
            return;
        }

        let mut numbers: Vec<u8> = filtered
            .chars()
            .map(|c| if c.is_ascii_digit() {
                c as u8 - b'0'
            } else {
                (c as u8 - b'a') % 10
            })
            .collect();

        for i in (1..numbers.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            numbers.swap(i, j);
        }

        let mut angles: Vec<f32> = (0..numbers.len())
            .map(|i| (i as f32 / numbers.len() as f32) * 2.0 * PI)
            .collect();

        for angle in &mut angles {
            *angle += rand::gen_range(-0.2, 0.2);
        }

        for i in (1..angles.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            angles.swap(i, j);
        }

        self.points = numbers
            .into_iter()
            .zip(angles)
            .map(|(num, angle)| {
                SigilPoint {
                    relative_pos: vec2(angle.cos(), angle.sin()) * CIRCLE_RADIUS,
                    number: num,
                }
            })
            .collect();

        self.state = State::Display;
    }

    fn save_sigil(&self) -> std::io::Result<()> {
        let dir = "sigils";
        if !Path::new(dir).exists() {
            std::fs::create_dir(dir)?;
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let sanitized_intention = self.intention
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>();
        let filename = format!("{}/sigil_{}_{}.svg", dir, timestamp, sanitized_intention);

        let mut file = File::create(filename)?;

        let svg_size = 600.0;
        let svg_center = svg_size / 2.0;

        writeln!(file, r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#)?;
        writeln!(file, r#"<svg width="{}" height="{}" viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">"#,
                 svg_size, svg_size, svg_size, svg_size)?;

        // ... (rest of SVG generation code remains identical) ...

        Ok(())
    }

    // ... (include ALL other methods exactly as they were) ...

    fn draw_saving_message(&self) {
        let center = self.get_center();
        draw_rectangle(
            center.x - 150.0,
            center.y - 50.0,
            300.0,
            100.0,
            Color::from_rgba(0, 0, 0, 200),
        );
        draw_text_ex(
            "Sigil Saved!",
            center.x - 60.0,
            center.y - 10.0,
            TextParams {
                font_size: 24,
                color: GREEN,
                ..Default::default()
            },
        );
    }
}

#[cfg(target_os = "windows")]
fn window_conf() -> Conf {
    Conf {
        window_title: "Chaos Sigil Generator".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        platform: Platform {
            linux_backend: LinuxBackend::X11Only,
            swap_interval: Some(1),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[cfg(not(target_os = "windows"))]
fn window_conf() -> Conf {
    Conf {
        window_title: "Chaos Sigil Generator".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = SigilApp::new();
    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
