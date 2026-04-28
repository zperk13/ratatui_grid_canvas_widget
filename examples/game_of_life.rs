use ratatui::crossterm;
use ratatui::crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui_grid_canvas_widget::grid::alloc::AllocBinaryGrid;
use ratatui_grid_canvas_widget::widget::binary::*;
use std::mem::swap;
use std::time::Duration;

fn main() {
    let (width, height) = crossterm::terminal::size().unwrap();
    ratatui::run(|terminal| {
        // -1 is for the text being rendered at the top of the screen
        GameOfLife::random((width as usize) * 2, (height as usize) * 4 - 1).run(terminal)
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
    pan_x: usize,
    pan_y: usize,
}

impl GameOfLife {
    fn random(width: usize, height: usize) -> Self {
        Self {
            mode: Mode::DoubleFullBlock,
            width,
            height,
            current: AllocBinaryGrid::from_fn(width, height, |_x, _y| fastrand::bool()),
            next: AllocBinaryGrid::new_filled(width, height, false),
            pan_x: 0,
            pan_y: 0,
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
        let [text_area, game_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        let [quit_area, wasd_area, mode_area] = Layout::horizontal([Constraint::Fill(1); 3])
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
        ratatui::macros::span!("{:?} (Cycle with m)", self.mode)
            .underlined()
            .bold()
            .reversed()
            .render(mode_area, buf);
        match self.mode {
            Mode::DoubleFullBlock => DoubleFullBlockBinaryGridWidget::new(&self.current)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(game_area, buf),
            Mode::FullBlock => FullBlockBinaryGridWidget::new(&self.current)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(game_area, buf),
            Mode::HalfBlock => HalfBlockBinaryGridWidget::new(&self.current)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(game_area, buf),
            Mode::HorizontalHalfBlock => HorizontalHalfBlockBinaryGridWidget::new(&self.current)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(game_area, buf),
            Mode::Quadrant => QuadrantBinaryGridWidget::new(&self.current)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(game_area, buf),
            Mode::Sextant => SextantBinaryGridWidget::new(&self.current)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(game_area, buf),
            Mode::Braille => BrailleBinaryGridWidget::new(&self.current)
                .with_pan_x(self.pan_x)
                .with_pan_y(self.pan_y)
                .render(game_area, buf),
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
