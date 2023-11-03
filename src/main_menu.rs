use console_engine::{Color, ConsoleEngine, KeyCode};
use console_engine::pixel::pxl;
use console_engine::screen::Screen;
use rand::thread_rng;
use crate::game::{Game, RandomMineFieldGenerator};
use crate::geom::Size2D;
use crate::state::{GameState, SystemEvent};

const MAIN_MENU_HEADER: &str = include_str!("../assets/main_menu_header.txt");
const RAINBOW_COLORS: [Color; 6] = [Color::Blue, Color::Cyan, Color::Green, Color::Yellow, Color::Red, Color::Magenta];

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

pub struct MainMenu {
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
