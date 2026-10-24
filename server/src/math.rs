use spacetimedb::SpacetimeType;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

/// A 2D vector for continuous space calculations, storable in SpacetimeDB tables.
#[derive(SpacetimeType, Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Squared length of the vector (avoids square root calculation).
    pub fn sqr_magnitude(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Euclidean length of the vector.
    pub fn magnitude(&self) -> f32 {
        self.sqr_magnitude().sqrt()
    }

    /// Normalized unit vector (returns ZERO if magnitude is 0).
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag > 1e-6 { *self / mag } else { Self::ZERO }
    }

    /// Squared distance to another point.
    pub fn sqr_distance(&self, other: &Vec2) -> f32 {
        (*self - *other).sqr_magnitude()
    }

    /// Euclidean distance to another point.
    pub fn distance(&self, other: &Vec2) -> f32 {
        (*self - *other).magnitude()
    }

    /// Normalized direction unit vector pointing towards target.
    pub fn direction_to(&self, target: &Vec2) -> Self {
        (*target - *self).normalized()
    }

    /// Angle / heading in radians [-PI, PI].
    pub fn heading(&self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Constructs a vector from an angle in radians and length.
    pub fn from_heading(heading_rad: f32, magnitude: f32) -> Self {
        Self {
            x: heading_rad.cos() * magnitude,
            y: heading_rad.sin() * magnitude,
        }
    }

    /// Clamps the magnitude to a maximum value.
    pub fn clamp_magnitude(&self, max_mag: f32) -> Self {
        let sqr_mag = self.sqr_magnitude();
        if sqr_mag > max_mag * max_mag && sqr_mag > 0.0 {
            self.normalized() * max_mag
        } else {
            *self
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Vec2> for Vec2 {
    type Output = Self;
    fn add(self, rhs: &Vec2) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<&Vec2> for Vec2 {
    type Output = Self;
    fn sub(self, rhs: &Vec2) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        if rhs.abs() > 1e-6 {
            Self {
                x: self.x / rhs,
                y: self.y / rhs,
            }
        } else {
            Self::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec2_operations() {
        let a = Vec2::new(3.0, 4.0);
        assert!((a.magnitude() - 5.0).abs() < 1e-5);
        assert!((a.sqr_magnitude() - 25.0).abs() < 1e-5);

        let norm = a.normalized();
        assert!((norm.magnitude() - 1.0).abs() < 1e-5);

        let b = Vec2::new(0.0, 0.0);
        assert!((a.distance(&b) - 5.0).abs() < 1e-5);

        let sum = a + Vec2::new(1.0, -2.0);
        assert_eq!(sum, Vec2::new(4.0, 2.0));

        let clamped = a.clamp_magnitude(2.5);
        assert!((clamped.magnitude() - 2.5).abs() < 1e-5);
    }
}
