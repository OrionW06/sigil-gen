use macroquad::prelude::*;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const CIRCLE_RADIUS: f32 = 250.0;
const ANIMATION_SPEED: f32 = 3.0;

#[derive(Clone)]
struct SigilPoint {
    // Store as relative position from center (0,0 = center)
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
}

impl SigilApp {
    fn new() -> Self {
        Self {
            state: State::Start,
            intention: String::new(),
            points: Vec::new(),
            blink_timer: 0.0,
            save_timer: 0.0,
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

        // Remove vowels and duplicates
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

        // Convert to numbers and shuffle
        let mut numbers: Vec<u8> = filtered
        .chars()
        .map(|c| if c.is_ascii_digit() {
            c as u8 - b'0'
        } else {
            (c as u8 - b'a') % 10
        })
        .collect();

        // Fisher-Yates shuffle
        for i in (1..numbers.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            numbers.swap(i, j);
        }

        // Generate points with random angles - store as relative positions
        let mut angles: Vec<f32> = (0..numbers.len())
        .map(|i| (i as f32 / numbers.len() as f32) * 2.0 * PI)
        .collect();

        // Add randomness
        for angle in &mut angles {
            *angle += rand::gen_range(-0.2, 0.2);
        }

        // Shuffle angles
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
        // Create directory
        let dir = "sigils";
        if !Path::new(dir).exists() {
            std::fs::create_dir(dir)?;
        }

        // Generate filename
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let sanitized_intention = self.intention
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>();
        let filename = format!("{}/sigil_{}_{}.svg", dir, timestamp, sanitized_intention);

        // Create SVG
        let mut file = File::create(filename)?;

        // SVG dimensions and center
        let svg_size = 600.0;
        let svg_center = svg_size / 2.0;

        // SVG header
        writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>")?;
        writeln!(file, "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">",
                 svg_size, svg_size, svg_size, svg_size)?;

                 // Background (black) - comment out this line to remove black background
                 // writeln!(file, "<rect width=\"100%\" height=\"100%\" fill=\"black\" />")?;

                 // Outer circle - properly centered
                 writeln!(file, "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" stroke=\"gray\" stroke-width=\"3\" fill=\"none\" />",
                          svg_center, svg_center, CIRCLE_RADIUS)?;

                          // Transform points from relative coordinates to SVG coordinates
                          let transform_point = |relative_pos: Vec2| -> (f32, f32) {
                              (svg_center + relative_pos.x, svg_center + relative_pos.y)
                          };

                          // Sigil lines
                          if self.points.len() > 1 {
                              let points_str: Vec<String> = self.points.iter()
                              .map(|p| {
                                  let (x, y) = transform_point(p.relative_pos);
                                  format!("{},{}", x, y)
                              })
                              .collect();

                              writeln!(file, "<polyline points=\"{}\" fill=\"none\" stroke=\"#87ceeb\" stroke-width=\"3\" />",
                                       points_str.join(" "))?;
                          }

                          // Only draw start and end points (no numbers)
                          if !self.points.is_empty() {
                              // Start point (green)
                              let (start_x, start_y) = transform_point(self.points[0].relative_pos);
                              writeln!(file, "<circle cx=\"{}\" cy=\"{}\" r=\"10\" fill=\"green\" />",
                                       start_x, start_y)?;

                                       // End point (red) - only if there's more than one point
                                       if self.points.len() > 1 {
                                           let last_idx = self.points.len() - 1;
                                           let (end_x, end_y) = transform_point(self.points[last_idx].relative_pos);
                                           writeln!(file, "<circle cx=\"{}\" cy=\"{}\" r=\"10\" fill=\"red\" />",
                                                    end_x, end_y)?;
                                       }
                          }

                          // Intention text at bottom
                          writeln!(file, "<text x=\"{}\" y=\"{}\" font-size=\"20\" text-anchor=\"middle\" fill=\"black\">{}</text>",
                                   svg_center, svg_size - 30.0, self.intention)?;

                                   // Close SVG
                                   writeln!(file, "</svg>")?;

                                   Ok(())
    }

    fn update(&mut self) {
        self.blink_timer += get_frame_time();

        if matches!(self.state, State::Saving) {
            self.save_timer += get_frame_time();
            if self.save_timer > 1.0 {
                self.state = State::Display;
                self.save_timer = 0.0;
            }
        }

        // Consume character input in all states to prevent unwanted text entry
        match &mut self.state {
            State::Start => {
                // Consume any character input to prevent it from being used later
                while get_char_pressed().is_some() {}

                if is_key_pressed(KeyCode::Space) {
                    self.state = State::Input;
                }
            }
            State::Input => {
                // Handle text input ONLY when in Input state
                while let Some(ch) = get_char_pressed() {
                    if ch.is_ascii_alphanumeric() || ch == ' ' {
                        if self.intention.len() < 100 {
                            self.intention.push(ch);
                        }
                    }
                }

                if is_key_pressed(KeyCode::Backspace) {
                    self.intention.pop();
                }

                if is_key_pressed(KeyCode::Enter) && !self.intention.trim().is_empty() {
                    self.generate_sigil();
                }
            }
            State::Display => {
                // Consume any character input to prevent it from being used later
                while get_char_pressed().is_some() {}

                if is_key_pressed(KeyCode::Space) && self.points.len() > 1 {
                    self.state = State::Animating { progress: 0.0, line: 0 };
                } else if is_key_pressed(KeyCode::R) {
                    self.reset();
                } else if is_key_pressed(KeyCode::S) {
                    if let Err(e) = self.save_sigil() {
                        eprintln!("Failed to save sigil: {}", e);
                    }
                    self.state = State::Saving;
                }
            }
            State::Animating { progress, line } => {
                // Consume any character input to prevent it from being used later
                while get_char_pressed().is_some() {}

                *progress += get_frame_time() * ANIMATION_SPEED;
                if *progress >= 1.0 {
                    *progress = 0.0;
                    *line += 1;
                    if *line >= self.points.len() - 1 {
                        self.state = State::Display;
                    }
                }
            }
            State::Saving => {
                // Consume any character input to prevent it from being used later
                while get_char_pressed().is_some() {}
            }
        }
    }

    fn reset(&mut self) {
        self.state = State::Input;
        self.intention.clear();
        self.points.clear();
        self.blink_timer = 0.0; // Reset blink timer so cursor starts fresh
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(10, 5, 20, 255));

        match &self.state {
            State::Start => self.draw_start(),
            State::Input => self.draw_input(),
            State::Display => self.draw_sigil(None),
            State::Animating { progress, line } => self.draw_sigil(Some((*line, *progress))),
            State::Saving => {
                self.draw_sigil(None);
                self.draw_saving_message();
            }
        }
    }

    fn draw_start(&self) {
        let center = self.get_center();

        draw_text_ex(
            "CHAOS SIGIL GENERATOR",
            center.x - 200.0,
            center.y - 50.0,
            TextParams {
                font_size: 32,
                color: WHITE,
                ..Default::default()
            },
        );

        draw_text_ex(
            "Press SPACE to begin",
            center.x - 120.0,
            center.y + 20.0,
            TextParams {
                font_size: 24,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
    }

    fn draw_input(&self) {
        let center = self.get_center();

        // Draw circle
        draw_circle_lines(center.x, center.y, CIRCLE_RADIUS, 3.0, GRAY);

        // Instructions
        draw_text_ex(
            "Enter your intention:",
            center.x - 150.0,
            center.y - 150.0,
            TextParams {
                font_size: 24,
                color: WHITE,
                ..Default::default()
            },
        );

        // Text input with cursor
        let cursor = if (self.blink_timer * 2.0) as i32 % 2 == 0 { "|" } else { " " };
        let display_text = format!("{}{}", self.intention, cursor);

        draw_text_ex(
            &display_text,
            center.x - 200.0,
            center.y - 100.0,
            TextParams {
                font_size: 20,
                color: YELLOW,
                ..Default::default()
            },
        );

        draw_text_ex(
            "Press ENTER when done",
            center.x - 120.0,
            center.y + 150.0,
            TextParams {
                font_size: 18,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
    }

    fn draw_sigil(&self, animation: Option<(usize, f32)>) {
        let center = self.get_center();

        // Draw circle
        draw_circle_lines(center.x, center.y, CIRCLE_RADIUS, 3.0, GRAY);

        if self.points.is_empty() {
            return;
        }

        // Draw completed lines
        let completed_lines = match animation {
            Some((current_line, _)) => current_line,
            None => self.points.len() - 1,
        };

        for i in 0..completed_lines {
            if i + 1 < self.points.len() {
                let start_pos = self.get_absolute_pos(&self.points[i]);
                let end_pos = self.get_absolute_pos(&self.points[i + 1]);
                draw_line(
                    start_pos.x,
                    start_pos.y,
                    end_pos.x,
                    end_pos.y,
                    3.0,
                    SKYBLUE,
                );
            }
        }

        // Draw current animating line
        if let Some((current_line, progress)) = animation {
            if current_line + 1 < self.points.len() {
                let start_pos = self.get_absolute_pos(&self.points[current_line]);
                let end_pos = self.get_absolute_pos(&self.points[current_line + 1]);
                let current_pos = start_pos + (end_pos - start_pos) * progress;

                draw_line(start_pos.x, start_pos.y, current_pos.x, current_pos.y, 3.0, SKYBLUE);
            }
        }

        // Draw points with numbers
        for (i, point) in self.points.iter().enumerate() {
            let pos = self.get_absolute_pos(point);
            let color = if i == 0 {
                GREEN
            } else if i == self.points.len() - 1 {
                RED
            } else {
                ORANGE
            };
            draw_circle(pos.x, pos.y, 10.0, color);

            // Draw the number inside the circle
            let number_text = point.number.to_string();
            let text_size = measure_text(&number_text, None, 16, 1.0);
            draw_text_ex(
                &number_text,
                pos.x - text_size.width / 2.0,
                pos.y + text_size.height / 2.0,
                TextParams {
                    font_size: 16,
                    color: BLACK,
                    ..Default::default()
                },
            );
        }

        // Instructions
        if matches!(self.state, State::Display) {
            draw_text_ex(
                "SPACE: Animate | R: Reset | S: Save",
                20.0,
                screen_height() - 30.0,
                         TextParams {
                             font_size: 16,
                             color: LIGHTGRAY,
                             ..Default::default()
                         },
            );
        }
    }

    fn draw_saving_message(&self) {
        let center = self.get_center();

        // Semi-transparent background
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

#[macroquad::main("Chaos Sigil Generator")]
async fn main() {
    let mut app = SigilApp::new();

    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
