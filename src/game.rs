use console_engine::{Color, ConsoleEngine, KeyCode};
use console_engine::pixel::{Pixel, pxl, pxl_fbg, pxl_fg};
use console_engine::screen::Screen;
use itertools::Itertools;
use rand::Rng;
use crate::collections::Vec2D;
use crate::geom::{Point2D, Size2D};
use crate::state::{GameState, SystemEvent};

const KEY_OPEN: KeyCode = KeyCode::Char(' ');
const KEY_FLAG: KeyCode = KeyCode::Char('f');
const NUMBER_COLORS: [Color; 6] = [Color::Cyan, Color::DarkCyan, Color::Yellow, Color::DarkYellow, Color::Magenta, Color::Red];

#[derive(Debug, Copy, Clone, PartialEq, Default)]
enum CellType {
    #[default]
    Water,
    Mine,
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
enum CellState {
    #[default]
    Closed,
    Flagged,
    Opened,
}

impl CellState {
    fn open(&self) -> CellState {
        match self {
            CellState::Closed => CellState::Opened,
            state => *state,
        }
    }

    fn toggle_flag(&self) -> CellState {
        match self {
            CellState::Closed => CellState::Flagged,
            CellState::Flagged => CellState::Closed,
            CellState::Opened => CellState::Opened,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
struct Cell {
    cell_type: CellType,
    state: CellState,
}

impl Cell {
    fn open(&mut self) {
        self.state = self.state.open();
    }

    fn is_open(&self) -> bool {
        self.state.eq(&CellState::Opened)
    }

    fn flag(&mut self) {
        self.state = self.state.toggle_flag();
    }
}

pub struct Minefield {
    data: Vec2D<Cell>,
}

impl Minefield {
    fn with_data(data: Vec2D<Cell>) -> Self {
        Self {
            data,
        }
    }

    fn size(&self) -> &Size2D {
        &self.data.size
    }

    fn get(&self, location: &Point2D) -> Option<&Cell> {
        self.data.get(location)
    }

    fn get_mut(&mut self, location: &Point2D) -> Option<&mut Cell> {
        self.data.get_mut(location)
    }

    fn count_neighbours(&self, location: &Point2D) -> u8 {
        let neighbours = location.neighbours();
        neighbours.iter()
            .filter_map(|point| self.get(point))
            .filter(|cell| cell.cell_type.eq(&CellType::Mine))
            .count() as u8
    }

    fn open(&mut self, location: &Point2D) -> Option<CellType> {
        let mut opened_type = None;

        if let Some(cell) = self.get_mut(location) {
            let opened = !cell.is_open();
            opened_type = if opened { Some(cell.cell_type) } else { None };
            cell.open();
            if !cell.is_open() {
                opened_type = None;
            }
        };
        let opened_water = opened_type.eq(&Some(CellType::Water));
        let no_neighbouring_mines = self.count_neighbours(location) == 0;
        if opened_water && no_neighbouring_mines {
            location.neighbours().iter().for_each(|neighbour| {
                self.open(neighbour);
            });
        }
        opened_type
    }

    fn flag(&mut self, location: &Point2D) {
        if let Some(cell) = self.get_mut(location) {
            cell.flag();
        }
    }

    fn draw(&self) -> Screen {
        let mut screen = Screen::new_fill(self.size().0 as u32 * 2 - 1, self.size().1 as u32, pxl(' '));
        self.data.all_locations().into_iter()
            .for_each(|location| {
                self.draw_cell(&location, &mut screen);
            });
        screen
    }

    fn draw_cell(&self, location: &Point2D, screen: &mut Screen) {
        if let Some(cell) = self.get(location) {
            let pixel = self.pixel_for_cell(location, cell);
            screen.set_pxl((location.0 * 2 + 1) as i32, location.1 as i32, pixel);
        }
    }

    fn pixel_for_cell(&self, location: &Point2D, cell: &Cell) -> Pixel {
        match cell.state {
            CellState::Closed => pxl('?'),
            CellState::Flagged => pxl_fbg('F', Color::White, Color::DarkGreen),
            CellState::Opened => self.pixel_for_open_cell(location, cell)
        }
    }

    fn pixel_for_open_cell(&self, location: &Point2D, cell: &Cell) -> Pixel {
        match cell.cell_type {
            CellType::Mine => pxl_fbg('M', Color::White, Color::DarkRed),
            CellType::Water => match self.count_neighbours(location) {
                0 => pxl(' '),
                num => pxl_fg((num + 0x30) as char, color_for_number(num)),
            }
        }
    }

    fn reveal_all(&mut self) {
        self.data.all_locations().into_iter()
            .for_each(|location| {
                self.open(&location);
            });
    }

    fn only_mines_remaining(&self) -> bool {
        (0..self.size().0).cartesian_product(0..self.size().1)
            .filter_map(|(x, y)| self.get(&Point2D(x, y)))
            .filter(|cell| !cell.is_open() && cell.cell_type == CellType::Water)
            .count() == 0
    }
}

pub struct RandomMineFieldGenerator<T> where T: Rng {
    pub random: T
}

impl<T> RandomMineFieldGenerator<T> where T: Rng {
    pub fn generate(&mut self, size: Size2D, mine_count: usize) -> Minefield {
        if size.0 * size.1 < mine_count {
            panic!("Cannot place more mines than there are cells!");
        }
        let mut cells = Vec2D::sized(&size, Cell::default());
        let mut mines_placed = 0;
        loop {
            let new_location = Point2D(self.random.gen_range(0..size.0), self.random.gen_range(0..size.1));
            let cell = cells.get_mut(&new_location).unwrap();
            if cell.cell_type == CellType::Water {
                cell.cell_type = CellType::Mine;
                mines_placed += 1;
                if mines_placed == mine_count {
                    break;
                }
            }
        }
        Minefield::with_data(cells)
    }
}

fn color_for_number(number: u8) -> Color {
    NUMBER_COLORS.get(number as usize).cloned().unwrap_or(Color::White)
}

pub struct Game {
    field: Minefield,
    cursor: Point2D,
    game_over: bool,
    won: bool,
}

impl Game {
    pub fn with_minefield(field: Minefield) -> Self {
        Self {
            field,
            cursor: Point2D::default(),
            game_over: false,
            won: false,
        }
    }

    fn move_cursor(&mut self, engine: &ConsoleEngine) {
        if engine.is_key_pressed(KeyCode::Left) && self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
        if engine.is_key_pressed(KeyCode::Up) && self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        }
        if engine.is_key_pressed(KeyCode::Right) {
            self.cursor.0 += 1;
        }
        if engine.is_key_pressed(KeyCode::Down) {
            self.cursor.1 += 1;
        }
        self.cursor.clip_excl(self.field.size());
    }
}

impl GameState for Game {

    fn update(&mut self, engine: &ConsoleEngine) -> Option<SystemEvent> {
        if engine.is_key_pressed(KeyCode::Esc) {
            return Some(SystemEvent::Exit);
        }
        if self.game_over || self.won {
            return None;
        }
        self.move_cursor(engine);
        let mut opened_type = None;
        if engine.is_key_pressed(KEY_OPEN) {
            opened_type = self.field.open(&self.cursor);
        }
        if engine.is_key_pressed(KEY_FLAG) {
            self.field.flag(&self.cursor);
        }

        if let Some(CellType::Mine) = opened_type {
            self.field.reveal_all();
            self.game_over = true;
        } else if self.field.only_mines_remaining() {
            self.field.reveal_all();
            self.won = true;
        }

        None
    }

    fn draw(&self, screen: &mut Screen) {
        let field_screen = self.field.draw();
        let field_offset_x = screen.get_width() / 2 - field_screen.get_width() / 2;
        let field_offset_y = screen.get_height() / 2 - field_screen.get_height() / 2;
        screen.print_screen(field_offset_x as i32, field_offset_y as i32, &field_screen);
        screen.set_pxl((self.cursor.0 * 2 + field_offset_x as usize) as i32, (self.cursor.1 + field_offset_y as usize) as i32, pxl('['));
        screen.set_pxl((self.cursor.0 * 2 + 2 + field_offset_x as usize) as i32, (self.cursor.1 + field_offset_y as usize) as i32, pxl(']'));

        let message_offset_y = (field_offset_y + field_screen.get_height() + 3) as i32;
        if self.game_over {
            screen.print(get_message_offset_x(screen, "Game Over!"), message_offset_y, "Game Over!");
        } else if self.won {
            screen.print(get_message_offset_x(screen, "You Won!"), message_offset_y, "You Won!");
        }
    }
}

fn get_message_offset_x(screen: &Screen, msg: &str) -> i32 {
    (screen.get_width() / 2 - msg.len() as u32 / 2) as i32
}

#[cfg(test)]
mod tests {

    mod cell_state {
        use crate::game::CellState;

        #[test]
        fn toggle_flag() {
            assert_eq!(CellState::Opened, CellState::Opened.toggle_flag());
            assert_eq!(CellState::Flagged, CellState::Closed.toggle_flag());
            assert_eq!(CellState::Closed, CellState::Flagged.toggle_flag());
        }

        #[test]
        fn open() {
            assert_eq!(CellState::Opened, CellState::Opened.open());
            assert_eq!(CellState::Opened, CellState::Closed.open());
            assert_eq!(CellState::Flagged, CellState::Flagged.open());
        }
    }

    mod minefield {
        use crate::collections::Vec2D;
        use crate::game::{Cell, CellState, CellType, Minefield};
        use crate::geom::{Point2D, Size2D};

        #[test]
        fn cannot_open_flagged() {
            let mut minefield = Minefield::with_data(Vec2D::sized(&Size2D(5, 5), Cell::default()));
            let location = Point2D(0, 0);
            minefield.flag(&location);
            minefield.open(&location);
            assert!(!minefield.get(&location).unwrap().is_open());
        }

        #[test]
        fn only_mines_remaining_in_water_only_field() {
            let mut minefield = Minefield::with_data(Vec2D::sized(&Size2D(5, 5), Cell::default()));
            minefield.reveal_all();
            assert!(minefield.only_mines_remaining());
        }

        #[test]
        fn mines_remaining_with_closed_water() {
            let mut data = Vec2D::sized(&Size2D(2, 1), Cell::default());
            data.get_mut(&Point2D(0, 0)).unwrap().state = CellState::Opened;
            let minefield = Minefield::with_data(data);
            assert!(!minefield.only_mines_remaining());
        }

        #[test]
        fn mines_remaining_mixed() {
            let mut data = Vec2D::sized(&Size2D(2, 1), Cell::default());
            data.get_mut(&Point2D(0, 0)).unwrap().state = CellState::Opened;
            data.get_mut(&Point2D(1, 0)).unwrap().cell_type = CellType::Mine;
            let minefield = Minefield::with_data(data);
            assert!(minefield.only_mines_remaining());
        }
    }

    mod generator {
        use rand::thread_rng;
        use crate::game::{CellType, RandomMineFieldGenerator};
        use crate::geom::Size2D;

        #[test]
        fn generator_puts_correct_number_of_mines() {
            let mut generator = RandomMineFieldGenerator {
                random: thread_rng(),
            };
            let minefield = generator.generate(Size2D(10, 10), 15);
            let mine_count = minefield.data.all_locations().into_iter()
                .filter(|location| minefield.get(location).unwrap().cell_type == CellType::Mine)
                .count();
            assert_eq!(15, mine_count);
        }
    }
}