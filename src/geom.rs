use std::cmp::min;
use std::ops::{Add, Sub};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct Size2D(pub usize, pub usize);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Point2D(pub usize, pub usize);


impl Size2D {
    pub fn contains(&self, point: &Point2D) -> bool {
        point.0 < self.0 && point.1 < self.1
    }
}

impl Point2D {
    pub fn clip_excl(&mut self, size2d: &Size2D) {
        self.0 = min(self.0, size2d.0.saturating_sub(1));
        self.1 = min(self.1, size2d.1.saturating_sub(1));
    }

    pub fn neighbours(&self) -> Vec<Point2D> {
        (0..=2).cartesian_product(0..=2)
            .map(|(x, y)| Point2D(x, y))
            .map(|offset| offset + self)
            .filter(|point| point.0 > 0 && point.1 > 0)
            .map(|point| Point2D(point.0 - 1, point.1 - 1))
            .filter(|point| !point.eq(self))
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