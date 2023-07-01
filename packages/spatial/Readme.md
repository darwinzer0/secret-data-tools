# Secret Data Tools - Spatial

This package includes data structures and algorithms for working with spatial data in Secret Contracts. It implements two sets of geometry types, one for working with geometry on an integer grid and another implemented with fixed point arithmetic. 

## Integer geometry types

`IntegerPoint2D` is a two-dimensional point with `x` and `y` as `i64` types.

`IntegerVector2D` is a two-dimensional vector with `x` and `y` as `i64` types.

`IntegerLineSegment2D` is a two-dimensional line segment built using two `IntegerPoint2D`s.

`IntegerPolygon2D` is a polygon built using a set of `IntegerPoint2D`s.

## Fixed-point geometry types

`FixedPoint2D` is a two-dimensional point with `x` and `y` as 64-bit fixed-point numbers with 32 integer bits and 32 fractional bits.

`FixedVector2D` is a two-dimensional vector with `x` and `y` as 64-bit fixed-point numbers with 32 integer bits and 32 fractional bits.

`FixedLineSegment2D` is a two-dimensional line segment built using two `FixedPoint2D`s.

`FixedPolygon2D` is a polygon built using a set of `FixedPoint2D`s.
