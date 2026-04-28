use ratatui::crossterm;
use ratatui::crossterm::event::{KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui_grid_canvas_widget::ToColor;
use ratatui_grid_canvas_widget::grid::alloc::AllocColoredGrid;
use ratatui_grid_canvas_widget::widget::color::*;
use std::time::{Duration, Instant};

fn main() {
    let (width, height) = crossterm::terminal::size().unwrap();
    ratatui::run(|terminal| LangtonsAnt::new(width as usize, (height - 1) as usize).run(terminal))
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
    x: usize,
    y: usize,
    facing: Facing,
    last_frame: Instant,
}

impl LangtonsAnt {
    fn new(width: usize, height: usize) -> Self {
        Self {
            mode: Mode::FullBlock,
            width,
            height,
            grid: AllocColoredGrid::new_filled(width, height, Cell(0)),
            x: width / 2,
            y: height / 2,
            facing: Facing::Up,
            last_frame: Instant::now(),
        }
    }
    fn tick(&mut self) {
        let cell = &mut self.grid.get_mut(self.x, self.y).unwrap().0;
        if cell.is_multiple_of(2) {
            self.facing.cw();
        } else {
            self.facing.ccw();
        }
        *cell += 1;
        self.facing
            .offset(&mut self.x, &mut self.y, self.width, self.height);
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
        let [top, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        ratatui::macros::span!(
            "{:?} (Cycle with m) q to quit {:.0} FPS",
            self.mode,
            1. / self.last_frame.elapsed().as_secs_f64()
        )
        .render(top, buf);
        match self.mode {
            Mode::DoubleFullBlock => {
                DoubleFullBlockColorGridWidget::new(&self.grid).render(rest, buf)
            }
            Mode::FullBlock => FullBlockColorGridWidget::new(&self.grid).render(rest, buf),
            Mode::HalfBlock => HalfBlockColorGridWidget::new(&self.grid).render(rest, buf),
            Mode::HorizontalHalfBlock => {
                HorizontalHalfBlockColorGridWidget::new(&self.grid).render(rest, buf)
            }
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
