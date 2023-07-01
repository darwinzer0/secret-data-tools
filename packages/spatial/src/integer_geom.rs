use std::{ops, cmp::Ordering};
use cosmwasm_std::{StdResult, StdError};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IntegerPoint2D {
    pub x: i64,
    pub y: i64,
}

impl IntegerPoint2D {
    pub fn as_vector_2d(&self) -> IntegerVector2D {
        IntegerVector2D { x: self.x, y: self.y }
    }
}

impl ops::Sub<IntegerPoint2D> for IntegerPoint2D {
    type Output = IntegerVector2D;
    fn sub(self, rhs: IntegerPoint2D) -> Self::Output {
        IntegerVector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IntegerVector2D {
    pub x: i64,
    pub y: i64,
}

impl IntegerVector2D {
    /// dot product
    pub fn dot(&self, other: &IntegerVector2D) -> i64 {
        self.x * other.x + self.y * other.y
    }

    /// length squared of a vector
    pub fn len_squared(&self) -> i64 {
        self.dot(self)
    }

    pub fn as_point_2d(&self) -> IntegerPoint2D {
        IntegerPoint2D { x: self.x, y: self.y }
    }
}

impl ops::Add<IntegerVector2D> for IntegerVector2D {
    type Output = IntegerVector2D;
    fn add(self, rhs: IntegerVector2D) -> Self::Output {
        IntegerVector2D { 
            x: self.x + rhs.x, 
            y: self.y + rhs.y
        }
    }
}

impl ops::Sub<IntegerVector2D> for IntegerVector2D {
    type Output = IntegerVector2D;
    fn sub(self, rhs: IntegerVector2D) -> Self::Output {
        IntegerVector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl ops::Mul<i64> for IntegerVector2D {
    type Output = IntegerVector2D;
    fn mul(self, rhs: i64) -> Self::Output {
        IntegerVector2D {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

/// Twice the area of the triangle abc
pub fn signed_area(a: IntegerPoint2D, b: IntegerPoint2D, c: IntegerPoint2D) -> i64 {
    let p = b - a.clone();
    let q = c - a;
    p.x * q.y - q.x * p.y
}

/// Returns: 
///   Some(true) if triangle abc is counterclockwise
///   Some(false) if triangle abc is clockwise
///   None if colinear
pub fn is_counterclockwise(a: &IntegerPoint2D, b: &IntegerPoint2D, c: &IntegerPoint2D) -> Option<bool> {
    let area = signed_area(a.clone(), b.clone(), c.clone());
    if area > 0 { Some(true) }
    else if area < 0 { Some(false) }
    else { None }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IntegerLineSegment2D {
    pub endpoints: (IntegerPoint2D, IntegerPoint2D),
}

impl IntegerLineSegment2D {
    pub fn new(endpoint1: IntegerPoint2D, endpoint2: IntegerPoint2D) -> StdResult<IntegerLineSegment2D> {
        if endpoint1 == endpoint2 {
            return Err(StdError::generic_err("Invalid: endpoints cannot be the same"));
        }
        Ok(Self {
            endpoints: (endpoint1, endpoint2)
        })
    }

    pub fn intersects(&self, other: &IntegerLineSegment2D) -> bool {
        (is_counterclockwise(&self.endpoints.0, &other.endpoints.1, &self.endpoints.0) != 
         is_counterclockwise(&self.endpoints.0, &other.endpoints.0, &self.endpoints.1)) && 
        (is_counterclockwise(&other.endpoints.0, &self.endpoints.0, &other.endpoints.1) != 
         is_counterclockwise(&other.endpoints.0, &self.endpoints.1, &other.endpoints.1))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct IntegerBBox {
    lower_left: IntegerPoint2D,
    upper_right: IntegerPoint2D,
}

impl IntegerBBox {
    pub fn contains(&self, point: &IntegerPoint2D) -> bool {
        point.x >= self.lower_left.x &&
        point.x <= self.upper_right.x &&
        point.y >= self.lower_left.y &&
        point.y <= self.upper_right.y
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IntegerPolygon2D {
    vertices: Vec<IntegerPoint2D>,
    anchor: IntegerPoint2D,
    bbox: IntegerBBox,
}

impl IntegerPolygon2D {
    pub fn new(points: Vec<IntegerPoint2D>) -> StdResult<IntegerPolygon2D> {
        let length = points.len();
        if length < 3 {
            return Err(StdError::generic_err("Polygon must have at least 3 vertices"));
        }
        if points[0] != points[length-1] {
            return Err(StdError::generic_err("First and last point vector must be the same"))
        }

        // calculate bounding box and anchor
        let mut anchor = points[0];
        let mut min_x: i64 = i64::MAX;
        let mut min_y: i64 = i64::MAX;
        let mut max_x: i64 = i64::MIN;
        let mut max_y: i64 = i64::MIN;
        points.iter().for_each(|pt| {
            if min_x > pt.x { min_x = pt.x }
            if max_x < pt.x { max_x = pt.x }
            if min_y > pt.y { min_y = pt.y }
            if max_y < pt.y { max_y = pt.y }

            if pt.y < anchor.y {
                anchor = pt.clone();
            } else if pt.y == anchor.y && pt.x < anchor.x {
                anchor = pt.clone();
            }
        });
        let bbox = IntegerBBox {
            lower_left: IntegerPoint2D { x: min_x, y: min_y },
            upper_right: IntegerPoint2D { x: max_x, y: max_y }
        };
        Ok(Self { vertices: points, anchor, bbox } )
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn contains(&self, point: &IntegerPoint2D) -> bool {
        if !self.bbox.contains(point) {
            return false;
        }

        let point_to_right = IntegerPoint2D {
            x: self.bbox.upper_right.x + 1,
            y: point.y
        };
        let test_segment = IntegerLineSegment2D {
            endpoints: (*point, point_to_right)
        };

        let mut intersections: u32 = 0;
        for i in 0..self.len() - 1 {
            let edge = IntegerLineSegment2D {
                endpoints: (self.vertices[i], self.vertices[i+1])
            };
            if edge.endpoints.0.y == edge.endpoints.1.y {
                // ignore horizontal edges
                continue;
            } else if edge.endpoints.0.y > edge.endpoints.1.y {
                // edge is directed downward
                // ignore intersection at start of edge
                if point.y == edge.endpoints.0.y {
                    continue;
                }
            } else {
                // edge is directed upward
                // ignore intersection at end of edge
                if point.y == edge.endpoints.1.y {
                    continue;
                }
            }
            if test_segment.intersects(&edge) {
                intersections += 1;
            }
        }
        intersections % 2 == 1
    }

    fn ccw_cmp(anchor: &IntegerPoint2D, a: &IntegerPoint2D, b: &IntegerPoint2D) -> Ordering {
        if a == anchor {
            return Ordering::Less;
        } else if b == anchor {
            return Ordering::Greater;
        }
        if let Some(ccw) = is_counterclockwise(anchor, a, b) {
            if ccw { Ordering::Less }
            else { Ordering::Greater }
        } else {
            Ordering::Equal
        }
    }

    pub fn as_counterclockwise_points(&self) -> Vec<IntegerPoint2D> {
        let mut points = self.vertices.clone();
        points.sort_unstable_by(|a, b| IntegerPolygon2D::ccw_cmp(&self.anchor, a, b));
        points
    }
}
