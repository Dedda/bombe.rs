use crate::geom::{Point2D, Size2D};


pub struct Vec2D<T> {
    pub size: Size2D,
    data: Vec<Vec<T>>,
}

impl<T> Vec2D<T> {
    pub fn sized(size: &Size2D, default: T) -> Self where T: Copy {
        Self {
            size: size.clone(),
            data: vec![vec![default; size.1]; size.0],
        }
    }

    pub fn get(&self, point2d: &Point2D) -> Option<&T> {
        if !self.size.contains(point2d) {
            None
        } else {
            self.data.get(point2d.0)?.get(point2d.1)
        }
    }

    pub fn get_mut(&mut self, point2d: &Point2D) -> Option<&mut T> {
        if !self.size.contains(point2d) {
            None
        } else {
            self.data.get_mut(point2d.0)?.get_mut(point2d.1)
        }
    }

    pub fn all_locations(&self) -> Vec<Point2D> {
        (0..self.size.0)
            .flat_map(|x| (0..self.size.1)
                .map(move |y| Point2D(x, y)))
            .collect()
    }
}
