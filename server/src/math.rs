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

/// Simple pseudo-random number generator (LCG) based on seed
#[derive(Debug, Clone)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E3779B97F4A7C15),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    pub fn next_i64_range(&mut self, min: i64, max: i64) -> i64 {
        let range = (max - min).unsigned_abs() + 1;
        min + (self.next_u64() % range) as i64
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() % 1_000_000) as f32 / 1_000_000.0
    }

    pub fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

/// Wraps a 2D coordinate inside the given bounds [min_x..max_x] and [min_y..max_y].
pub fn wrap_position(mut pos: Vec2, min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Vec2 {
    let span_x = max_x - min_x;
    let span_y = max_y - min_y;

    if span_x > 0.0 {
        while pos.x < min_x {
            pos.x += span_x;
        }
        while pos.x > max_x {
            pos.x -= span_x;
        }
    }

    if span_y > 0.0 {
        while pos.y < min_y {
            pos.y += span_y;
        }
        while pos.y > max_y {
            pos.y -= span_y;
        }
    }

    pos
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

    #[test]
    fn test_simple_rng() {
        let mut rng = SimpleRng::new(123456789);
        let v1 = rng.next_f32();
        let v2 = rng.next_f32();
        assert_ne!(v1, v2);
        assert!((0.0..=1.0).contains(&v1));

        let speed = rng.next_f32_range(10.0, 20.0);
        assert!((10.0..=20.0).contains(&speed));
    }

    #[test]
    fn test_wrap_position() {
        let min_x = -50.0;
        let max_x = 3250.0;
        let min_y = -50.0;
        let max_y = 2050.0;

        // Inside bounds remains unchanged
        let p1 = Vec2::new(100.0, 200.0);
        assert_eq!(wrap_position(p1, min_x, max_x, min_y, max_y), p1);

        // Past right edge wraps to left
        let p2 = Vec2::new(3255.0, 500.0);
        let wrapped2 = wrap_position(p2, min_x, max_x, min_y, max_y);
        assert!((wrapped2.x - (-45.0)).abs() < 1e-4);
        assert_eq!(wrapped2.y, 500.0);

        // Past bottom edge wraps to top
        let p3 = Vec2::new(500.0, 2055.0);
        let wrapped3 = wrap_position(p3, min_x, max_x, min_y, max_y);
        assert_eq!(wrapped3.x, 500.0);
        assert!((wrapped3.y - (-45.0)).abs() < 1e-4);
    }
}
