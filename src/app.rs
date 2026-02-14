//! State machine for Lissajous Machine.
//!
//! States:
//!   Menu      — choose frequency ratio, phase, amplitude
//!   Drawing   — live curve rendering
//!   Gallery   — preset patterns
//!   Table     — frequency ratio grid showing pattern shapes

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::math;

const KEY_UP: char = '\u{F700}';
const KEY_DOWN: char = '\u{F701}';
const KEY_LEFT: char = '\u{F702}';
const KEY_RIGHT: char = '\u{F703}';
const KEY_ENTER: char = '\u{000D}';
const KEY_BACKSPACE: char = '\u{0008}';
const KEY_MENU: char = '\u{2234}';

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Menu,
    Drawing,
    Gallery,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuField {
    FreqA,
    FreqB,
    Phase,
    Draw,
    Gallery,
    Table,
}

/// A preset Lissajous pattern.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    pub name: &'static str,
    pub freq_a: i32,
    pub freq_b: i32,
    pub phase: i32,  // in 1/1024 units (256 = π/2)
}

pub const PRESETS: &[Preset] = &[
    Preset { name: "Circle",         freq_a: 1, freq_b: 1, phase: 256 },
    Preset { name: "Figure-8",       freq_a: 1, freq_b: 2, phase: 0 },
    Preset { name: "Bowtie",         freq_a: 2, freq_b: 3, phase: 0 },
    Preset { name: "Trefoil",        freq_a: 3, freq_b: 2, phase: 256 },
    Preset { name: "Clover",         freq_a: 3, freq_b: 4, phase: 0 },
    Preset { name: "Star Knot",      freq_a: 5, freq_b: 4, phase: 256 },
    Preset { name: "Diamond",        freq_a: 1, freq_b: 2, phase: 256 },
    Preset { name: "Infinity",       freq_a: 2, freq_b: 1, phase: 256 },
    Preset { name: "Ribbon",         freq_a: 3, freq_b: 5, phase: 128 },
    Preset { name: "Atomic",         freq_a: 5, freq_b: 6, phase: 0 },
];

pub struct LissajousApp {
    pub state: AppState,
    pub needs_redraw: bool,

    // Menu
    pub menu_field: MenuField,
    pub freq_a: i32,
    pub freq_b: i32,
    pub phase: i32,     // in 1/1024 units (256 = π/2)

    // Drawing - precomputed points
    pub points: Vec<(i16, i16)>,

    // Gallery
    pub gallery_cursor: usize,

    // Table
    pub table_row: usize,
    pub table_col: usize,
}

impl LissajousApp {
    pub fn new() -> Self {
        Self {
            state: AppState::Menu,
            needs_redraw: true,
            menu_field: MenuField::FreqA,
            freq_a: 1,
            freq_b: 2,
            phase: 0,
            points: Vec::new(),
            gallery_cursor: 0,
            table_row: 0,
            table_col: 0,
        }
    }

    fn compute_curve(&mut self) {
        self.compute_curve_params(self.freq_a, self.freq_b, self.phase);
    }

    fn compute_curve_params(&mut self, fa: i32, fb: i32, ph: i32) {
        self.points.clear();
        // Period: LCM(fa, fb) full cycles at 1024 steps per cycle
        let period = math::lcm(fa, fb);
        let total_steps = (period * 1024).min(16384);

        let cx: i16 = 168; // center x
        let cy: i16 = 260; // center y

        let amp = 140i32; // pixel amplitude

        for step in 0..total_steps {
            let t = step;
            let x_fp = math::lissajous_x(t, fa, ph, amp);
            let y_fp = math::lissajous_y(t, fb, amp);

            // Convert from fixed-point to screen coords
            let sx = cx + (x_fp >> 16) as i16;
            let sy = cy + (y_fp >> 16) as i16;

            self.points.push((sx, sy));
        }
    }

    pub fn handle_key(&mut self, key: char) -> bool {
        self.needs_redraw = true;
        match self.state {
            AppState::Menu => self.handle_menu(key),
            AppState::Drawing => self.handle_drawing(key),
            AppState::Gallery => self.handle_gallery(key),
            AppState::Table => self.handle_table(key),
        }
    }

    fn handle_menu(&mut self, key: char) -> bool {
        match key {
            KEY_MENU => return false,
            KEY_UP => {
                self.menu_field = match self.menu_field {
                    MenuField::FreqA => MenuField::Table,
                    MenuField::FreqB => MenuField::FreqA,
                    MenuField::Phase => MenuField::FreqB,
                    MenuField::Draw => MenuField::Phase,
                    MenuField::Gallery => MenuField::Draw,
                    MenuField::Table => MenuField::Gallery,
                };
            }
            KEY_DOWN => {
                self.menu_field = match self.menu_field {
                    MenuField::FreqA => MenuField::FreqB,
                    MenuField::FreqB => MenuField::Phase,
                    MenuField::Phase => MenuField::Draw,
                    MenuField::Draw => MenuField::Gallery,
                    MenuField::Gallery => MenuField::Table,
                    MenuField::Table => MenuField::FreqA,
                };
            }
            KEY_LEFT | KEY_RIGHT => {
                let delta = if key == KEY_RIGHT { 1 } else { -1 };
                match self.menu_field {
                    MenuField::FreqA => {
                        self.freq_a = (self.freq_a + delta).max(1).min(9);
                    }
                    MenuField::FreqB => {
                        self.freq_b = (self.freq_b + delta).max(1).min(9);
                    }
                    MenuField::Phase => {
                        // Step by 64 (≈22.5°)
                        self.phase = ((self.phase + delta * 64) % 1024 + 1024) % 1024;
                    }
                    _ => {}
                }
            }
            KEY_ENTER => match self.menu_field {
                MenuField::Draw => {
                    self.compute_curve();
                    self.state = AppState::Drawing;
                }
                MenuField::Gallery => {
                    self.state = AppState::Gallery;
                }
                MenuField::Table => {
                    self.state = AppState::Table;
                }
                _ => {}
            },
            _ => {}
        }
        true
    }

    fn handle_drawing(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_LEFT | KEY_ENTER => {
                self.state = AppState::Menu;
            }
            _ => {}
        }
        true
    }

    fn handle_gallery(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_LEFT => {
                self.state = AppState::Menu;
            }
            KEY_UP => {
                if self.gallery_cursor > 0 {
                    self.gallery_cursor -= 1;
                }
            }
            KEY_DOWN => {
                if self.gallery_cursor < PRESETS.len() - 1 {
                    self.gallery_cursor += 1;
                }
            }
            KEY_ENTER => {
                let preset = PRESETS[self.gallery_cursor];
                self.freq_a = preset.freq_a;
                self.freq_b = preset.freq_b;
                self.phase = preset.phase;
                self.compute_curve();
                self.state = AppState::Drawing;
            }
            _ => {}
        }
        true
    }

    fn handle_table(&mut self, key: char) -> bool {
        match key {
            KEY_MENU | KEY_LEFT => {
                self.state = AppState::Menu;
            }
            KEY_UP => {
                if self.table_row > 0 {
                    self.table_row -= 1;
                }
            }
            KEY_DOWN => {
                if self.table_row < 5 {
                    self.table_row += 1;
                }
            }
            KEY_RIGHT => {
                if self.table_col < 5 {
                    self.table_col += 1;
                }
            }
            KEY_ENTER => {
                // Draw the selected ratio
                self.freq_a = (self.table_col as i32) + 1;
                self.freq_b = (self.table_row as i32) + 1;
                self.phase = 256; // π/2 for nice shapes
                self.compute_curve();
                self.state = AppState::Drawing;
            }
            _ => {}
        }
        true
    }

    /// Phase in degrees (approx) for display
    pub fn phase_degrees(&self) -> i32 {
        self.phase * 360 / 1024
    }
}
