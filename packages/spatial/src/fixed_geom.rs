use std::{ops, cmp::Ordering};
use cosmwasm_std::{StdResult, StdError};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use substrate_fixed::{types::I32F32};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FixedPoint2D {
    pub x: I32F32,
    pub y: I32F32,
}

impl FixedPoint2D {
    pub fn as_vector_2d(&self) -> FixedVector2D {
        FixedVector2D { x: self.x, y: self.y }
    }

    pub fn into_stored(&self) -> StoredFixedPoint2D {
        StoredFixedPoint2D { 
            x: self.x.to_be_bytes().to_vec(), 
            y: self.y.to_be_bytes().to_vec() 
        }
    }
}

impl ops::Sub<FixedPoint2D> for FixedPoint2D {
    type Output = FixedVector2D;
    fn sub(self, rhs: FixedPoint2D) -> Self::Output {
        FixedVector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StoredFixedPoint2D {
    pub x: Vec<u8>,
    pub y: Vec<u8>,
}

impl StoredFixedPoint2D {
    pub fn into_humanized(&self) -> StdResult<FixedPoint2D> {
        let point = FixedPoint2D {
            x: I32F32::from_be_bytes(
                match self.x.as_slice().try_into() {
                    Ok(x_bytes) => x_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
            y: I32F32::from_be_bytes(
                match self.y.as_slice().try_into() {
                    Ok(y_bytes) => y_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
        };
        Ok(point)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedVector2D {
    pub x: I32F32,
    pub y: I32F32,
}

impl FixedVector2D {
    /// dot product
    pub fn dot(&self, other: &FixedVector2D) -> I32F32 {
        self.x * other.x + self.y * other.y
    }

    /// length squared of a vector
    pub fn len_squared(&self) -> I32F32 {
        self.dot(self)
    }

    pub fn as_point_2d(&self) -> FixedPoint2D {
        FixedPoint2D { x: self.x, y: self.y }
    }

    pub fn into_stored(&self) -> StoredFixedVector2D {
        StoredFixedVector2D { 
            x: self.x.to_be_bytes().to_vec(), 
            y: self.y.to_be_bytes().to_vec() 
        }
    }
}

impl ops::Add<FixedVector2D> for FixedVector2D {
    type Output = FixedVector2D;
    fn add(self, rhs: FixedVector2D) -> Self::Output {
        FixedVector2D { 
            x: self.x + rhs.x, 
            y: self.y + rhs.y
        }
    }
}

impl ops::Sub<FixedVector2D> for FixedVector2D {
    type Output = FixedVector2D;
    fn sub(self, rhs: FixedVector2D) -> Self::Output {
        FixedVector2D {
            x: self.x - rhs.x,
            y: self.y - rhs.y
        }
    }
}

impl ops::Mul<I32F32> for FixedVector2D {
    type Output = FixedVector2D;
    fn mul(self, rhs: I32F32) -> Self::Output {
        FixedVector2D {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StoredFixedVector2D {
    pub x: Vec<u8>,
    pub y: Vec<u8>,
}

impl StoredFixedVector2D {
    pub fn into_humanized(&self) -> StdResult<FixedVector2D> {
        let vector = FixedVector2D {
            x: I32F32::from_be_bytes(
                match self.x.as_slice().try_into() {
                    Ok(x_bytes) => x_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
            y: I32F32::from_be_bytes(
                match self.y.as_slice().try_into() {
                    Ok(y_bytes) => y_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
        };
        Ok(vector)
    }
}

/// Twice the area of the triangle abc
pub fn signed_area(a: FixedPoint2D, b: FixedPoint2D, c: FixedPoint2D) -> I32F32 {
    let p = b - a.clone();
    let q = c - a;
    p.x * q.y - q.x * p.y
}

/// Returns: 
///   Some(true) if triangle abc is counterclockwise
///   Some(false) if triangle abc is clockwise
///   None if colinear
pub fn is_counterclockwise(a: &FixedPoint2D, b: &FixedPoint2D, c: &FixedPoint2D) -> Option<bool> {
    let area = signed_area(a.clone(), b.clone(), c.clone());
    if area > 0 { Some(true) }
    else if area < 0 { Some(false) }
    else { None }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedLineSegment2D {
    pub endpoints: (FixedPoint2D, FixedPoint2D),
}

impl FixedLineSegment2D {
    pub fn new(endpoint1: FixedPoint2D, endpoint2: FixedPoint2D) -> StdResult<FixedLineSegment2D> {
        if endpoint1 == endpoint2 {
            return Err(StdError::generic_err("Invalid: endpoints cannot be the same"));
        }
        Ok(FixedLineSegment2D {
            endpoints: (endpoint1, endpoint2)
        })
    }

    pub fn intersects(&self, other: &FixedLineSegment2D) -> bool {
        (is_counterclockwise(&self.endpoints.0, &other.endpoints.1, &self.endpoints.0) != 
         is_counterclockwise(&self.endpoints.0, &other.endpoints.0, &self.endpoints.1)) && 
        (is_counterclockwise(&other.endpoints.0, &self.endpoints.0, &other.endpoints.1) != 
         is_counterclockwise(&other.endpoints.0, &self.endpoints.1, &other.endpoints.1))
    }

    pub fn into_stored(&self) -> StoredFixedLineSegment2D {
        StoredFixedLineSegment2D { 
            endpoints: (
                self.endpoints.0.into_stored(),
                self.endpoints.1.into_stored(),
            )
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StoredFixedLineSegment2D {
    pub endpoints: (StoredFixedPoint2D, StoredFixedPoint2D)
}

impl StoredFixedLineSegment2D {
    pub fn into_humanized(&self) -> StdResult<FixedLineSegment2D> {
        Ok(FixedLineSegment2D { 
            endpoints: (
                self.endpoints.0.into_humanized()?,
                self.endpoints.1.into_humanized()?,
            ) 
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedBBox2D {
    lower_left: FixedPoint2D,
    upper_right: FixedPoint2D,
}

impl FixedBBox2D {
    pub fn contains(&self, point: &FixedPoint2D) -> bool {
        point.x >= self.lower_left.x &&
        point.x <= self.upper_right.x &&
        point.y >= self.lower_left.y &&
        point.y <= self.upper_right.y
    }

    pub fn into_stored(&self) -> StoredFixedBBox2D {
        StoredFixedBBox2D { 
            lower_left: self.lower_left.into_stored(),
            upper_right: self.upper_right.into_stored(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StoredFixedBBox2D {
    lower_left: StoredFixedPoint2D,
    upper_right: StoredFixedPoint2D,
}

impl StoredFixedBBox2D {
    pub fn into_humanized(&self) -> StdResult<FixedBBox2D> {
        Ok(FixedBBox2D { 
            lower_left: self.lower_left.into_humanized()?,
            upper_right: self.upper_right.into_humanized()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FixedPolygon2D {
    vertices: Vec<FixedPoint2D>,
    anchor: FixedPoint2D,
    bbox: FixedBBox2D,
}

impl FixedPolygon2D {
    pub fn new(points: Vec<FixedPoint2D>) -> StdResult<FixedPolygon2D> {
        let length = points.len();
        if length < 3 {
            return Err(StdError::generic_err("Polygon must have at least 3 vertices"));
        }
        if points[0] != points[length-1] {
            return Err(StdError::generic_err("First and last point vector must be the same"))
        }

        // calculate bounding box and anchor
        let mut anchor = points[0];
        let mut min_x: I32F32 = I32F32::max_value();
        let mut min_y: I32F32 = I32F32::max_value();
        let mut max_x: I32F32 = I32F32::min_value();
        let mut max_y: I32F32 = I32F32::min_value();
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
        let bbox = FixedBBox2D {
            lower_left: FixedPoint2D { x: min_x, y: min_y },
            upper_right: FixedPoint2D { x: max_x, y: max_y }
        };
        Ok(FixedPolygon2D { vertices: points, anchor, bbox } )
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn contains(&self, point: &FixedPoint2D) -> bool {
        if !self.bbox.contains(point) {
            return false;
        }

        let point_to_right = FixedPoint2D {
            x: self.bbox.upper_right.x + I32F32::from(1),
            y: point.y
        };
        let test_segment = FixedLineSegment2D {
            endpoints: (*point, point_to_right)
        };

        let mut intersections: u32 = 0;
        for i in 0..self.len() - 1 {
            let edge = FixedLineSegment2D {
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

    fn ccw_cmp(anchor: &FixedPoint2D, a: &FixedPoint2D, b: &FixedPoint2D) -> Ordering {
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

    pub fn as_counterclockwise_points(&self) -> Vec<FixedPoint2D> {
        let mut points = self.vertices.clone();
        points.sort_unstable_by(|a, b| FixedPolygon2D::ccw_cmp(&self.anchor, a, b));
        points
    }

    pub fn into_stored(&self) -> StoredFixedPolygon2D {
        StoredFixedPolygon2D { 
            vertices: self.vertices.iter().map(|v| v.into_stored()).collect(),
            anchor: self.anchor.into_stored(),
            bbox: self.bbox.into_stored(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StoredFixedPolygon2D {
    vertices: Vec<StoredFixedPoint2D>,
    anchor: StoredFixedPoint2D,
    bbox: StoredFixedBBox2D,
}

impl StoredFixedPolygon2D {
    pub fn into_humanized(&self) -> StdResult<FixedPolygon2D> {
        Ok(FixedPolygon2D { 
            vertices: self.vertices
                .iter()
                .map(|v| v.into_humanized().unwrap())
                .collect(),
            anchor: self.anchor.into_humanized()?,
            bbox: self.bbox.into_humanized()?,
        })
    }
}