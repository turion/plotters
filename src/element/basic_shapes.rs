use super::{Drawable, PointCollection};
use crate::drawing::backend::{BackendCoord, DrawingBackend, DrawingErrorKind};
use crate::style::ShapeStyle;

/// An element of a single pixel
pub struct Pixel<Coord> {
    pos: Coord,
    style: ShapeStyle,
}

impl<Coord> Pixel<Coord> {
    pub fn new<P: Into<Coord>, S: Into<ShapeStyle>>(pos: P, style: S) -> Self {
        Self {
            pos: pos.into(),
            style: style.into(),
        }
    }
}

impl<'a, Coord: 'a> PointCollection<'a, Coord> for &'a Pixel<Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        std::iter::once(&self.pos)
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for Pixel<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            return backend.draw_pixel((x, y), &self.style.color);
        }
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_pixel_element() {
    use crate::prelude::*;
    let da = crate::create_mocked_drawing_area(300, 300, |m| {
        m.check_draw_pixel(|c, (x, y)| {
            assert_eq!(x, 150);
            assert_eq!(y, 152);
            assert_eq!(c, RED.to_rgba());
        });

        m.drop_check(|b| {
            assert_eq!(b.num_draw_pixel_call, 1);
            assert_eq!(b.draw_count, 1);
        });
    });
    da.draw(&Pixel::new((150, 152), &RED))
        .expect("Drawing Failure");
}

/// An element of a series of connected lines
pub struct Path<Coord> {
    points: Vec<Coord>,
    style: ShapeStyle,
}
impl<Coord> Path<Coord> {
    /// Create a new path
    /// - `points`: The iterator of the points
    /// - `style`: The shape style
    /// - returns the created element
    pub fn new<P: Into<Vec<Coord>>, S: Into<ShapeStyle>>(points: P, style: S) -> Self {
        Self {
            points: points.into(),
            style: style.into(),
        }
    }
}

impl<'a, Coord: 'a> PointCollection<'a, Coord> for &'a Path<Coord> {
    type Borrow = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        &self.points
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for Path<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        backend.draw_path(points, &self.style.color)
    }
}

#[cfg(test)]
#[test]
fn test_path_element() {
    use crate::prelude::*;
    let da = crate::create_mocked_drawing_area(300, 300, |m| {
        m.check_draw_path(|c, path| {
            assert_eq!(c, BLUE.to_rgba());
            assert_eq!(path, vec![(100, 101), (105, 107), (150, 157)]);
        });
        m.drop_check(|b| {
            assert_eq!(b.num_draw_path_call, 1);
            assert_eq!(b.draw_count, 1);
        });
    });
    da.draw(&Path::new(vec![(100, 101), (105, 107), (150, 157)], &BLUE))
        .expect("Drawing Failure");
}

/// A rectangle element
pub struct Rectangle<Coord> {
    points: [Coord; 2],
    style: ShapeStyle,
    margin: (u32, u32, u32, u32),
}

impl<Coord> Rectangle<Coord> {
    /// Create a new path
    /// - `points`: The left upper and right lower coner of the rectangle
    /// - `style`: The shape style
    /// - returns the created element
    pub fn new<S: Into<ShapeStyle>>(points: [Coord; 2], style: S) -> Self {
        Self {
            points,
            style: style.into(),
            margin: (0, 0, 0, 0),
        }
    }

    /// Set the margin of the rectangle
    /// - `t`: The top margin
    /// - `b`: The bottom margin
    /// - `l`: The left margin
    /// - `r`: The right margin
    pub fn set_margin(&mut self, t: u32, b: u32, l: u32, r: u32) -> &mut Self {
        self.margin = (t, b, l, r);
        self
    }
}

impl<'a, Coord: 'a> PointCollection<'a, Coord> for &'a Rectangle<Coord> {
    type Borrow = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        &self.points
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for Rectangle<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        match (points.next(), points.next()) {
            (Some(a), Some(b)) => {
                let (mut a, mut b) = ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)));
                a.1 += self.margin.0 as i32;
                b.1 -= self.margin.1 as i32;
                a.0 += self.margin.2 as i32;
                b.0 -= self.margin.3 as i32;
                backend.draw_rect(a, b, &self.style.color, self.style.filled)
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
#[test]
fn test_rect_element() {
    use crate::prelude::*;
    let da = crate::create_mocked_drawing_area(300, 300, |m| {
        m.check_draw_rect(|c, f, u, d| {
            assert_eq!(c, BLUE.to_rgba());
            assert_eq!(f, false);
            assert_eq!([u, d], [(100, 101), (105, 107)]);
        });
        m.drop_check(|b| {
            assert_eq!(b.num_draw_rect_call, 1);
            assert_eq!(b.draw_count, 1);
        });
    });
    da.draw(&Rectangle::new([(100, 101), (105, 107)], &BLUE))
        .expect("Drawing Failure");
}

/// A circle element
pub struct Circle<Coord> {
    center: Coord,
    size: u32,
    style: ShapeStyle,
}

impl<Coord> Circle<Coord> {
    /// Create a new circle element
    /// - `coord` The center of the circle
    /// - `size` The radius of the circle
    /// - `style` The style of the circle
    /// - Return: The newly created circle element
    pub fn new<S: Into<ShapeStyle>>(coord: Coord, size: u32, style: S) -> Self {
        Self {
            center: coord,
            size,
            style: style.into(),
        }
    }
}

impl<'a, Coord: 'a> PointCollection<'a, Coord> for &'a Circle<Coord> {
    type Borrow = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> std::iter::Once<&'a Coord> {
        std::iter::once(&self.center)
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for Circle<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        if let Some((x, y)) = points.next() {
            return backend.draw_circle((x, y), self.size, &self.style.color, self.style.filled);
        }
        Ok(())
    }
}

#[cfg(test)]
#[test]
fn test_circle_element() {
    use crate::prelude::*;
    let da = crate::create_mocked_drawing_area(300, 300, |m| {
        m.check_draw_circle(|c, f, s, r| {
            assert_eq!(c, BLUE.to_rgba());
            assert_eq!(f, false);
            assert_eq!(s, (150, 151));
            assert_eq!(r, 20);
        });
        m.drop_check(|b| {
            assert_eq!(b.num_draw_circle_call, 1);
            assert_eq!(b.draw_count, 1);
        });
    });
    da.draw(&Circle::new((150, 151), 20, &BLUE))
        .expect("Drawing Failure");
}

/// An element of a filled polygon
pub struct Polygon<Coord> {
    points: Vec<Coord>,
    style: ShapeStyle,
}
impl<Coord> Polygon<Coord> {
    /// Create a new polygon
    /// - `points`: The iterator of the points
    /// - `style`: The shape style
    /// - returns the created element
    pub fn new<P: Into<Vec<Coord>>, S: Into<ShapeStyle>>(points: P, style: S) -> Self {
        Self {
            points: points.into(),
            style: style.into(),
        }
    }
}

impl<'a, Coord: 'a> PointCollection<'a, Coord> for &'a Polygon<Coord> {
    type Borrow = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        &self.points
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for Polygon<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        points: I,
        backend: &mut DB,
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        backend.fill_polygon(points, &self.style.color)
    }
}
