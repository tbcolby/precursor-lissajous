//! UI rendering for Lissajous Machine.

extern crate alloc;
use alloc::format;
use alloc::string::String;

use gam::{Gam, Point, Line, TextView, TextBounds, GlyphStyle};

use crate::app::{LissajousApp, AppState, MenuField, PRESETS};

const SCREEN_W: i16 = 336;
const HEADER_H: i16 = 30;
const FOOTER_Y: i16 = 490;

fn draw_header(gam: &Gam, title: &str) {
    gam.draw_rectangle(Point::new(0, 0), Point::new(SCREEN_W, HEADER_H)).ok();

    let mut tv = TextView::new(
        TextBounds::BoundingBox(Point::new(8, 2), Point::new(SCREEN_W - 8, HEADER_H)),
    );
    tv.style = GlyphStyle::Bold;
    tv.invert = true;
    tv.text = String::from(title);
    gam.post_textview(&mut tv).ok();
}

fn draw_footer(gam: &Gam, text: &str) {
    gam.draw_line(Line::new(Point::new(0, FOOTER_Y), Point::new(SCREEN_W, FOOTER_Y))).ok();

    let mut tv = TextView::new(
        TextBounds::BoundingBox(Point::new(8, FOOTER_Y + 4), Point::new(SCREEN_W - 8, FOOTER_Y + 40)),
    );
    tv.style = GlyphStyle::Small;
    tv.text = String::from(text);
    gam.post_textview(&mut tv).ok();
}

pub fn draw(gam: &Gam, app: &LissajousApp) {
    gam.clear().ok();

    match app.state {
        AppState::Menu => draw_menu(gam, app),
        AppState::Drawing => draw_curve(gam, app),
        AppState::Gallery => draw_gallery(gam, app),
        AppState::Table => draw_table(gam, app),
    }

    gam.redraw().ok();
}

fn draw_menu(gam: &Gam, app: &LissajousApp) {
    draw_header(gam, "Lissajous Machine");

    let fields = [
        (MenuField::FreqA, format!("Freq A: < {} >", app.freq_a)),
        (MenuField::FreqB, format!("Freq B: < {} >", app.freq_b)),
        (MenuField::Phase, format!("Phase:  < {}° >", app.phase_degrees())),
        (MenuField::Draw, String::from("[ Draw Curve ]")),
        (MenuField::Gallery, String::from("[ Gallery ]")),
        (MenuField::Table, String::from("[ Ratio Table ]")),
    ];

    let start_y: i16 = 60;
    let line_h: i16 = 36;

    for (i, (field, label)) in fields.iter().enumerate() {
        let y = start_y + (i as i16) * line_h;
        let selected = app.menu_field == *field;

        if selected {
            gam.draw_rectangle(
                Point::new(4, y - 2),
                Point::new(SCREEN_W - 4, y + line_h - 6),
            ).ok();
        }

        let mut tv = TextView::new(
            TextBounds::BoundingBox(
                Point::new(12, y),
                Point::new(SCREEN_W - 12, y + line_h),
            ),
        );
        tv.style = GlyphStyle::Regular;
        if selected {
            tv.invert = true;
        }
        tv.text = label.clone();
        gam.post_textview(&mut tv).ok();
    }

    // Ratio display
    let info_y = start_y + 6 * line_h + 10;
    let mut tv = TextView::new(
        TextBounds::BoundingBox(
            Point::new(12, info_y),
            Point::new(SCREEN_W - 12, info_y + 80),
        ),
    );
    tv.style = GlyphStyle::Small;
    tv.text = format!(
        "Ratio A:B = {}:{}\nArrows to adjust. Enter to draw.\nMenu to quit.",
        app.freq_a, app.freq_b
    );
    gam.post_textview(&mut tv).ok();

    draw_footer(gam, "Lissajous Machine v0.1");
}

fn draw_curve(gam: &Gam, app: &LissajousApp) {
    let title = format!("{}:{} phase={}°",
        app.freq_a, app.freq_b, app.phase_degrees());
    draw_header(gam, &title);

    if app.points.len() >= 2 {
        for i in 1..app.points.len() {
            let (x0, y0) = app.points[i - 1];
            let (x1, y1) = app.points[i];

            if y0 >= HEADER_H && y1 >= HEADER_H && y0 < FOOTER_Y && y1 < FOOTER_Y
               && x0 >= 0 && x1 >= 0 && x0 < SCREEN_W && x1 < SCREEN_W
            {
                gam.draw_line(Line::new(
                    Point::new(x0, y0),
                    Point::new(x1, y1),
                )).ok();
            }
        }
    }

    draw_footer(gam, "Enter/Left/Menu: back to menu");
}

fn draw_gallery(gam: &Gam, app: &LissajousApp) {
    draw_header(gam, "Gallery — Presets");

    let start_y: i16 = 40;
    let line_h: i16 = 32;
    let visible = 12;

    let scroll = if app.gallery_cursor >= visible {
        app.gallery_cursor - visible + 1
    } else {
        0
    };

    for i in 0..PRESETS.len().min(visible) {
        let idx = scroll + i;
        if idx >= PRESETS.len() { break; }

        let preset = &PRESETS[idx];
        let y = start_y + (i as i16) * line_h;
        let selected = idx == app.gallery_cursor;

        if selected {
            gam.draw_rectangle(
                Point::new(4, y - 2),
                Point::new(SCREEN_W - 4, y + line_h - 6),
            ).ok();
        }

        let phase_deg = preset.phase * 360 / 1024;
        let label = format!("{} ({}:{} {}°)",
            preset.name, preset.freq_a, preset.freq_b, phase_deg);

        let mut tv = TextView::new(
            TextBounds::BoundingBox(
                Point::new(12, y),
                Point::new(SCREEN_W - 12, y + line_h),
            ),
        );
        tv.style = GlyphStyle::Small;
        if selected {
            tv.invert = true;
        }
        tv.text = label;
        gam.post_textview(&mut tv).ok();
    }

    draw_footer(gam, "Up/Down: browse  Enter: draw  Left: back");
}

fn draw_table(gam: &Gam, app: &LissajousApp) {
    draw_header(gam, "Ratio Table — A:B");

    // 6x6 grid showing ratios 1:1 through 6:6
    let grid_x: i16 = 20;
    let grid_y: i16 = 50;
    let cell_w: i16 = 50;
    let cell_h: i16 = 40;

    // Column headers (A values)
    for col in 0..6 {
        let x = grid_x + (col as i16) * cell_w;
        let mut tv = TextView::new(
            TextBounds::BoundingBox(
                Point::new(x + 18, grid_y - 20),
                Point::new(x + cell_w, grid_y),
            ),
        );
        tv.style = GlyphStyle::Small;
        tv.text = format!("{}", col + 1);
        gam.post_textview(&mut tv).ok();
    }

    // Row headers (B values) and cells
    for row in 0..6 {
        let y = grid_y + (row as i16) * cell_h;

        // Row header
        let mut tv = TextView::new(
            TextBounds::BoundingBox(
                Point::new(2, y + 10),
                Point::new(grid_x, y + cell_h),
            ),
        );
        tv.style = GlyphStyle::Small;
        tv.text = format!("{}", row + 1);
        gam.post_textview(&mut tv).ok();

        for col in 0..6 {
            let x = grid_x + (col as i16) * cell_w;
            let selected = row == app.table_row && col == app.table_col;

            // Cell border
            gam.draw_line(Line::new(Point::new(x, y), Point::new(x + cell_w, y))).ok();
            gam.draw_line(Line::new(Point::new(x, y), Point::new(x, y + cell_h))).ok();

            if selected {
                gam.draw_rectangle(
                    Point::new(x + 1, y + 1),
                    Point::new(x + cell_w - 1, y + cell_h - 1),
                ).ok();
            }

            // Ratio label
            let mut tv = TextView::new(
                TextBounds::BoundingBox(
                    Point::new(x + 6, y + 8),
                    Point::new(x + cell_w - 2, y + cell_h - 2),
                ),
            );
            tv.style = GlyphStyle::Small;
            if selected {
                tv.invert = true;
            }
            tv.text = format!("{}:{}", col + 1, row + 1);
            gam.post_textview(&mut tv).ok();
        }
    }

    // Bottom border
    let bottom_y = grid_y + 6 * cell_h;
    gam.draw_line(Line::new(
        Point::new(grid_x, bottom_y),
        Point::new(grid_x + 6 * cell_w, bottom_y),
    )).ok();
    // Right border
    gam.draw_line(Line::new(
        Point::new(grid_x + 6 * cell_w, grid_y),
        Point::new(grid_x + 6 * cell_w, bottom_y),
    )).ok();

    draw_footer(gam, "Arrows: select  Enter: draw  Left: back");
}
