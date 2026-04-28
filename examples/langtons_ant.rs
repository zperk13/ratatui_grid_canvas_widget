use ratatui::crossterm;
use ratatui::crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui_grid_canvas_widget::ToColor;
use ratatui_grid_canvas_widget::grid::alloc::AllocColoredGrid;
use ratatui_grid_canvas_widget::widget::color::*;
use std::time::{Duration, Instant};

fn main() {
    let (width, height) = crossterm::terminal::size().unwrap();
    // -1 is for the text being rendered at the top of the screen
    ratatui::run(|terminal| {
        LangtonsAnt::new(width as usize, (height as usize) - 1).run(terminal)
    })
    .unwrap();
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
enum Mode {
    DoubleFullBlock,
    FullBlock,
    HalfBlock,
    HorizontalHalfBlock,
}

impl Mode {
    fn cycle(&mut self) {
        use Mode::*;
        *self = match self {
            DoubleFullBlock => FullBlock,
            FullBlock => HalfBlock,
            HalfBlock => HorizontalHalfBlock,
            HorizontalHalfBlock => DoubleFullBlock,
        }
    }
}

#[derive(Clone, Copy)]
struct Cell(u64);
impl ToColor for Cell {
    fn to_color(&self) -> Color {
        Color::Indexed((self.0 % 16) as u8)
    }
}

enum Facing {
    Up,
    Right,
    Down,
    Left,
}

impl Facing {
    fn cw(&mut self) {
        use Facing::*;
        *self = match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }
    fn ccw(&mut self) {
        use Facing::*;
        *self = match self {
            Up => Left,
            Right => Up,
            Down => Right,
            Left => Down,
        }
    }
    fn offset(&self, x: &mut usize, y: &mut usize, width: usize, height: usize) {
        use Facing::*;
        match self {
            Up => *y = wrap_sub(*y, 1, 0, height - 1),
            Right => *x = wrap_add(*x, 1, 0, width - 1),
            Down => *y = wrap_add(*y, 1, 0, height - 1),
            Left => *x = wrap_sub(*x, 1, 0, width - 1),
        }
    }
}

struct LangtonsAnt {
    mode: Mode,
    width: usize,
    height: usize,
    grid: AllocColoredGrid<Cell>,
    ant_x: usize,
    ant_y: usize,
    facing: Facing,
    last_frame: Instant,
    pan_x: usize,
    pan_y: usize,
}

impl LangtonsAnt {
    fn new(width: usize, height: usize) -> Self {
        Self {
            mode: Mode::HalfBlock,
            width,
            height,
            grid: AllocColoredGrid::new_filled(width, height, Cell(0)),
            ant_x: width / 2,
            ant_y: height / 2,
            facing: Facing::Up,
            last_frame: Instant::now(),
            pan_x: 0,
            pan_y: 0,
        }
    }
    fn tick(&mut self) {
        let cell = &mut self.grid.get_mut(self.ant_x, self.ant_y).unwrap().0;
        if cell.is_multiple_of(2) {
            self.facing.cw();
        } else {
            self.facing.ccw();
        }
        *cell += 1;
        self.facing
            .offset(&mut self.ant_x, &mut self.ant_y, self.width, self.height);
    }
    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        loop {
            if crossterm::event::poll(Duration::from_secs(0))? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(key_event)
                        if key_event.kind == KeyEventKind::Press =>
                    {
                        match key_event.code {
                            KeyCode::Char('m') => {
                                self.mode.cycle();
                            }
                            KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Char('w') => {
                                self.pan_y = self.pan_y.saturating_sub(1);
                            }
                            KeyCode::Char('a') => {
                                self.pan_x = self.pan_x.saturating_sub(1);
                            }
                            KeyCode::Char('s') => {
                                self.pan_y += 1;
                            }
                            KeyCode::Char('d') => {
                                self.pan_x += 1;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
            self.last_frame = Instant::now();
            self.tick();
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &LangtonsAnt {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [text_area, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        let [quit_area, wasd_area, fps_area, mode_area] =
            Layout::horizontal([Constraint::Fill(1); 4])
                .flex(Flex::SpaceBetween)
                .areas(text_area);
        ratatui::macros::span!("q to quit")
            .underlined()
            .bold()
            .reversed()
            .render(quit_area, buf);
        ratatui::macros::span!("wasd to move")
            .underlined()
            .bold()
            .reversed()
            .render(wasd_area, buf);
        ratatui::macros::span!("{:.0} FPS", 1. / self.last_frame.elapsed().as_secs_f64())
            .underlined()
            .bold()
            .reversed()
            .render(fps_area, buf);
        ratatui::macros::span!("{:?} (Cycle with m)", self.mode)
            .underlined()
            .bold()
            .reversed()
            .render(mode_area, buf);
        match self.mode {
            Mode::DoubleFullBlock => DoubleFullBlockColorGridWidget::new(&self.grid)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(rest, buf),
            Mode::FullBlock => FullBlockColorGridWidget::new(&self.grid)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(rest, buf),
            Mode::HalfBlock => HalfBlockColorGridWidget::new(&self.grid)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(rest, buf),
            Mode::HorizontalHalfBlock => HorizontalHalfBlockColorGridWidget::new(&self.grid)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(rest, buf),
        }
    }
}

pub fn wrap_sub(lhs: usize, rhs: usize, min: usize, max: usize) -> usize {
    debug_assert!(min <= max, "min can't be greater than max");
    let range = max - min + 1;
    let effective_rhs = rhs % range;
    let zero_indexed_lhs = lhs - min;
    let wrapped_zero_indexed = (zero_indexed_lhs + range - effective_rhs) % range;
    min + wrapped_zero_indexed
}

pub fn wrap_add(lhs: usize, rhs: usize, min: usize, max: usize) -> usize {
    debug_assert!(min <= max, "min can't be greater than max");
    let range = max - min + 1;
    let effective_rhs = rhs % range;
    let zero_indexed_lhs = lhs - min;
    let wrapped_zero_indexed = (zero_indexed_lhs + effective_rhs) % range;
    min + wrapped_zero_indexed
}
