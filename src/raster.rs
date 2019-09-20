use crate::model::{Coordinate, absdelta};
use crate::model::PathSegment;

pub fn rasterize(p: PathSegment) -> impl Iterator<Item=Coordinate> {
    return match p {
        PathSegment::Line(a, b) => Line::new(a, b),
    };
}

pub struct Line {
    dx: i16,
    dy: i16,

    cur: Coordinate,
    end: Coordinate,

    err: i16,
}

impl Line {
    pub fn new(a: Coordinate, b: Coordinate) -> Self {
        let dx = (b.x - a.x).abs();
        let dy = -(b.y - a.y).abs();

        let err = dx + dy;

        return Self {
            dx,
            dy,
            cur: a,
            end: b,
            err,
        };
    }
}

impl Iterator for Line {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.cur;

        let e2 = 2 * self.err;

        if e2 >= self.dy {
            if self.cur.x == self.end.x { return None; }

            self.err += self.dy;

            if self.cur.x < self.end.x {
                self.cur.x += 1;
            } else {
                self.cur.x -= 1;
            }
        }

        if e2 <= self.dx {
            if self.cur.y == self.end.y { return None; }

            self.err += self.dx;

            if self.cur.y < self.end.y {
                self.cur.y += 1;
            } else {
                self.cur.y -= 1;
            }
        }

        return Some(ret);
    }
}

//    plotLine(int x0, int y0, int x1, int y1)
//       dx =  abs(x1-x0);
//       dy = -abs(y1-y0);
//       sx = x0<x1 ? 1 : -1;
//       sy = y0<y1 ? 1 : -1;
//       err = dx+dy;  /* error value e_xy */
//       while (true)   /* loop */
//           if (x0==x1 && y0==y1) break;
//           e2 = 2*err;
//           if (e2 >= dy)
//               err += dy; /* e_xy+e_x > 0 */
//               x0 += sx;
//           end if
//           if (e2 <= dx) /* e_xy+e_y < 0 */
//               err += dx;
//               y0 += sy;
//           end if
//       end while

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wp_example() {
        let res = Line::new(Coordinate::from((0, 1)), Coordinate::from((6, 4)))
            .map(|c| (c.x, c.y))
            .collect::<Vec<_>>();
        assert_eq!(res, [(0, 1), (1, 1), (2, 2), (3, 2), (4, 3), (5, 3)])
    }

//    #[test]
//    fn test_inverse_wp() {
//        let bi = Bresenham::new((6, 4), (0, 1));
//        let res: Vec<_> = bi.collect();
//
//        assert_eq!(res, [(6, 4), (5, 4), (4, 3), (3, 3), (2, 2), (1, 2)])
//    }
//
//    #[test]
//    fn test_straight_hline() {
//        let bi = Bresenham::new((2, 3), (5, 3));
//        let res: Vec<_> = bi.collect();
//
//        assert_eq!(res, [(2, 3), (3, 3), (4, 3)]);
//    }
//
//    #[test]
//    fn test_straight_vline() {
//        let bi = Bresenham::new((2, 3), (2, 6));
//        let res: Vec<_> = bi.collect();
//
//        assert_eq!(res, [(2, 3), (2, 4), (2, 5)]);
//    }
}