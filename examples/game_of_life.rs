use ratatui::crossterm;
use ratatui::crossterm::event::{KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui_grid_canvas_widget::grid::alloc::AllocBinaryGrid;
use ratatui_grid_canvas_widget::widget::binary::*;
use std::mem::swap;
use std::time::Duration;

fn main() {
    let (width, height) = crossterm::terminal::size().unwrap();
    ratatui::run(|terminal| {
        GameOfLife::random(width as usize, (height - 1) as usize).run(terminal)
    })
    .unwrap();
}

#[derive(Debug)]
enum Mode {
    DoubleFullBlock,
    FullBlock,
    HalfBlock,
    HorizontalHalfBlock,
    Quadrant,
    Sextant,
    Braille,
}

impl Mode {
    fn cycle(&mut self) {
        use Mode::*;
        *self = match self {
            DoubleFullBlock => FullBlock,
            FullBlock => HalfBlock,
            HalfBlock => HorizontalHalfBlock,
            HorizontalHalfBlock => Quadrant,
            Quadrant => Sextant,
            Sextant => Braille,
            Braille => DoubleFullBlock,
        }
    }
}

struct GameOfLife {
    mode: Mode,
    width: usize,
    height: usize,
    current: AllocBinaryGrid,
    next: AllocBinaryGrid,
}

impl GameOfLife {
    fn random(width: usize, height: usize) -> Self {
        Self {
            mode: Mode::FullBlock,
            width,
            height,
            current: AllocBinaryGrid::from_fn(width, height, |_x, _y| fastrand::bool()),
            next: AllocBinaryGrid::new_filled(width, height, false),
        }
    }
    fn neighbors(&self, x: usize, y: usize) -> usize {
        let left = wrap_sub(x, 1, 0, self.width - 1);
        let right = wrap_add(x, 1, 0, self.width - 1);
        let up = wrap_sub(y, 1, 0, self.height - 1);
        let down = wrap_add(y, 1, 0, self.height - 1);
        let mut count = 0;
        if self.current.get(left, up).unwrap() {
            count += 1;
        }
        if self.current.get(x, up).unwrap() {
            count += 1;
        }
        if self.current.get(right, up).unwrap() {
            count += 1;
        }
        if self.current.get(left, y).unwrap() {
            count += 1;
        }
        if self.current.get(right, y).unwrap() {
            count += 1;
        }
        if self.current.get(left, down).unwrap() {
            count += 1;
        }
        if self.current.get(x, down).unwrap() {
            count += 1;
        }
        if self.current.get(right, down).unwrap() {
            count += 1;
        }
        count
    }
    fn tick(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.next.set(
                    x,
                    y,
                    match (self.current.get(x, y).unwrap(), self.neighbors(x, y)) {
                        (true, ..2) => false,
                        (true, 2..=3) => true,
                        (true, 4..) => false,
                        (false, 3) => true,
                        (false, _) => false,
                    },
                );
            }
        }
        swap(&mut self.current, &mut self.next);
    }
    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        loop {
            if crossterm::event::poll(Duration::from_secs_f64(1. / 20.))? {
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
            self.tick();
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &GameOfLife {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, rest] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        ratatui::macros::span!("{:?} (Cycle with m) q to quit", self.mode).render(top, buf);
        match self.mode {
            Mode::DoubleFullBlock => {
                DoubleFullBlockBinaryGridWidget::new(&self.current).render(rest, buf)
            }
            Mode::FullBlock => FullBlockBinaryGridWidget::new(&self.current).render(rest, buf),
            Mode::HalfBlock => HalfBlockBinaryGridWidget::new(&self.current).render(rest, buf),
            Mode::HorizontalHalfBlock => {
                HorizontalHalfBlockBinaryGridWidget::new(&self.current).render(rest, buf)
            }
            Mode::Quadrant => QuadrantBinaryGridWidget::new(&self.current).render(rest, buf),
            Mode::Sextant => SextantBinaryGridWidget::new(&self.current).render(rest, buf),
            Mode::Braille => BrailleBinaryGridWidget::new(&self.current).render(rest, buf),
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
