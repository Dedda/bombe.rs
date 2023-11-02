use std::cmp::min;
use std::ops::{Add, Sub};
use console_engine::{Color, ConsoleEngine, KeyCode};
use console_engine::pixel::{Pixel, pxl, pxl_fbg, pxl_fg};
use console_engine::screen::Screen;
use rand::{Rng, thread_rng};

const KEY_OPEN: KeyCode = KeyCode::Char(' ');
const KEY_FLAG: KeyCode = KeyCode::Char('f');
const NUMBER_COLORS: [Color; 6] = [Color::Cyan, Color::DarkCyan, Color::Yellow, Color::DarkYellow, Color::Magenta, Color::Red];
const RAINBOW_COLORS: [Color; 6] = [Color::Blue, Color::Cyan, Color::Green, Color::Yellow, Color::Red, Color::Magenta];
const MAIN_MENU_HEADER: &str = include_str!("../assets/main_menu_header.txt");


#[derive(Debug, Clone)]
struct Size2D(usize, usize);

#[derive(Debug, Clone, Default, PartialEq)]
struct Point2D(usize, usize);

impl Size2D {
    fn contains(&self, point: &Point2D) -> bool {
        point.0 < self.0 && point.1 < self.1
    }
}

impl Point2D {
    fn clip_excl(&mut self, size2d: &Size2D) {
        self.0 = min(self.0, sub_non_zero(&size2d.0, 1));
        self.1 = min(self.1, sub_non_zero(&size2d.1, 1));
    }

    fn neighbours(&self) -> Vec<Point2D> {
        let origin = Point2D(self.0 + 1, self.1 + 1);
        vec![
            Point2D(origin.0 - 1, origin.1 - 1),
            Point2D(origin.0, origin.1 - 1),
            Point2D(origin.0 + 1, origin.1 - 1),
            Point2D(origin.0 - 1, origin.1),
            Point2D(origin.0 + 1, origin.1),
            Point2D(origin.0 - 1, origin.1 + 1),
            Point2D(origin.0, origin.1 + 1),
            Point2D(origin.0 + 1, origin.1 + 1),
        ].into_iter()
            .filter(|point| point.0 > 0 && point.1 > 0)
            .map(|point| Point2D(point.0 - 1, point.1 - 1))
            .collect()
    }
}

impl Add<&Point2D> for Point2D {
    type Output = Point2D;

    fn add(self, rhs: &Point2D) -> Self::Output {
        Point2D(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub<&Point2D> for Point2D {
    type Output = Point2D;

    fn sub(self, rhs: &Point2D) -> Self::Output {
        Point2D(
            self.0.saturating_sub(rhs.0),
            self.1.saturating_sub(rhs.1),
        )
    }
}

fn sub_non_zero(a: &usize, b: usize) -> usize {
    a.checked_sub(b).unwrap_or(0)
}

struct Vec2D<T> {
    size: Size2D,
    data: Vec<Vec<T>>,
}

impl<T> Vec2D<T> {
    fn sized(size: &Size2D, default: T) -> Self where T: Copy {
        Self {
            size: size.clone(),
            data: vec![vec![default; size.1]; size.0],
        }
    }

    fn get(&self, point2d: &Point2D) -> Option<&T> {
        if !self.size.contains(point2d) {
            None
        } else {
            self.data.get(point2d.0)?.get(point2d.1)
        }
    }

    fn get_mut(&mut self, point2d: &Point2D) -> Option<&mut T> {
        if !self.size.contains(point2d) {
            None
        } else {
            self.data.get_mut(point2d.0)?.get_mut(point2d.1)
        }
    }

    fn all_locations(&self) -> Vec<Point2D> {
        (0..self.size.0)
            .flat_map(|x| (0..self.size.1)
                .map(move |y| Point2D(x, y)))
            .collect()
    }
}

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

struct Minefield {
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
        for x in 0 .. self.size().0 {
            for y in 0 .. self.size().1 {
                if let Some(cell) = self.get(&Point2D(x, y)) {
                    if !cell.is_open() && cell.cell_type == CellType::Water {
                        return false;
                    }
                }
            }
        }
        true
    }
}

struct RandomMineFieldGenerator<T> where T: Rng {
    random: T
}

impl<T> RandomMineFieldGenerator<T> where T: Rng {
    fn generate(&mut self, size: Size2D, mine_count: usize) -> Minefield {
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

enum SystemEvent {
    ChangeState(Box<dyn GameState>),
    Exit,
}

trait GameState {
    fn update(&mut self, engine: &ConsoleEngine) -> Option<SystemEvent>;

    fn draw(&self, screen: &mut Screen);
}

#[derive(Debug, Clone, PartialEq)]
enum MainMenuCursorPosition {
    Width = 0,
    Height,
    MineCount,
    StartGame,
}

impl MainMenuCursorPosition {
    fn next(&self) -> MainMenuCursorPosition {
        use MainMenuCursorPosition::*;
        match self {
            Width => Height,
            Height => MineCount,
            MineCount => StartGame,
            StartGame => Width,
        }
    }

    fn prev(&self) -> MainMenuCursorPosition {
        use MainMenuCursorPosition::*;
        match self {
            StartGame => MineCount,
            MineCount => Height,
            Height => Width,
            Width => StartGame,
        }
    }
}

struct MainMenu {
    cursor_position: MainMenuCursorPosition,
    width: usize,
    height: usize,
    mine_count: usize,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self {
            cursor_position: MainMenuCursorPosition::StartGame,
            width: 10,
            height: 10,
            mine_count: 10,
        }
    }
}

impl MainMenu {
    fn start_game(&self) -> SystemEvent {
        let minefield = RandomMineFieldGenerator {
            random: thread_rng(),
        }.generate(Size2D(self.width, self.height), self.mine_count);
        let game = Game::with_minefield(minefield);
        SystemEvent::ChangeState(Box::new(game))
    }
}

impl GameState for MainMenu {
    fn update(&mut self, engine: &ConsoleEngine) -> Option<SystemEvent> {
        if engine.is_key_pressed(KeyCode::Esc) {
            return Some(SystemEvent::Exit);
        }
        if engine.is_key_pressed(KeyCode::Up) {
            self.cursor_position = self.cursor_position.prev();
        }
        if engine.is_key_pressed(KeyCode::Down) {
            self.cursor_position = self.cursor_position.next();
        }
        if self.cursor_position == MainMenuCursorPosition::StartGame && engine.is_key_pressed(KeyCode::Enter) {
            return Some(self.start_game());
        }
        None
    }

    fn draw(&self, screen: &mut Screen) {
        const WIDTH: i32 = 13;
        const HEIGHT: i32 = 7;

        let center_x = screen.get_width() as i32 / 2;
        let center_y = screen.get_height() as i32 / 2;
        let offset_x = center_x - WIDTH / 2;
        let offset_y = center_y - HEIGHT / 2;
        let text_x = offset_x + 2;

        let header_width = MAIN_MENU_HEADER.lines().map(|line| line.len()).max().unwrap_or(0) as u32;
        let header_height = MAIN_MENU_HEADER.lines().count() as u32;

        if screen.get_width() >= header_width && offset_y > (header_height + 2) as i32 {
            let offset_x = center_x - header_width as i32 / 2;
            let offset_y = offset_y / 2 - header_height as i32 / 2;
            MAIN_MENU_HEADER.lines().enumerate().for_each(|(idx, line)| {
                let color = RAINBOW_COLORS.get(idx % RAINBOW_COLORS.len()).unwrap();
                screen.print_fbg(offset_x, offset_y + idx as i32, line, *color, Color::Reset);
            });
            // screen.print(center_x - header_width as i32 / 2, offset_y / 2 - header_height as i32 / 2, MAIN_MENU_HEADER);
        }

        screen.print(text_x, offset_y, &format!("Width: {}", self.width));
        screen.print(text_x, offset_y + 2, &format!("Height: {}", self.height));
        screen.print(text_x, offset_y + 4, &format!("Mines: {}", self.mine_count));
        screen.print(text_x, offset_y + 6, "Start Game");
        screen.set_pxl(offset_x, offset_y + self.cursor_position.clone() as i32 * 2, pxl('*'))
    }
}

struct Game {
    field: Minefield,
    cursor: Point2D,
    game_over: bool,
    won: bool,
}

impl Game {
    fn with_minefield(field: Minefield) -> Self {
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

fn main() {
    let mut game_state: Box<dyn GameState> = Box::<MainMenu>::default();

    let mut engine = ConsoleEngine::init_fill_require(42, 25, 15).unwrap();

    loop {
        engine.wait_frame();
        if let Some(event) = game_state.update(&engine) {
            match event {
                SystemEvent::ChangeState(new_state) => {
                    game_state = new_state;
                    continue;
                }
                SystemEvent::Exit => break,
            }
        }
        engine.check_resize();
        engine.clear_screen();
        let mut screen = engine.get_screen();
        game_state.draw(&mut screen);
        engine.set_screen(&screen);
        engine.draw();
    }
}

#[cfg(test)]
mod tests {
    use crate::sub_non_zero;

    #[test]
    fn sub_non_zero_under_zero() {
        assert_eq!(0, sub_non_zero(&6, 8));
    }

    mod size2d {
        use crate::{Point2D, Size2D};

        #[test]
        fn contains_point() {
            let size = Size2D(3, 4);
            let point = Point2D(2, 3);
            assert!(size.contains(&point));
        }

        #[test]
        fn not_contains_width() {
            let size = Size2D(3, 4);
            let point = Point2D(3, 3);
            assert!(!size.contains(&point));
        }

        #[test]
        fn not_contains_height() {
            let size = Size2D(3, 4);
            let point = Point2D(2, 4);
            assert!(!size.contains(&point));
        }
    }

    mod point2d {
        use crate::{Point2D, Size2D};

        #[test]
        fn add_points() {
            let p1 = Point2D(1, 2);
            let p2 = Point2D(3, 4);
            assert_eq!(Point2D(4, 6), p1 + &p2);
        }

        #[test]
        fn clip() {
            let mut point = Point2D(10, 10);
            let size = Size2D(6, 8);
            point.clip_excl(&size);
            assert_eq!(Point2D(5, 7), point);
        }

        #[test]
        fn all_neighbours() {
            let point = Point2D(1, 1);
            assert_eq!(8, point.neighbours().len());
        }

        #[test]
        fn neighbours_for_origin() {
            assert_eq!(3, Point2D::default().neighbours().len());
        }
    }

    mod vec2d {
        use crate::{Point2D, Size2D, Vec2D};

        #[test]
        fn get_value_in_single_cell_vec2d() {
            let v = Vec2D::sized(&Size2D(1, 1), 5);
            assert_eq!(&5, v.get(&Point2D(0, 0)).unwrap());
        }
    }

    mod cell_state {
        use crate::CellState;

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
        use crate::{Cell, CellState, CellType, Minefield, Point2D, Size2D, Vec2D};

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
        use crate::{CellType, RandomMineFieldGenerator, Size2D};

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