use std::ops;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Coordinate {
    pub x: i16,
    pub y: i16,
}

impl Coordinate {
    pub fn zero() -> Self {
        return Self { x: 0, y: 0 };
    }
}

impl From<(i16, i16)> for Coordinate {
    fn from(tuple: (i16, i16)) -> Self {
        return Self { x: tuple.0, y: tuple.1 };
    }
}

impl ops::Add for Coordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        return Self {
            x: self.x.overflowing_add(rhs.x).0,
            y: self.y.overflowing_add(rhs.y).0,
        };
    }
}

impl ops::Mul<u16> for Coordinate {
    type Output = Self;

    fn mul(self, rhs: u16) -> Self::Output {
        return Self {
            x: self.x * rhs as i16,
            y: self.y * rhs as i16,
        };
    }
}

impl ops::Div<u16> for Coordinate {
    type Output = Self;

    fn div(self, rhs: u16) -> Self::Output {
        return Self {
            x: self.x / rhs as i16,
            y: self.y / rhs as i16,
        };
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PathSegment {
    Line(Coordinate, Coordinate),

//    QuadraticCurve {
//
//    },
//
//    CubicCurve {
//
//    },
//
//    EllipticalArc {
//
//    },
}

impl PathSegment {
    pub fn line(a: Coordinate, b: Coordinate) -> Self {
        return Self::Line(a, b);
    }
}

impl ops::Mul<u16> for PathSegment {
    type Output = Self;

    fn mul(self, rhs: u16) -> Self::Output {
        return match self {
            Self::Line(a, b) => Self::Line(a * rhs, b * rhs)
        };
    }
}

impl ops::Div<u16> for PathSegment {
    type Output = Self;

    fn div(self, rhs: u16) -> Self::Output {
        return match self {
            Self::Line(a, b) => Self::Line(a / rhs, b / rhs)
        };
    }
}

pub fn absdelta(a: i16, b: i16) -> u16 {
    return if a < b { (b - a) as u16 } else { (a - b) as u16 };
}

pub fn dedup_segments(segments: &[PathSegment], fuzziness: u16) -> Vec<PathSegment> {
    let mut result = Vec::new();

    fn delta(a: &Coordinate, b: &Coordinate) -> u16 {
        let dx = absdelta(a.x, b.x);
        let dy = absdelta(a.y, b.y);

        return u16::max(dx, dy);
    }

    for i in 0..segments.len() {
        let current = &segments[i];

        let duplicate = segments[0..i].iter()
            .any(|other| match (current, other) {
                (PathSegment::Line(a1, b1), PathSegment::Line(a2, b2)) =>
                    (delta(a1, a2) <= fuzziness && delta(b1, b2) <= fuzziness) || (delta(a1, b2) <= fuzziness && delta(a2, b1) <= fuzziness),
                _ => false
            });
        if !duplicate {
            result.push(current.clone());
        }
    }

    return result;
}
