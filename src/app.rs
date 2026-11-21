//! Lissajous shell: keys -> core operations, drawing, nothing else. The curve
//! maths, presets and the incremental tracer are in `lissajous-core`.

use gam::menu::Point;
use gam::GlyphStyle;
use lissajous_core::{thumbnail, Curve, Params, Tracer, FREQ_MAX, PRESETS, SEGMENTS_PER_TICK};
use precursor_appkit::keys::Key;
use precursor_appkit::layout::ListState;
use precursor_appkit::screen::{Screen, MARGIN};

#[derive(PartialEq, Eq)]
pub enum Action {
    Stay,
    Leave,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Menu,
    Drawing,
    Gallery,
    Table,
}

const MENU_ROWS: usize = 6;
const ROW_DRAW: usize = 3;
const ROW_GALLERY: usize = 4;
const ROW_TABLE: usize = 5;

pub struct App {
    screen: Screen,
    view: View,
    params: Params,
    menu: ListState,
    gallery: ListState,
    visible: usize,
    /// Ratio table cursor: column = x frequency - 1, row = y frequency - 1.
    cell: (i32, i32),
    tracer: Tracer,
}

impl App {
    pub fn new(screen: Screen) -> Self {
        let params = PRESETS[1].params;
        let mut app = Self {
            screen,
            view: View::Menu,
            params,
            menu: ListState::default(),
            gallery: ListState::default(),
            visible: 1,
            cell: (params.a - 1, params.b - 1),
            tracer: Tracer::new(Curve::new(params, (0, 0), (1, 1))),
        };
        app.restart();
        app
    }

    pub fn screen(&self) -> &Screen { &self.screen }

    /// True while the pen is still moving (the pump runs only then).
    pub fn animating(&self) -> bool { self.view == View::Drawing && !self.tracer.done() }

    /// Content area between header and footer, matching `Screen::header`/`footer`.
    fn content(&self) -> (isize, isize) {
        let top = 2 + self.screen.line_height(GlyphStyle::Bold) + 2 + 4;
        let bottom = self.screen.size().y - self.screen.line_height(GlyphStyle::Small) - 2 - 4;
        (top, bottom)
    }

    fn restart(&mut self) {
        let (top, bottom) = self.content();
        let w = self.screen.size().x as i32;
        let amp = ((w - 2 * MARGIN as i32) / 2).min(((bottom - top) as i32) / 2 - 2).max(8);
        let center = (w / 2, ((top + bottom) / 2) as i32);
        self.tracer = Tracer::new(Curve::new(self.params, center, (amp, amp)));
    }

    fn start(&mut self, p: Params) {
        self.params = p;
        self.cell = (p.a - 1, p.b - 1);
        self.restart();
        self.view = View::Drawing;
    }

    pub fn on_key(&mut self, key: Key) -> Action {
        match self.view {
            View::Menu => return self.menu_key(key),
            View::Drawing => match key {
                Key::Left | Key::Right => {
                    self.params.step_phase(if key == Key::Left { -1 } else { 1 });
                    self.restart();
                }
                Key::Enter => self.restart(),
                k if k.is_back() => self.view = View::Menu,
                _ => {}
            },
            View::Gallery => match key {
                Key::Up => self.gallery.step(-1, PRESETS.len(), self.visible),
                Key::Down => self.gallery.step(1, PRESETS.len(), self.visible),
                Key::Enter => self.start(PRESETS[self.gallery.selected].params),
                k if k.is_back() => self.view = View::Menu,
                _ => {}
            },
            View::Table => {
                let (c, r) = &mut self.cell;
                match key {
                    Key::Left => *c = (*c - 1).max(0),
                    Key::Right => *c = (*c + 1).min(FREQ_MAX - 1),
                    Key::Up => *r = (*r - 1).max(0),
                    Key::Down => *r = (*r + 1).min(FREQ_MAX - 1),
                    // Keep the user's phase; the ratio is what the table chooses.
                    Key::Enter => {
                        let p = Params::new(*c + 1, *r + 1, self.params.phase);
                        self.start(p);
                    }
                    k if k.is_back() => self.view = View::Menu,
                    _ => {}
                }
            }
        }
        Action::Stay
    }

    fn menu_key(&mut self, key: Key) -> Action {
        match key {
            Key::Up => self.menu.step_wrapping(-1, MENU_ROWS, self.visible),
            Key::Down => self.menu.step_wrapping(1, MENU_ROWS, self.visible),
            Key::Left | Key::Right => {
                let d = if key == Key::Left { -1 } else { 1 };
                match self.menu.selected {
                    0 => self.params.step_a(d),
                    1 => self.params.step_b(d),
                    2 => self.params.step_phase(d),
                    _ => {}
                }
            }
            Key::Enter => match self.menu.selected {
                ROW_DRAW => self.start(self.params),
                ROW_GALLERY => self.view = View::Gallery,
                ROW_TABLE => {
                    self.cell = (self.params.a - 1, self.params.b - 1);
                    self.view = View::Table;
                }
                _ => {}
            },
            k if k.is_back() => return Action::Leave,
            _ => {}
        }
        Action::Stay
    }

    pub fn on_foreground(&mut self) {
        self.screen.refresh_size();
        if self.view == View::Drawing {
            self.restart();
        }
    }

    /// Reveal the next few segments. Only the new ones are drawn: no clear, no
    /// redraw of what is already on screen.
    pub fn on_tick(&mut self) {
        if self.view != View::Drawing {
            return;
        }
        let range = self.tracer.advance(SEGMENTS_PER_TICK);
        let clip = self.clip();
        self.screen.lines(self.tracer.curve.segments(range), clip, 1);
        self.screen.flush();
    }

    fn clip(&self) -> ((i32, i32), (i32, i32)) {
        let (top, bottom) = self.content();
        ((0, top as i32), (self.screen.size().x as i32 - 1, bottom as i32))
    }

    pub fn draw(&mut self) {
        self.screen.clear();
        let p = self.params;
        match self.view {
            View::Menu => {
                let top = self.screen.header("Lissajous");
                let bottom = self.screen.footer("↑↓ field  ←→ change  ⏎ go  ⌫ quit");
                let items = [
                    format!("A (x) frequency: {}", p.a),
                    format!("B (y) frequency: {}", p.b),
                    format!("Phase: {}°", p.phase_degrees()),
                    "Draw".to_string(),
                    "Presets".to_string(),
                    "Ratio table".to_string(),
                ];
                self.visible = self.screen.list(top, bottom, &items, &self.menu);
            }
            View::Drawing => {
                self.screen.header(&format!("{}:{}  phase {}°", p.a, p.b, p.phase_degrees()));
                self.screen.footer("←→ phase  ⏎ redraw  ⌫ back");
                let drawn = self.tracer.drawn();
                self.screen.lines(self.tracer.curve.segments(0..drawn), self.clip(), 1);
            }
            View::Gallery => {
                let top = self.screen.header("Presets");
                let bottom = self.screen.footer("↑↓ move  ⏎ draw  ⌫ back");
                let items: Vec<String> = PRESETS
                    .iter()
                    .map(|pr| format!("{}  {}:{}", pr.name, pr.params.a, pr.params.b))
                    .collect();
                self.visible = self.screen.list(top, bottom, &items, &self.gallery);
            }
            View::Table => self.draw_table(),
        }
        self.screen.flush();
    }

    fn draw_table(&self) {
        let (c, r) = self.cell;
        self.screen.header(&format!("Ratios  x{} : y{}", c + 1, r + 1));
        let (top, bottom) = self.content();
        self.screen.footer("arrows move  ⏎ draw  ⌫ back");
        let w = self.screen.size().x - 2 * MARGIN;
        let side = (w / FREQ_MAX as isize).min((bottom - top) / FREQ_MAX as isize);
        let x0 = (self.screen.size().x - side * FREQ_MAX as isize) / 2;
        let rad = (side / 2 - 3) as i32;
        let mut segs = Vec::with_capacity((FREQ_MAX * FREQ_MAX) as usize * 64);
        for row in 0..FREQ_MAX {
            for col in 0..FREQ_MAX {
                let center = (
                    (x0 + col as isize * side + side / 2) as i32,
                    (top + row as isize * side + side / 2) as i32,
                );
                segs.extend(thumbnail(Params::new(col + 1, row + 1, self.params.phase), center, rad).segments(0..64));
            }
        }
        let clip = ((0, top as i32), (self.screen.size().x as i32 - 1, bottom as i32));
        self.screen.lines(segs, clip, 1);
        let tl = Point::new(x0 + c as isize * side, top + r as isize * side);
        self.screen.frame(tl, Point::new(tl.x + side - 1, tl.y + side - 1), 2);
    }
}
