# Secret Data Tools - Differential Privacy

This package provides storage pattens for implementing differentially private statistics in secret contracts. 

## RunningStatsStore

`RunningStatsStore` is used to calculate fuzzy COUNT and AVERAGE statistics on a collected set of data observations represented as 64-bit fixed-point fractional numbers (32 integer bits and 32 fractional bits). With 32 integer bits, the values correspond roughly to `f32` in range. 