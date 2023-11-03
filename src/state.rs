use console_engine::ConsoleEngine;
use console_engine::screen::Screen;

pub enum SystemEvent {
    ChangeState(Box<dyn GameState>),
    Exit,
}

pub trait GameState {
    fn update(&mut self, engine: &ConsoleEngine) -> Option<SystemEvent>;

    fn draw(&self, screen: &mut Screen);
}
