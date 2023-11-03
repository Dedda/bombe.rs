use console_engine::ConsoleEngine;
use crate::main_menu::MainMenu;
use crate::state::{GameState, SystemEvent};

mod collections;
mod game;
mod geom;
mod main_menu;
mod state;

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

    mod size2d {
        use crate::geom::{Point2D, Size2D};

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
        use crate::geom::{Point2D, Size2D};

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
            let point = Point2D::default();
            assert_eq!(3, point.neighbours().len());
        }
    }

    mod vec2d {
        use crate::collections::Vec2D;
        use crate::geom::{Point2D, Size2D};

        #[test]
        fn get_value_in_single_cell_vec2d() {
            let v = Vec2D::sized(&Size2D(1, 1), 5);
            assert_eq!(&5, v.get(&Point2D(0, 0)).unwrap());
        }
    }

}
