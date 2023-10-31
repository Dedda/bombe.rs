use std::cmp::min;
use console_engine::{Color, ConsoleEngine, KeyCode};
use console_engine::pixel::{pxl, pxl_fbg, pxl_fg};

const KEY_OPEN: KeyCode = KeyCode::Char(' ');
const KEY_FLAG: KeyCode = KeyCode::Char('f');

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
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum CellType {
    Water,
    Mine,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum CellState {
    Closed,
    Flagged,
    Opened,
}

impl CellState {
    fn open(&self) -> CellState {
        match self {
            CellState::Closed => CellState::Opened,
            state => state.clone(),
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

#[derive(Debug, Copy, Clone, PartialEq)]
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

    fn open(&mut self, location: &Point2D) {
        let mut opened_water = false;
        if let Some(cell) = self.get_mut(location) {
            opened_water = !cell.is_open() && cell.cell_type.eq(&CellType::Water);
            cell.open();
        };
        let no_neighbouring_mines = self.count_neighbours(location).eq(&0);
        if opened_water && no_neighbouring_mines {
            location.neighbours().iter().for_each(|neighbour| {
                self.open(neighbour);
            });
        }
    }

    fn flag(&mut self, location: &Point2D) {
        if let Some(cell) = self.get_mut(location) {
            cell.flag();
        }
    }

    fn draw(&self, engine: &mut ConsoleEngine) {
        for x in 0 .. self.size().0 {
            for y in 0 .. self.size().1 {
                let location = Point2D(x, y);
                self.draw_cell(&location, engine);
            }
        }
    }

    fn draw_cell(&self, location: &Point2D, engine: &mut ConsoleEngine) {
        if let Some(cell) = self.get(location) {
            let pixel = match cell.state {
                CellState::Closed => pxl('?'),
                CellState::Flagged => pxl_fbg('F', Color::White, Color::DarkGreen),
                CellState::Opened => match cell.cell_type {
                    CellType::Mine => pxl_fbg('M', Color::Black, Color::DarkRed),
                    CellType::Water => match self.count_neighbours(location) {
                        0 => pxl(' '),
                        num => pxl_fg((num + 0x30) as char, color_for_number(num)),
                    }
                }
            };
            engine.set_pxl((location.0 * 2 + 1) as i32, location.1 as i32, pixel);
        }
    }
}

fn color_for_number(number: u8) -> Color {
    let colors = [Color::Cyan, Color::DarkCyan, Color::Yellow, Color::DarkYellow, Color::Magenta, Color::Red];
    colors.get(number as usize).cloned().unwrap_or(Color::White)
}

enum SystemEvent {
    Exit,
}

struct Game {
    field: Minefield,
    cursor: Point2D,
}

impl Game {
    fn with_minefield(field: Minefield) -> Self {
        Self {
            field,
            cursor: Point2D::default(),
        }
    }

    fn update(&mut self, engine: &ConsoleEngine) -> Option<SystemEvent> {
        if engine.is_key_pressed(KeyCode::Esc) {
            return Some(SystemEvent::Exit);
        }
        self.move_cursor(engine);
        if engine.is_key_pressed(KEY_OPEN) {
            self.field.open(&self.cursor);
        }
        if engine.is_key_pressed(KEY_FLAG) {
            self.field.flag(&self.cursor);
        }
        None
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

    fn draw(&self, engine: &mut ConsoleEngine) {
        self.field.draw(engine);
        engine.set_pxl((self.cursor.0 * 2) as i32, self.cursor.1 as i32, pxl('['));
        engine.set_pxl((self.cursor.0 * 2 + 2) as i32, self.cursor.1 as i32, pxl(']'));
    }
}

fn main() {
    let mut cells = Vec2D::sized(&Size2D(10, 10), Cell {
        cell_type: CellType::Water,
        state: CellState::Closed,
    });
    cells.get_mut(&Point2D(1, 1)).unwrap().cell_type = CellType::Mine;
    cells.get_mut(&Point2D(2, 1)).unwrap().cell_type = CellType::Mine;
    cells.get_mut(&Point2D(3, 1)).unwrap().cell_type = CellType::Mine;
    cells.get_mut(&Point2D(3, 2)).unwrap().cell_type = CellType::Mine;
    let minefield = Minefield::with_data(cells);
    let mut game = Game::with_minefield(minefield);

    let mut engine = ConsoleEngine::init_fill_require(42, 25, 10).unwrap();

    loop {
        engine.wait_frame();
        if let Some(event) = game.update(&engine) {
            match event {
                SystemEvent::Exit => break,
            }
        }
        engine.clear_screen();
        game.draw(&mut engine);
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
}