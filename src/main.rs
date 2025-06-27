use macroquad::prelude::*;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::path::Path;

// TODO: Figure out why it doesn't wanna work on Windows Proper (in QEMU) but it works under WINE
// TODO: This code could probably be somewhat refactored



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

    /// Save the current sigil as a PNG file
    fn save_sigil(&self) -> std::io::Result<()> {
        use macroquad::texture::Image;
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
        let filename = format!("{}/sigil_{}_{}.png", dir, timestamp, sanitized_intention);

        // PNG dimensions and center
        let img_size = 600u16;
        let img_center = img_size as f32 / 2.0;
        let mut image = Image::gen_image_color(img_size, img_size, Color::from_rgba(10, 5, 20, 255));

        // Helper closure to convert relative to image coordinates
        let transform_point = |relative_pos: Vec2| -> (u32, u32) {
            let x = (img_center + relative_pos.x).round().clamp(0.0, (img_size - 1) as f32) as u32;
            let y = (img_center + relative_pos.y).round().clamp(0.0, (img_size - 1) as f32) as u32;
            (x, y)
        };

        // Draw the main circle (using Bresenham's algorithm for a circle)
        let r = CIRCLE_RADIUS.round() as i32;
        let cx = img_center.round() as i32;
        let cy = img_center.round() as i32;
        for t in 0..360 {
            let theta = (t as f32).to_radians();
            let x = (cx as f32 + r as f32 * theta.cos()).round() as i32;
            let y = (cy as f32 + r as f32 * theta.sin()).round() as i32;
            if x >= 0 && x < img_size as i32 && y >= 0 && y < img_size as i32 {
                image.set_pixel(x as u32, y as u32, GRAY);
            }
        }

        // Draw the sigil lines
        if self.points.len() > 1 {
            for i in 0..self.points.len() - 1 {
                let (x0, y0) = transform_point(self.points[i].relative_pos);
                let (x1, y1) = transform_point(self.points[i + 1].relative_pos);
                draw_line_on_image(&mut image, x0, y0, x1, y1, SKYBLUE);
            }
        }

        // Draw start (green) and end (red) points
        if !self.points.is_empty() {
            let (start_x, start_y) = transform_point(self.points[0].relative_pos);
            draw_circle_on_image(&mut image, start_x, start_y, 10, GREEN);
            if self.points.len() > 1 {
                let (end_x, end_y) = transform_point(self.points[self.points.len() - 1].relative_pos);
                draw_circle_on_image(&mut image, end_x, end_y, 10, RED);
            }
        }
        // Draw intermediate points (orange) and numbers
        for (i, point) in self.points.iter().enumerate() {
            if i != 0 && i != self.points.len() - 1 {
                let (x, y) = transform_point(point.relative_pos);
                draw_circle_on_image(&mut image, x, y, 10, ORANGE);
            }
            // Draw the number as a single pixel (for now, as text rendering is nontrivial)
            let (x, y) = transform_point(point.relative_pos);
            image.set_pixel(x, y, BLACK);
        }
        // Save the image as PNG
        image.export_png(&filename);
        Ok(())
    }

    /// Helper to get the (start, end) indices of the current selection, if any
    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            if start < self.cursor_pos {
                (start, self.cursor_pos)
            } else {
                (self.cursor_pos, start)
            }
        })
    }

    /// Helper to delete the current selection, if any, and return true if something was deleted
    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            self.intention.drain(start..end);
            self.cursor_pos = start;
            self.selection_start = None;
            true
        } else {
            false
        }
    }

    /// Helper to check if Ctrl is held
    fn ctrl_down() -> bool {
        is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl)
    }

    /// Handle text input, cursor movement, and selection (ASCII only)
    fn handle_text_input(&mut self) {
        // Handle character input (ASCII alphanumeric and space only)
        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii_alphanumeric() || ch == ' ' {
                self.delete_selection();
                if self.intention.len() < 100 {
                    self.intention.insert(self.cursor_pos, ch);
                    self.cursor_pos += 1;
                }
            }
        }

        // Handle backspace
        if is_key_pressed(KeyCode::Backspace) {
            if !self.delete_selection() && self.cursor_pos > 0 {
                self.cursor_pos -= 1;
                self.intention.remove(self.cursor_pos);
            }
        }

        // Handle delete
        if is_key_pressed(KeyCode::Delete) {
            if !self.delete_selection() && self.cursor_pos < self.intention.len() {
                self.intention.remove(self.cursor_pos);
            }
        }

        // Handle left arrow (with/without selection)
        if is_key_pressed(KeyCode::Left) {
            if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    if self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor_pos + 1);
                    }
                }
            } else {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                self.selection_start = None;
            }
        }

        // Handle right arrow (with/without selection)
        if is_key_pressed(KeyCode::Right) {
            if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                if self.cursor_pos < self.intention.len() {
                    if self.selection_start.is_none() {
                        self.selection_start = Some(self.cursor_pos);
                    }
                    self.cursor_pos += 1;
                }
            } else {
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
        if is_key_pressed(KeyCode::A) && Self::ctrl_down() {
            self.selection_start = Some(0);
            self.cursor_pos = self.intention.len();
        }

        // Handle Ctrl+C (Copy) - prints to console for now
        if is_key_pressed(KeyCode::C) && Self::ctrl_down() {
            if let Some((start, end)) = self.selection_range() {
                let selected_text = &self.intention[start..end];
                println!("Copied: {}", selected_text);
            }
        }

        // Handle Ctrl+V (Paste) - inserts placeholder text for now
        if is_key_pressed(KeyCode::V) && Self::ctrl_down() {
            let paste_text = "pasted_text"; // Placeholder for clipboard
            if self.intention.len() + paste_text.len() <= 100 {
                self.delete_selection();
                for ch in paste_text.chars() {
                    if ch.is_ascii_alphanumeric() || ch == ' ' {
                        self.intention.insert(self.cursor_pos, ch);
                        self.cursor_pos += 1;
                    }
                }
            }
        }

        // Handle Ctrl+X (Cut) - prints to console for now
        if is_key_pressed(KeyCode::X) && Self::ctrl_down() {
            if let Some((start, end)) = self.selection_range() {
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
            "SIGIL GENERATOR",
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

// Helper functions for drawing lines and circles on Image
fn draw_line_on_image(image: &mut macroquad::texture::Image, x0: u32, y0: u32, x1: u32, y1: u32, color: Color) {
    let (mut x0, mut y0, x1, y1) = (x0 as i32, y0 as i32, x1 as i32, y1 as i32);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let w = image.width() as u32;
    let h = image.height() as u32;
    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as u32) < w && (y0 as u32) < h {
            image.set_pixel(x0 as u32, y0 as u32, color);
        }
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}
fn draw_circle_on_image(image: &mut macroquad::texture::Image, cx: u32, cy: u32, radius: u32, color: Color) {
    let (cx, cy, r) = (cx as i32, cy as i32, radius as i32);
    let mut x = r;
    let mut y = 0;
    let mut err = 0;
    let w = image.width() as u32;
    let h = image.height() as u32;
    while x >= y {
        for &(dx, dy) in &[(x, y), (y, x), (-y, x), (-x, y), (-x, -y), (-y, -x), (y, -x), (x, -y)] {
            let px = cx + dx;
            let py = cy + dy;
            if px >= 0 && py >= 0 && (px as u32) < w && (py as u32) < h {
                image.set_pixel(px as u32, py as u32, color);
            }
        }
        y += 1;
        if err <= 0 {
            err += 2 * y + 1;
        } else {
            x -= 1;
            err -= 2 * x + 1;
        }
    }
}

/// Main entry point for the Macroquad application
#[macroquad::main("Sigil-Gen")]
async fn main() {
    let mut app = SigilApp::new();
    loop {
        app.update();
        app.draw();
        next_frame().await;
    }
}
