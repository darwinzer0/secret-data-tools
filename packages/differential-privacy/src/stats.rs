use cosmwasm_std::{StdResult, StdError};
use rand_chacha::ChaChaRng;
use substrate_fixed::types::{I64F64, I32F32};
use crate::laplace;

pub const DP_COLLECTING_DATA: u8 = 0;
pub const DP_CALCULATING_STATS: u8 = 1;

/// Struct for a data set collected one observation at a time.
/// Calculates a running sum, can be used for COUNT and AVERAGE queries
/// in O(1) time.
#[derive(Clone, Debug, PartialEq)]
pub struct RunningStats {
    pub count: u32,
    pub sum: I64F64,
    pub upper_bound: I32F32,
    pub lower_bound: I32F32,
    pub epsilon: I32F32,
    pub sensitivity_for_average: Option<I32F32>,
    pub privacy_budget: I32F32,
    pub status: u8,
}

impl RunningStats {
    pub fn new(epsilon: I32F32, sensitivity_for_average: Option<I32F32>, privacy_budget: I32F32) -> RunningStats {
        RunningStats {
            count: 0,
            sum: I64F64::from(0_i32),
            upper_bound: I32F32::from(0_i32),
            lower_bound: I32F32::from(0_i32),
            epsilon,
            sensitivity_for_average,
            privacy_budget,
            status: DP_COLLECTING_DATA,
        }
    }

    pub fn into_stored(self) -> StoredRunningStats {
        StoredRunningStats {
            count: self.count,
            sum: self.sum.to_be_bytes().to_vec(),
            upper_bound: self.upper_bound.to_be_bytes().to_vec(),
            lower_bound: self.lower_bound.to_be_bytes().to_vec(),
            epsilon: self.epsilon.to_be_bytes().to_vec(),
            sensitivity_for_average: self.sensitivity_for_average
                .map(|s| s.to_be_bytes().to_vec()),
            privacy_budget: self.privacy_budget.to_be_bytes().to_vec(),
            status: self.status,
        }
    }

    pub fn add_observation(mut self, x: I32F32) -> StdResult<()> {
        if self.status != DP_CALCULATING_STATS {
            return Err(StdError::generic_err("Cannot add more observations") );
        }

        self.count = self.count.checked_add(1).ok_or(
            StdError::generic_err("Count overflow")
        )?;
        self.sum = self.sum.checked_add(I64F64::from_num(x)).ok_or(
            StdError::generic_err("Sum overflow")
        )?;
        if self.upper_bound < x {
            self.upper_bound = x;
        }
        if self.lower_bound > x {
            self.lower_bound = x;
        }
        Ok(())
    }

    pub fn dp_count(mut self, rng: &mut ChaChaRng) -> StdResult<I32F32> {
        if self.status != DP_CALCULATING_STATS {
            return Err(StdError::generic_err("Cannot calculate differentally private count") );
        }

        if self.count == 0 {
            return Err(StdError::generic_err("No data to count"));
        }

        if self.privacy_budget < self.epsilon { // privacy cost of COUNT = 1 * epsilon
            return Err(StdError::generic_err("Privacy budget exhausted"));
        }

        // sensitivity is always 1 for COUNT queries
        let sensitivity = I32F32::from_num(1_u32);
        
        // calculate a fuzzy count
        let scale = sensitivity / self.epsilon;
        let noise = laplace(rng, scale);
        let fuzzy_count = noise + I32F32::from_num(self.count);

        // update the remaining privacy budget
        self.privacy_budget = self.privacy_budget - self.epsilon;

        Ok(fuzzy_count)
    }

    pub fn dp_average(mut self, rng: &mut ChaChaRng) -> StdResult<I32F32> {
        if self.status != DP_CALCULATING_STATS {
            return Err(StdError::generic_err("Cannot calculate differentially private average") );
        }

        if self.count == 0 {
            return Err(StdError::generic_err("No data to count"));
        }

        // sequential queries for sum + count
        let privacy_cost: I32F32 = 2 * self.epsilon;
        if self.privacy_budget < privacy_cost { 
            return Err(StdError::generic_err("Privacy budget exhausted"));
        }

        let sensitivity: I32F32;
        if let Some(sensitivity_for_average) = self.sensitivity_for_average {
            sensitivity = sensitivity_for_average;
        } else {
            // using a bounded sensitivity for sum
            sensitivity = self.upper_bound - self.lower_bound;
        }

        let scale = sensitivity / self.epsilon;
    
        let sum_noise = laplace(rng, scale);
        let fuzzy_sum = self.sum + I64F64::from_num(sum_noise);
    
        // calculate fuzzy count
        let sensitivity = I32F32::from_num(1_u32);
        let scale = sensitivity / self.epsilon;
    
        let noise = laplace(rng, scale);
        let real_count = I32F32::from_num(self.count);
        let fuzzy_count = I64F64::from_num(real_count + noise);
    
        let fuzzy_average = I32F32::from_num(fuzzy_sum / fuzzy_count);

        // update the remaining privacy budget
        self.privacy_budget = self.privacy_budget - privacy_cost;

        Ok(fuzzy_average)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredRunningStats {
    pub count: u32,
    pub sum: Vec<u8>,
    pub upper_bound: Vec<u8>,
    pub lower_bound: Vec<u8>,
    pub epsilon: Vec<u8>,
    pub sensitivity_for_average: Option<Vec<u8>>,
    pub privacy_budget: Vec<u8>,
    pub status: u8,
}

impl StoredRunningStats {
    pub fn into_humanized(self) -> StdResult<RunningStats> {
        let stats = RunningStats {
            count: self.count,
            sum: I64F64::from_be_bytes(
                match self.sum.as_slice().try_into() {
                    Ok(sum_bytes) => sum_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
            upper_bound: I32F32::from_be_bytes(
                match self.upper_bound.as_slice().try_into() {
                    Ok(upper_bound_bytes) => upper_bound_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
            lower_bound: I32F32::from_be_bytes(
                match self.lower_bound.as_slice().try_into() {
                    Ok(lower_bound_bytes) => lower_bound_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
            epsilon: I32F32::from_be_bytes(
                match self.epsilon.as_slice().try_into() {
                    Ok(epsilon_bytes) => epsilon_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
            sensitivity_for_average: match self.sensitivity_for_average {
                Some(s) => {
                    let bytes_result: Result<[u8; 8],_> = s.as_slice().try_into();
                    match bytes_result {
                        Ok(sensitivity_bytes) => Some(I32F32::from_be_bytes(sensitivity_bytes)),
                        Err(err) => { 
                            return Err(StdError::generic_err(format!("{:?}", err))) 
                        }
                    }
                },
                None => None,
            },
            privacy_budget: I32F32::from_be_bytes(
                match self.privacy_budget.as_slice().try_into() {
                    Ok(privacy_budget_bytes) => privacy_budget_bytes,
                    Err(err) => { 
                        return Err(StdError::generic_err(format!("{:?}", err))) 
                    },
                }
            ),
            status: self.status,
        };
        Ok(stats)
    }
}