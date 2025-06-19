use macroquad::prelude::*;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// Constants for the sigil's appearance and animation
const CIRCLE_RADIUS: f32 = 250.0; // Radius of the main circle
const ANIMATION_SPEED: f32 = 3.0; // Speed of the sigil drawing animation

/// Represents a point in the sigil, with a relative position and a number label
#[derive(Clone)]
struct SigilPoint {
    // Position relative to the center of the circle
    relative_pos: Vec2,
    // The number associated with this point (0-9)
    number: u8,
}

/// Enum for the different states of the application
#[derive(Clone)]
enum State {
    Start,      // Initial screen
    Input,      // User is entering their intention
    Display,    // Sigil is displayed
    Animating { progress: f32, line: usize }, // Sigil is being animated
    Saving,     // Sigil is being saved
}

/// Main application struct holding all state
struct SigilApp {
    state: State,                // Current state of the app
    intention: String,           // User's intention text
    points: Vec<SigilPoint>,     // Points that make up the sigil
    blink_timer: f32,            // Timer for blinking cursor
    save_timer: f32,             // Timer for save message
    cursor_pos: usize,           // Cursor position in the input string
    selection_start: Option<usize>, // Start of text selection (if any)
}

impl SigilApp {
    /// Create a new SigilApp with default state
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

    /// Get the center of the screen as a Vec2
    fn get_center(&self) -> Vec2 {
        vec2(screen_width() / 2.0, screen_height() / 2.0)
    }

    /// Convert a SigilPoint's relative position to an absolute screen position
    fn get_absolute_pos(&self, point: &SigilPoint) -> Vec2 {
        self.get_center() + point.relative_pos
    }

    /// Generate the sigil points from the user's intention
    fn generate_sigil(&mut self) {
        if self.intention.trim().is_empty() {
            return;
        }

        // Remove vowels and duplicate characters from the intention
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

        // Convert filtered characters to numbers (0-9)
        let mut numbers: Vec<u8> = filtered
            .chars()
            .map(|c| if c.is_ascii_digit() {
                c as u8 - b'0'
            } else {
                (c as u8 - b'a') % 10
            })
            .collect();

        // Shuffle the numbers using Fisher-Yates
        for i in (1..numbers.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            numbers.swap(i, j);
        }

        // Generate random angles for each point
        let mut angles: Vec<f32> = (0..numbers.len())
            .map(|i| (i as f32 / numbers.len() as f32) * 2.0 * PI)
            .collect();

        // Add randomness to the angles
        for angle in &mut angles {
            *angle += rand::gen_range(-0.2, 0.2);
        }

        // Shuffle the angles
        for i in (1..angles.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            angles.swap(i, j);
        }

        // Create the sigil points from the numbers and angles
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

    /// Save the current sigil as an SVG file
    fn save_sigil(&self) -> std::io::Result<()> {
        // Create output directory if it doesn't exist
        let dir = "sigils";
        if !Path::new(dir).exists() {
            std::fs::create_dir(dir)?;
        }

        // Generate a filename with timestamp and sanitized intention
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let sanitized_intention = self.intention
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>();
        let filename = format!("{}/sigil_{}_{}.svg", dir, timestamp, sanitized_intention);

        // Create SVG file
        let mut file = File::create(filename)?;

        // SVG dimensions and center
        let svg_size = 600.0;
        let svg_center = svg_size / 2.0;

        // SVG header
        writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>")?;
        writeln!(file, "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\">",
                 svg_size, svg_size, svg_size, svg_size)?;

        // Outer circle
        writeln!(file, "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" stroke=\"gray\" stroke-width=\"3\" fill=\"none\" />",
                 svg_center, svg_center, CIRCLE_RADIUS)?;

        // Helper closure to convert relative to SVG coordinates
        let transform_point = |relative_pos: Vec2| -> (f32, f32) {
            (svg_center + relative_pos.x, svg_center + relative_pos.y)
        };

        // Draw the sigil lines as a polyline
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

        // Draw start (green) and end (red) points
        if !self.points.is_empty() {
            // Start point
            let (start_x, start_y) = transform_point(self.points[0].relative_pos);
            writeln!(file, "<circle cx=\"{}\" cy=\"{}\" r=\"10\" fill=\"green\" />",
                     start_x, start_y)?;

            // End point (if more than one point)
            if self.points.len() > 1 {
                let last_idx = self.points.len() - 1;
                let (end_x, end_y) = transform_point(self.points[last_idx].relative_pos);
                writeln!(file, "<circle cx=\"{}\" cy=\"{}\" r=\"10\" fill=\"red\" />",
                         end_x, end_y)?;
            }
        }

        // Draw the intention text at the bottom of the SVG
        writeln!(file, "<text x=\"{}\" y=\"{}\" font-size=\"20\" text-anchor=\"middle\" fill=\"black\">{}</text>",
                 svg_center, svg_size - 30.0, self.intention)?;

        // Close SVG
        writeln!(file, "</svg>")?;

        Ok(())
    }

    /// Handle text input, cursor movement, and selection (ASCII only)
    fn handle_text_input(&mut self) {
        // Handle character input (ASCII alphanumeric and space only)
        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii_alphanumeric() || ch == ' ' {
                // If there's a selection, delete it first
                if let Some(start) = self.selection_start {
                    let (start, end) = if start < self.cursor_pos {
                        (start, self.cursor_pos)
                    } else {
                        (self.cursor_pos, start)
                    };
                    self.intention.drain(start..end);
                    self.cursor_pos = start;
                    self.selection_start = None;
                }
                // Insert character at cursor
                if self.intention.len() < 100 {
                    self.intention.insert(self.cursor_pos, ch);
                    self.cursor_pos += 1;
                }
            }
        }

        // Handle backspace
        if is_key_pressed(KeyCode::Backspace) {
            if let Some(start) = self.selection_start {
                // Delete selection
                let (start, end) = if start < self.cursor_pos {
                    (start, self.cursor_pos)
                } else {
                    (self.cursor_pos, start)
                };
                self.intention.drain(start..end);
                self.cursor_pos = start;
                self.selection_start = None;
            } else if self.cursor_pos > 0 {
                // Delete character before cursor
                self.cursor_pos -= 1;
                self.intention.remove(self.cursor_pos);
            }
        }

        // Handle delete
        if is_key_pressed(KeyCode::Delete) {
            if let Some(start) = self.selection_start {
                // Delete selection
                let (start, end) = if start < self.cursor_pos {
                    (start, self.cursor_pos)
                } else {
                    (self.cursor_pos, start)
                };
                self.intention.drain(start..end);
                self.cursor_pos = start;
                self.selection_start = None;
            } else if self.cursor_pos < self.intention.len() {
                // Delete character after cursor
                self.intention.remove(self.cursor_pos);
            }
        }

        // Handle left arrow (with/without selection)
        if is_key_pressed(KeyCode::Left) {
            if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                // Extend selection left
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    if self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor_pos + 1);
                    }
                }
            } else {
                // Move cursor left
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                self.selection_start = None;
            }
        }

        // Handle right arrow (with/without selection)
        if is_key_pressed(KeyCode::Right) {
            if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                // Extend selection right
                if self.cursor_pos < self.intention.len() {
                    if self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor_pos);
                    }
                    self.cursor_pos += 1;
                }
            } else {
                // Move cursor right
                if self.cursor_pos < self.intention.len() {
                    self.cursor_pos += 1;
                }
                self.selection_start = None;
            }
        }

        // Handle Home/End keys
        if is_key_pressed(KeyCode::Home) {
            if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                if self.selection_start.is_none() {
                    self.selection_start = Some(self.cursor_pos);
                }
            } else {
                self.selection_start = None;
            }
            self.cursor_pos = 0;
        }
        if is_key_pressed(KeyCode::End) {
            if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                if self.selection_start.is_none() {
                    self.selection_start = Some(self.cursor_pos);
                }
            } else {
                self.selection_start = None;
            }
            self.cursor_pos = self.intention.len();
        }

        // Handle Ctrl+A (Select All)
        if is_key_pressed(KeyCode::A) && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            self.selection_start = Some(0);
            self.cursor_pos = self.intention.len();
        }

        // Handle Ctrl+C (Copy) - prints to console for now
        if is_key_pressed(KeyCode::C) && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            if let Some(start) = self.selection_start {
                let (start, end) = if start < self.cursor_pos {
                    (start, self.cursor_pos)
                } else {
                    (self.cursor_pos, start)
                };
                let selected_text = &self.intention[start..end];
                println!("Copied: {}", selected_text);
            }
        }

        // Handle Ctrl+V (Paste) - inserts placeholder text for now
        if is_key_pressed(KeyCode::V) && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            let paste_text = "pasted_text"; // Placeholder for clipboard
            if self.intention.len() + paste_text.len() <= 100 {
                // Delete selection if any
                if let Some(start) = self.selection_start {
                    let (start, end) = if start < self.cursor_pos {
                        (start, self.cursor_pos)
                    } else {
                        (self.cursor_pos, start)
                    };
                    self.intention.drain(start..end);
                    self.cursor_pos = start;
                    self.selection_start = None;
                }
                // Insert pasted text (ASCII only)
                for ch in paste_text.chars() {
                    if ch.is_ascii_alphanumeric() || ch == ' ' {
                        self.intention.insert(self.cursor_pos, ch);
                        self.cursor_pos += 1;
                    }
                }
            }
        }

        // Handle Ctrl+X (Cut) - prints to console for now
        if is_key_pressed(KeyCode::X) && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)) {
            if let Some(start) = self.selection_start {
                let (start, end) = if start < self.cursor_pos {
                    (start, self.cursor_pos)
                } else {
                    (self.cursor_pos, start)
                };
                let selected_text = &self.intention[start..end];
                println!("Cut: {}", selected_text);
                self.intention.drain(start..end);
                self.cursor_pos = start;
                self.selection_start = None;
            }
        }
    }

    /// Update the application state each frame
    fn update(&mut self) {
        self.blink_timer += get_frame_time();

        // Handle save timer
        if matches!(self.state, State::Saving) {
            self.save_timer += get_frame_time();
            if self.save_timer > 1.0 {
                self.state = State::Display;
                self.save_timer = 0.0;
            }
        }

        // State machine for the app
        match &mut self.state {
            State::Start => {
                // Consume any character input
                while get_char_pressed().is_some() {}
                if is_key_pressed(KeyCode::Space) {
                    self.state = State::Input;
                }
            }
            State::Input => {
                // Handle text input and editing
                self.handle_text_input();
                if is_key_pressed(KeyCode::Enter) && !self.intention.trim().is_empty() {
                    self.generate_sigil();
                }
            }
            State::Display => {
                // Consume any character input
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
                // Consume any character input
                while get_char_pressed().is_some() {}
                // Animate the drawing of the sigil
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
                // Consume any character input
                while get_char_pressed().is_some() {}
            }
        }
    }

    /// Reset the app to the input state
    fn reset(&mut self) {
        self.state = State::Input;
        self.intention.clear();
        self.points.clear();
        self.blink_timer = 0.0;
        self.cursor_pos = 0;
        self.selection_start = None;
    }

    /// Draw the current frame
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

    /// Draw the start screen
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

    /// Draw the input screen with text box, cursor, and selection
    fn draw_input(&self) {
        let center = self.get_center();
        // Draw the main circle
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
        // Blinking cursor
        let cursor = if (self.blink_timer * 2.0) as i32 % 2 == 0 { "|" } else { " " };
        // Text box position
        let text_x = center.x - 200.0;
        let text_y = center.y - 100.0;
        // Draw selection background if any
        if let Some(selection_start) = self.selection_start {
            let (start, end) = if selection_start < self.cursor_pos {
                (selection_start, self.cursor_pos)
            } else {
                (self.cursor_pos, selection_start)
            };
            let before_selection = &self.intention[..start];
            let selection_text = &self.intention[start..end];
            let before_width = measure_text(before_selection, None, 20, 1.0).width;
            let selection_width = measure_text(selection_text, None, 20, 1.0).width;
            draw_rectangle(
                text_x + before_width,
                text_y - 15.0,
                selection_width,
                25.0,
                Color::from_rgba(100, 150, 255, 100),
            );
        }
        // Draw the text
        draw_text_ex(
            &self.intention,
            text_x,
            text_y,
            TextParams {
                font_size: 20,
                color: YELLOW,
                ..Default::default()
            },
        );
        // Draw the cursor at the correct position
        let cursor_x = text_x + measure_text(&self.intention[..self.cursor_pos], None, 20, 1.0).width;
        draw_text_ex(
            cursor,
            cursor_x,
            text_y,
            TextParams {
                font_size: 20,
                color: YELLOW,
                ..Default::default()
            },
        );
        // Input instructions
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

    /// Draw the sigil and its points, optionally animating the lines
    fn draw_sigil(&self, animation: Option<(usize, f32)>) {
        let center = self.get_center();
        // Draw the main circle
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
        // Draw the currently animating line
        if let Some((current_line, progress)) = animation {
            if current_line + 1 < self.points.len() {
                let start_pos = self.get_absolute_pos(&self.points[current_line]);
                let end_pos = self.get_absolute_pos(&self.points[current_line + 1]);
                let current_pos = start_pos + (end_pos - start_pos) * progress;
                draw_line(start_pos.x, start_pos.y, current_pos.x, current_pos.y, 3.0, SKYBLUE);
            }
        }
        // Draw the points with numbers
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
        // Display instructions at the bottom
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

    /// Draw the 'Sigil Saved!' message overlay
    fn draw_saving_message(&self) {
        let center = self.get_center();
        // Draw a semi-transparent background
        draw_rectangle(
            center.x - 150.0,
            center.y - 50.0,
            300.0,
            100.0,
            Color::from_rgba(0, 0, 0, 200),
        );
        // Draw the message
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

/// Main entry point for the Macroquad application
#[macroquad::main("Chaos Sigil Generator")]
async fn main() {
    let mut app = SigilApp::new();
    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
