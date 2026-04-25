//! This crate provides widgets for rendering colored grids in [Ratatui](https://docs.rs/ratatui/latest/ratatui/index.html).
//! # Feature Flags (all on by default)
//! - binary
//!     - You might want this disabled if you're not using it since it brings in dependencies ([bitvec](https://docs.rs/bitvec/1.0.1/bitvec/) and [whatever it depends on](https://crates.io/crates/bitvec/dependencies))
//! - color
//!     - Honestly, this is just a feature because binary is. This doesn't bring in any dependencies.
//!       But if you're using binary grids but not color grids and you want to minimize your compile
//!       time...
//! - alloc
//!     - This enables `state::alloc` which contains heap-allocated grids
//!       for cases where you don't know the size at compile time.
//!       They are not resizeable (at least for now).
//! # Color Grids
//! Instead of storing [Color]s,
//! it takes in a generic type that implements [ToColor]
//! so you can use your own type for the grid and use it to implement logic better,
//! and then just add the trait so it knows how to turn it into a [Color].
//! If you don't provide a type, it will use [Color] by default.
//! # Binary Grids
//! At the cost of being limited to 2 colors instead of 16777216, these have 2 advantages over color grids:
//! 1. Smaller memory usage, since each cell only takes up 1 bit
//! 2. Higher resolution options using quadrants, sextants, or braille.
//! # Coordinates
//! Like Ratatui, the coordinate system in the grids runs left to right, top to bottom, with the origin (0, 0) in the top left corner.
#[cfg(all(feature = "binary", feature = "alloc"))]
use bitvec::vec::BitVec;
use ratatui_core::style::Color;
use ratatui_core::{layout::Position, widgets::Widget};

#[cfg(feature = "binary")]
pub trait BinaryGrid {
    fn _getb(&self, x: usize, y: usize) -> Option<bool>;
}

#[cfg(feature = "color")]
pub trait ColorGrid {
    fn _getc(&self, x: usize, y: usize) -> Option<Color>;
}

pub trait ToColor {
    fn to_color(&self) -> Color;
}

impl ToColor for Color {
    fn to_color(&self) -> Color {
        *self
    }
}

pub mod grid {
    pub mod stack {
        use super::super::*;
        #[derive(Debug, Clone, Copy)]
        #[cfg(feature = "color")]
        pub struct StackColoredGrid<const WIDTH: usize, const HEIGHT: usize, T: ToColor = Color>(
            pub [[T; WIDTH]; HEIGHT],
        );

        impl<const WIDTH: usize, const HEIGHT: usize, T: ToColor> StackColoredGrid<WIDTH, HEIGHT, T> {
            /// Creates a new ColoredGridState with all values set to the provided value
            pub fn new_filled(value: T) -> Self
            where
                T: Copy,
            {
                Self([[value; WIDTH]; HEIGHT])
            }

            /// Creates a new ColoredGridState with all values set to the provided value,
            /// cloning it for each cell
            pub fn new_filled_clone(value: T) -> Self
            where
                T: Clone,
            {
                Self(std::array::from_fn(|_| {
                    std::array::from_fn(|_| value.clone())
                }))
            }

            /// Creates a new ColoredGridState will all values initialized by calling f(x, y)
            pub fn from_fn(mut f: impl FnMut(usize, usize) -> T) -> Self {
                Self(std::array::from_fn(|y| std::array::from_fn(|x| f(x, y))))
            }

            pub fn get(&self, x: usize, y: usize) -> Option<&T> {
                self.0.get(y)?.get(x)
            }

            pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
                self.0.get_mut(y)?.get_mut(x)
            }
            /// Panics if out of bounds
            pub fn set(&mut self, x: usize, y: usize, value: T) {
                self.0[y][x] = value
            }

            pub fn area(&self) -> usize {
                WIDTH * HEIGHT
            }
        }

        impl<const WIDTH: usize, const HEIGHT: usize, T: ToColor> ColorGrid
            for StackColoredGrid<WIDTH, HEIGHT, T>
        {
            fn _getc(&self, x: usize, y: usize) -> Option<Color> {
                self.get(x, y).map(ToColor::to_color)
            }
        }
    }

    #[cfg(feature = "alloc")]
    pub mod alloc {
        use super::super::*;

        #[derive(Debug, Clone)]
        #[cfg(feature = "color")]
        pub struct AllocColoredGrid<T: ToColor = Color> {
            width: usize,
            height: usize,
            grid: Vec<T>,
        }

        impl<T: ToColor> AllocColoredGrid<T> {
            /// Creates a new ColoredGridState with all values set to the provided value
            pub fn new_filled(width: usize, height: usize, value: T) -> Self
            where
                T: Clone,
            {
                Self {
                    width,
                    height,
                    grid: Vec::from_iter(std::iter::repeat_n(value, width * height)),
                }
            }

            /// Creates a new ColoredGridState will all values initialized by calling f(x, y)
            pub fn from_fn(
                width: usize,
                height: usize,
                mut f: impl FnMut(usize, usize) -> T,
            ) -> Self {
                let mut grid = Vec::with_capacity(width * height);
                for y in 0..height {
                    for x in 0..width {
                        grid.push(f(x, y));
                    }
                }
                Self {
                    width,
                    height,
                    grid,
                }
            }

            pub fn get(&self, x: usize, y: usize) -> Option<&T> {
                if x >= self.width || y >= self.height {
                    return None;
                }
                self.grid.get(y * self.width + x)
            }

            pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
                if x >= self.width || y >= self.height {
                    return None;
                }
                self.grid.get_mut(y * self.width + x)
            }

            /// Panics if out of bounds
            pub fn set(&mut self, x: usize, y: usize, value: T) {
                assert!(x < self.width);
                assert!(y < self.height);
                self.grid[y * self.width + x] = value
            }

            pub fn area(&self) -> usize {
                self.width * self.height
            }
        }

        impl<T: ToColor> ColorGrid for AllocColoredGrid<T> {
            fn _getc(&self, x: usize, y: usize) -> Option<Color> {
                self.get(x, y).map(ToColor::to_color)
            }
        }

        #[derive(Debug, Clone)]
        #[cfg(feature = "binary")]
        pub struct AllocBinaryGrid {
            width: usize,
            height: usize,
            grid: BitVec,
        }

        impl AllocBinaryGrid {
            /// Creates a new BinaryGridState with all values set to the provided bit
            pub fn new_filled(width: usize, height: usize, bit: bool) -> Self {
                Self {
                    width,
                    height,
                    grid: BitVec::repeat(bit, width * height),
                }
            }

            /// Creates a new BinaryGridState will all values initialized by calling f(x, y)
            pub fn from_fn(
                width: usize,
                height: usize,
                mut f: impl FnMut(usize, usize) -> bool,
            ) -> Self {
                let mut grid = BitVec::with_capacity(width * height);
                for y in 0..height {
                    for x in 0..width {
                        grid.push(f(x, y));
                    }
                }
                Self {
                    width,
                    height,
                    grid,
                }
            }

            pub fn get(&self, x: usize, y: usize) -> Option<bool> {
                if x >= self.width || y >= self.height {
                    return None;
                }
                self.grid.get(y * self.width + x).as_deref().copied()
            }

            /// Panics if out of bounds
            pub fn set(&mut self, x: usize, y: usize, bit: bool) {
                assert!(x < self.width);
                assert!(y < self.height);
                self.grid.set(y * self.width + x, bit);
            }

            pub fn area(&self) -> usize {
                self.width * self.height
            }
        }

        impl BinaryGrid for AllocBinaryGrid {
            fn _getb(&self, x: usize, y: usize) -> Option<bool> {
                self.get(x, y)
            }
        }
    }
}

pub mod widget {
    #[cfg(feature = "color")]
    pub mod color {
        use crate::*;
        macro_rules! color_widget {
            ($(#[$attr:meta])* $ident:ident) => {
                $(#[$attr])*
                /// # bg
                /// If the render area is bigger than the grid,
                /// the remaining area will be the terminal's default background color ([Color::Reset]).
                /// You can change that with [Self::with_bg].
                #[derive(Debug)]
                pub struct $ident<'a, T: ColorGrid> {
                    bg: Color,
                    grid: &'a T,
                }
                impl<'a, T: ColorGrid> $ident<'a, T> {
                    pub fn new(grid: &'a T) -> Self {
                        Self {
                            bg: Color::Reset,
                            grid,
                        }
                    }
                    pub fn with_bg(self, bg: Color) -> Self {
                        Self {
                            bg,
                            grid: self.grid,
                        }
                    }
                }
            };
        }
        color_widget!(
            /// Uses Unicode full blocks which fill an entire terminal cell.
            /// # Used Characters:
            /// - █
            FullBlockColorGridWidget
        );
        impl<T: ColorGrid> Widget for FullBlockColorGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();
                        let grid_x = x as usize;
                        let grid_y = y as usize;
                        match self.grid._getc(grid_x, grid_y) {
                            None => {
                                cell.set_char(' ');
                                cell.bg = self.bg;
                            }
                            Some(color) => {
                                cell.set_char('█');
                                cell.fg = color;
                            }
                        }
                    }
                }
            }
        }

        color_widget!(
            /// Uses Unicode half blocks and full blocks
            /// which allows you to have 2 colors per terminal cell,
            /// doubling the vertical resolution.
            /// # Used Characters:
            /// - ▀
            /// - █
            /// - ▄
            HalfBlockColorGridWidget
        );
        impl<T: ColorGrid> Widget for HalfBlockColorGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();
                        let grid_x = x as usize;
                        let grid_y = y as usize * 2;
                        match (
                            self.grid._getc(grid_x, grid_y),
                            self.grid._getc(grid_x, grid_y + 1),
                        ) {
                            (None, None) => {
                                cell.set_char(' ');
                                cell.bg = self.bg;
                            }
                            (None, Some(bottom)) => {
                                cell.set_char('▄');
                                cell.fg = bottom;
                                cell.bg = self.bg;
                            }
                            (Some(top), None) => {
                                cell.set_char('▀');
                                cell.fg = top;
                                cell.bg = self.bg;
                            }
                            (Some(top), Some(bottom)) => {
                                cell.set_char('▀');
                                cell.fg = top;
                                cell.bg = bottom;
                            }
                        }
                    }
                }
            }
        }

        color_widget!(
            /// Uses Unicode half blocks and full blocks
            /// which allows you to have 2 colors per terminal cell,
            /// doubling the horizontal resolution,
            /// albeit with perhaps weirdly skinny "pixels".
            /// # Used Characters:
            /// - █
            /// - ▌
            /// - ▐
            HorizontalHalfBlockColorGridWidget
        );
        impl<T: ColorGrid> Widget for HorizontalHalfBlockColorGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();
                        let grid_x = x as usize * 2;
                        let grid_y = y as usize;
                        match (
                            self.grid._getc(grid_x, grid_y),
                            self.grid._getc(grid_x + 1, grid_y),
                        ) {
                            (None, None) => {
                                cell.set_char(' ');
                                cell.bg = self.bg;
                            }
                            (None, Some(right)) => {
                                cell.set_char('▐');
                                cell.fg = right;
                                cell.bg = self.bg;
                            }
                            (Some(left), None) => {
                                cell.set_char('▌');
                                cell.fg = left;
                                cell.bg = self.bg;
                            }
                            (Some(left), Some(right)) => {
                                cell.set_char('▌');
                                cell.fg = left;
                                cell.bg = right;
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(feature = "binary")]
    pub mod binary {
        use crate::*;
        macro_rules! binary_widget {
            ($(#[$attr:meta])* $ident:ident) => {
                $(#[$attr])*
                /// # fg and bg
                /// By default, fg and bg will be your terminal's default foreground and background color respectively ([Color::Reset]).
                /// These can be changed with [Self::with_fg] and [Self::with_bg] respectively.
                /// If the render area is bigger than the grid,
                /// the remaining area will be bg.
                #[derive(Debug)]
                pub struct $ident<'a, T: BinaryGrid> {
                    fg: Color,
                    bg: Color,
                    grid: &'a T,
                }
                impl<'a, T: BinaryGrid> $ident<'a, T> {
                    pub fn new(grid: &'a T) -> Self {
                        Self {
                            fg: Color::Reset,
                            bg: Color::Reset,
                            grid,
                        }
                    }
                    pub fn with_fg(self, fg: Color) -> Self {
                        Self {
                            fg,
                            ..self
                        }
                    }
                    pub fn with_bg(self, bg: Color) -> Self {
                        Self {
                            bg,
                            ..self
                        }
                    }

                    fn get(&self, x: usize, y: usize) -> bool {
                        self.grid._getb(x, y).unwrap_or(false)
                    }
                }
            };
        }
        binary_widget!(
            /// Uses Unicode full blocks which fill an entire terminal cell.
            /// # Used Characters:
            /// - █
            FullBlockBinaryGridWidget
        );
        impl<T: BinaryGrid> Widget for FullBlockBinaryGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();
                        let grid_x = x as usize;
                        let grid_y = y as usize;
                        if self.get(grid_x, grid_y) {
                            cell.set_char('█');
                            cell.fg = self.fg;
                        } else {
                            cell.set_char(' ');
                            cell.bg = self.bg;
                        }
                    }
                }
            }
        }

        binary_widget!(
            /// Uses Unicode half blocks and full blocks,
            /// doubling the vertical resolution.
            /// # Used Characters:
            /// - ▀
            /// - █
            /// - ▄
            HalfBlockBinaryGridWidget
        );
        impl<T: BinaryGrid> Widget for HalfBlockBinaryGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();
                        cell.fg = self.fg;
                        cell.bg = self.bg;
                        let grid_x = x as usize;
                        let grid_y = y as usize * 2;
                        match (self.get(grid_x, grid_y), self.get(grid_x, grid_y + 1)) {
                            (false, false) => {
                                cell.set_char(' ');
                            }
                            (false, true) => {
                                cell.set_char('▄');
                            }
                            (true, false) => {
                                cell.set_char('▀');
                            }
                            (true, true) => {
                                cell.set_char('█');
                            }
                        }
                    }
                }
            }
        }

        binary_widget!(
            /// Uses Unicode half blocks and full blocks,
            /// doubling the horizontal resolution,
            /// albeit with perhaps weirdly skinny "pixels".
            /// # Used Characters:
            /// - █
            /// - ▌
            /// - ▐
            HorizontalHalfBlockBinaryGridWidget
        );
        impl<T: BinaryGrid> Widget for HorizontalHalfBlockBinaryGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();
                        cell.fg = self.fg;
                        cell.bg = self.bg;
                        let grid_x = x as usize * 2;
                        let grid_y = y as usize;
                        match (self.get(grid_x, grid_y), self.get(grid_x + 1, grid_y)) {
                            (false, false) => {
                                cell.set_char(' ');
                            }
                            (false, true) => {
                                cell.set_char('▐');
                            }
                            (true, false) => {
                                cell.set_char('▌');
                            }
                            (true, true) => {
                                cell.set_char('█');
                            }
                        }
                    }
                }
            }
        }

        binary_widget!(
            /// Uses Unicode quadrants, half blocks, and full blocks
            /// which only allows you to have 2 colors in the whole grid,
            /// but with double the vertical and horizontal resolution.
            /// If the render area is bigger than the grid,
            /// the remaining area will be bg.
            /// # Used Characters:
            /// - ▘
            /// - ▝
            /// - ▀
            /// - ▖
            /// - ▌
            /// - ▞
            /// - ▛
            /// - ▗
            /// - ▚
            /// - ▐
            /// - ▜
            /// - ▄
            /// - ▙
            /// - ▟
            /// - █
            QuadrantBinaryGridWidget
        );
        impl<T: BinaryGrid> Widget for QuadrantBinaryGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();

                        cell.fg = self.fg;
                        cell.bg = self.bg;

                        let grid_x = x as usize * 2;
                        let grid_y = y as usize * 2;

                        match (
                            self.get(grid_x, grid_y),
                            self.get(grid_x + 1, grid_y),
                            self.get(grid_x, grid_y + 1),
                            self.get(grid_x + 1, grid_y + 1),
                        ) {
                            (false, false, false, false) => cell.set_char(' '),
                            (false, false, false, true) => cell.set_char('▗'),
                            (false, false, true, false) => cell.set_char('▖'),
                            (false, false, true, true) => cell.set_char('▄'),
                            (false, true, false, false) => cell.set_char('▝'),
                            (false, true, false, true) => cell.set_char('▐'),
                            (false, true, true, false) => cell.set_char('▞'),
                            (false, true, true, true) => cell.set_char('▟'),
                            (true, false, false, false) => cell.set_char('▘'),
                            (true, false, false, true) => cell.set_char('▚'),
                            (true, false, true, false) => cell.set_char('▌'),
                            (true, false, true, true) => cell.set_char('▙'),
                            (true, true, false, false) => cell.set_char('▀'),
                            (true, true, false, true) => cell.set_char('▜'),
                            (true, true, true, false) => cell.set_char('▛'),
                            (true, true, true, true) => cell.set_char('█'),
                        };
                    }
                }
            }
        }

        binary_widget!(
            /// Uses Unicode sextants, third blocks, half blocks, and full blocks,
            /// which only allows to have 2 colors in the whole grid,
            /// but with double the horizontal resolution and triple the vertical resolution
            /// # Used Characters
            /// - 🬀
            /// - 🬁
            /// - 🬂
            /// - 🬃
            /// - 🬄
            /// - 🬅
            /// - 🬆
            /// - 🬇
            /// - 🬈
            /// - 🬉
            /// - 🬊
            /// - 🬋
            /// - 🬌
            /// - 🬍
            /// - 🬎
            /// - 🬏
            /// - 🬐
            /// - 🬑
            /// - 🬒
            /// - 🬓
            /// - ▌
            /// - 🬔
            /// - 🬕
            /// - 🬖
            /// - 🬗
            /// - 🬘
            /// - 🬙
            /// - 🬚
            /// - 🬛
            /// - 🬜
            /// - 🬝
            /// - 🬞
            /// - 🬟
            /// - 🬠
            /// - 🬡
            /// - 🬢
            /// - 🬣
            /// - 🬤
            /// - 🬥
            /// - 🬦
            /// - 🬧
            /// - ▐
            /// - 🬨
            /// - 🬩
            /// - 🬪
            /// - 🬫
            /// - 🬬
            /// - 🬭
            /// - 🬮
            /// - 🬯
            /// - 🬰
            /// - 🬱
            /// - 🬲
            /// - 🬳
            /// - 🬴
            /// - 🬵
            /// - 🬶
            /// - 🬷
            /// - 🬸
            /// - 🬹
            /// - 🬺
            /// - 🬻
            /// - █
            SextantBinaryGridWidget
        );
        impl<T: BinaryGrid> Widget for SextantBinaryGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                const SEXTANTS: [char; 64] = [
                    ' ', '🬀', '🬁', '🬂', '🬃', '🬄', '🬅', '🬆', '🬇', '🬈', '🬉', '🬊', '🬋', '🬌', '🬍', '🬎',
                    '🬏', '🬐', '🬑', '🬒', '🬓', '▌', '🬔', '🬕', '🬖', '🬗', '🬘', '🬙', '🬚', '🬛', '🬜', '🬝',
                    '🬞', '🬟', '🬠', '🬡', '🬢', '🬣', '🬤', '🬥', '🬦', '🬧', '▐', '🬨', '🬩', '🬪', '🬫', '🬬',
                    '🬭', '🬮', '🬯', '🬰', '🬱', '🬲', '🬳', '🬴', '🬵', '🬶', '🬷', '🬸', '🬹', '🬺', '🬻', '█',
                ];
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();

                        cell.fg = self.fg;
                        cell.bg = self.bg;

                        let grid_x = x as usize * 2;
                        let grid_y = y as usize * 3;

                        let mut index = 0;

                        if self.get(grid_x, grid_y) {
                            index |= 1;
                        }
                        if self.get(grid_x + 1, grid_y) {
                            index |= 2;
                        }
                        if self.get(grid_x, grid_y + 1) {
                            index |= 4;
                        }
                        if self.get(grid_x + 1, grid_y + 1) {
                            index |= 8;
                        }
                        if self.get(grid_x, grid_y + 2) {
                            index |= 16;
                        }
                        if self.get(grid_x + 1, grid_y + 2) {
                            index |= 32;
                        }

                        cell.set_char(SEXTANTS[index]);
                    }
                }
            }
        }

        binary_widget!(
            /// Uses Unicode braille patterns,
            /// which only allows to have 2 colors in the whole grid,
            /// and has true cells being a bit surrounded by the false color,
            /// but with double the horizontal resolution and quadruple the vertical resolution
            /// # Used Characters
            /// - ⠁
            /// - ⠈
            /// - ⠉
            /// - ⠂
            /// - ⠃
            /// - ⠊
            /// - ⠋
            /// - ⠐
            /// - ⠑
            /// - ⠘
            /// - ⠙
            /// - ⠒
            /// - ⠓
            /// - ⠚
            /// - ⠛
            /// - ⠄
            /// - ⠅
            /// - ⠌
            /// - ⠍
            /// - ⠆
            /// - ⠇
            /// - ⠎
            /// - ⠏
            /// - ⠔
            /// - ⠕
            /// - ⠜
            /// - ⠝
            /// - ⠖
            /// - ⠗
            /// - ⠞
            /// - ⠟
            /// - ⠠
            /// - ⠡
            /// - ⠨
            /// - ⠩
            /// - ⠢
            /// - ⠣
            /// - ⠪
            /// - ⠫
            /// - ⠰
            /// - ⠱
            /// - ⠸
            /// - ⠹
            /// - ⠲
            /// - ⠳
            /// - ⠺
            /// - ⠻
            /// - ⠤
            /// - ⠥
            /// - ⠬
            /// - ⠭
            /// - ⠦
            /// - ⠧
            /// - ⠮
            /// - ⠯
            /// - ⠴
            /// - ⠵
            /// - ⠼
            /// - ⠽
            /// - ⠶
            /// - ⠷
            /// - ⠾
            /// - ⠿
            /// - ⡀
            /// - ⡁
            /// - ⡈
            /// - ⡉
            /// - ⡂
            /// - ⡃
            /// - ⡊
            /// - ⡋
            /// - ⡐
            /// - ⡑
            /// - ⡘
            /// - ⡙
            /// - ⡒
            /// - ⡓
            /// - ⡚
            /// - ⡛
            /// - ⡄
            /// - ⡅
            /// - ⡌
            /// - ⡍
            /// - ⡆
            /// - ⡇
            /// - ⡎
            /// - ⡏
            /// - ⡔
            /// - ⡕
            /// - ⡜
            /// - ⡝
            /// - ⡖
            /// - ⡗
            /// - ⡞
            /// - ⡟
            /// - ⡠
            /// - ⡡
            /// - ⡨
            /// - ⡩
            /// - ⡢
            /// - ⡣
            /// - ⡪
            /// - ⡫
            /// - ⡰
            /// - ⡱
            /// - ⡸
            /// - ⡹
            /// - ⡲
            /// - ⡳
            /// - ⡺
            /// - ⡻
            /// - ⡤
            /// - ⡥
            /// - ⡬
            /// - ⡭
            /// - ⡦
            /// - ⡧
            /// - ⡮
            /// - ⡯
            /// - ⡴
            /// - ⡵
            /// - ⡼
            /// - ⡽
            /// - ⡶
            /// - ⡷
            /// - ⡾
            /// - ⡿
            /// - ⢀
            /// - ⢁
            /// - ⢈
            /// - ⢉
            /// - ⢂
            /// - ⢃
            /// - ⢊
            /// - ⢋
            /// - ⢐
            /// - ⢑
            /// - ⢘
            /// - ⢙
            /// - ⢒
            /// - ⢓
            /// - ⢚
            /// - ⢛
            /// - ⢄
            /// - ⢅
            /// - ⢌
            /// - ⢍
            /// - ⢆
            /// - ⢇
            /// - ⢎
            /// - ⢏
            /// - ⢔
            /// - ⢕
            /// - ⢜
            /// - ⢝
            /// - ⢖
            /// - ⢗
            /// - ⢞
            /// - ⢟
            /// - ⢠
            /// - ⢡
            /// - ⢨
            /// - ⢩
            /// - ⢢
            /// - ⢣
            /// - ⢪
            /// - ⢫
            /// - ⢰
            /// - ⢱
            /// - ⢸
            /// - ⢹
            /// - ⢲
            /// - ⢳
            /// - ⢺
            /// - ⢻
            /// - ⢤
            /// - ⢥
            /// - ⢬
            /// - ⢭
            /// - ⢦
            /// - ⢧
            /// - ⢮
            /// - ⢯
            /// - ⢴
            /// - ⢵
            /// - ⢼
            /// - ⢽
            /// - ⢶
            /// - ⢷
            /// - ⢾
            /// - ⢿
            /// - ⣀
            /// - ⣁
            /// - ⣈
            /// - ⣉
            /// - ⣂
            /// - ⣃
            /// - ⣊
            /// - ⣋
            /// - ⣐
            /// - ⣑
            /// - ⣘
            /// - ⣙
            /// - ⣒
            /// - ⣓
            /// - ⣚
            /// - ⣛
            /// - ⣄
            /// - ⣅
            /// - ⣌
            /// - ⣍
            /// - ⣆
            /// - ⣇
            /// - ⣎
            /// - ⣏
            /// - ⣔
            /// - ⣕
            /// - ⣜
            /// - ⣝
            /// - ⣖
            /// - ⣗
            /// - ⣞
            /// - ⣟
            /// - ⣠
            /// - ⣡
            /// - ⣨
            /// - ⣩
            /// - ⣢
            /// - ⣣
            /// - ⣪
            /// - ⣫
            /// - ⣰
            /// - ⣱
            /// - ⣸
            /// - ⣹
            /// - ⣲
            /// - ⣳
            /// - ⣺
            /// - ⣻
            /// - ⣤
            /// - ⣥
            /// - ⣬
            /// - ⣭
            /// - ⣦
            /// - ⣧
            /// - ⣮
            /// - ⣯
            /// - ⣴
            /// - ⣵
            /// - ⣼
            /// - ⣽
            /// - ⣶
            /// - ⣷
            /// - ⣾
            /// - ⣿
            BrailleBinaryGridWidget
        );
        impl<T: BinaryGrid> Widget for BrailleBinaryGridWidget<'_, T> {
            fn render(
                self,
                area: ratatui_core::layout::Rect,
                buf: &mut ratatui_core::buffer::Buffer,
            ) {
                for y in 0..area.height {
                    let buf_y = y + area.y;
                    for x in 0..area.width {
                        let buf_x = x + area.x;
                        let cell = buf.cell_mut(Position { x: buf_x, y: buf_y }).unwrap();

                        cell.fg = self.fg;
                        cell.bg = self.bg;

                        let grid_x = x as usize * 2;
                        let grid_y = y as usize * 4;

                        // Dots 1-3 are the left column
                        // Dots 4-6 are the right column
                        // Dots 7-8 are the bottom row
                        let mut index: u32 = 0;

                        if self.get(grid_x, grid_y) {
                            index |= 1 << 0;
                        }
                        if self.get(grid_x + 1, grid_y) {
                            index |= 1 << 3;
                        }
                        if self.get(grid_x, grid_y + 1) {
                            index |= 1 << 1;
                        }
                        if self.get(grid_x + 1, grid_y + 1) {
                            index |= 1 << 4;
                        }
                        if self.get(grid_x, grid_y + 2) {
                            index |= 1 << 2;
                        }
                        if self.get(grid_x + 1, grid_y + 2) {
                            index |= 1 << 5;
                        }
                        if self.get(grid_x, grid_y + 3) {
                            index |= 1 << 6;
                        }
                        if self.get(grid_x + 1, grid_y + 3) {
                            index |= 1 << 7;
                        }
                        cell.set_char(std::char::from_u32(0x2800 + index).unwrap());
                    }
                }
            }
        }
    }
}
