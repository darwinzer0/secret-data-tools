use std::{sync::Mutex, marker::PhantomData};
use cosmwasm_std::{Storage, StdResult, StdError};
use cosmwasm_storage::to_length_prefixed;
use rand_chacha::ChaChaRng;
use secret_toolkit::serialization::{Serde, Bincode2};
use serde::Serialize;
use substrate_fixed::{types::{I32F32, I64F64}, traits::Fixed};

use crate::laplace;

const COUNT_KEY: &[u8] = b"count";
const SUM_KEY: &[u8] = b"sum";
const UPPER_BOUND_KEY: &[u8] = b"ub";
const LOWER_BOUND_KEY: &[u8] = b"lb";
const EPSILON_KEY: &[u8] = b"epsilon";
const SENSITIVITY_FOR_AVG_KEY: &[u8] = b"a-sen";
const PRIVACY_BUDGET_KEY: &[u8] = b"budget";
const STATUS_KEY: &[u8] = b"status";

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RunningStatsStatus {
    CollectingData,
    CalculatingStats
}

pub const STATUS_COLLECTING_DATA: u8 = 0;
pub const STATUS_CALCULATING_STATS: u8 = 1;

pub struct RunningStatsStore<'a, Ser = Bincode2>
where
    Ser: Serde,
{
    /// prefix of the newly constructed Storage
    namespace: &'a [u8],
    /// needed if any suffixes were added to the original namespace.
    prefix: Option<Vec<u8>>,
    count: Mutex<Option<u32>>,
    sum: Mutex<Option<I64F64>>,
    upper_bound: Mutex<Option<I32F32>>,
    lower_bound: Mutex<Option<I32F32>>,
    epsilon: Mutex<Option<I32F32>>,
    avg_sensitivity: Mutex<Option<I32F32>>,
    privacy_budget: Mutex<Option<I32F32>>,
    status: Mutex<Option<RunningStatsStatus>>,
    serialization_type: PhantomData<Ser>,
}

impl<'a, Ser: Serde> RunningStatsStore<'a, Ser> {
    /// constructor
    pub const fn new(
        namespace: &'a [u8], 
        epsilon: Option<I32F32>, 
        avg_sensitivity: Option<I32F32>, 
        privacy_budget: Option<I32F32>
    ) -> Self {
        Self {
            namespace,
            prefix: None,
            count: Mutex::new(None),
            sum: Mutex::new(None),
            upper_bound: Mutex::new(None),
            lower_bound: Mutex::new(None),
            epsilon: Mutex::new(None),
            avg_sensitivity: Mutex::new(None),
            privacy_budget: Mutex::new(None),
            status: Mutex::new(None),
            serialization_type: PhantomData,
        }
    }

    /// This is used to produce a new RunningStatsStore. It can be used when you want to associate 
    /// a RunningStatsStore to multiple suffixes and you still want to define the RunningStatsStore 
    /// as a static constant
    pub fn add_suffix(&self, suffix: &[u8]) -> Self {
        let suffix = to_length_prefixed(suffix);
        let prefix = self.prefix.as_deref().unwrap_or(self.namespace);
        let prefix = [prefix, suffix.as_slice()].concat();
        Self {
            namespace: self.namespace,
            prefix: Some(prefix),
            count: Mutex::new(None),
            sum: Mutex::new(None),
            upper_bound: Mutex::new(None),
            lower_bound: Mutex::new(None),
            epsilon: Mutex::new(None),
            avg_sensitivity: Mutex::new(None),
            privacy_budget: Mutex::new(None),
            status: Mutex::new(None),
            serialization_type: self.serialization_type,
        }
    }
}

impl<'a, Ser: Serde> RunningStatsStore<'a, Ser> {
    fn as_slice(&self) -> &[u8] {
        if let Some(prefix) = &self.prefix {
            prefix
        } else {
            self.namespace
        }
    }

    fn get_count(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<u32> {
        let mut may_count = self.count.lock().unwrap();
        match *may_count {
            Some(count) => {
                Ok(count)
            }
            None => {
                let count_key = [self.as_slice(), COUNT_KEY].concat();
                if let Some(count_vec) = storage.get(&count_key) {
                    let count_bytes = count_vec
                        .as_slice()
                        .try_into()
                        .map_err(|err| StdError::parse_err("u32", err))?;
                    let count = u32::from_be_bytes(count_bytes);
                    *may_count = Some(count);
                    Ok(count)
                } else {
                    *may_count = Some(0);
                    Ok(0)
                }
            }
        }
    }

    pub fn is_empty(&self, storage: &dyn Storage) -> StdResult<bool> {
        Ok(self.get_count(storage)? == 0)
    }

    fn set_count(
        &self,
        storage: &mut dyn Storage,
        count: u32,
    ) {
        let count_key = [self.as_slice(), COUNT_KEY].concat();
        storage.set(&count_key, &count.to_be_bytes());
    }

    pub fn get_sum(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<I64F64> {
        let mut may_sum = self.sum.lock().unwrap();
        match *may_sum {
            Some(sum) => {
                Ok(sum)
            }
            None => {
                let sum_key = [self.as_slice(), SUM_KEY].concat();
                if let Some(sum_vec) = storage.get(&sum_key) {
                    let sum = I64F64::from_be_bytes(
                        match sum_vec.try_into() {
                            Ok(sum_bytes) => sum_bytes,
                            Err(err) => { 
                                return Err(StdError::generic_err(format!("{:?}", err))) 
                            },
                        }
                    );
                    *may_sum = Some(sum);
                    Ok(sum)
                } else {
                    // default upper bound
                    let sum = I64F64::from(0);
                    *may_sum = Some(sum);
                    Ok(sum)
                }
            }
        }
    }

    fn set_sum(
        &self,
        storage: &mut dyn Storage,
        sum: I64F64,
    ) {
        let sum_key = [self.as_slice(), SUM_KEY].concat();
        storage.set(&sum_key, &sum.to_be_bytes());
    }

    pub fn get_upper_bound(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<I32F32> {
        let mut may_upper_bound = self.upper_bound.lock().unwrap();
        match *may_upper_bound {
            Some(upper_bound) => {
                Ok(upper_bound)
            }
            None => {
                let upper_bound_key = [self.as_slice(), UPPER_BOUND_KEY].concat();
                if let Some(upper_bound_vec) = storage.get(&upper_bound_key) {
                    let upper_bound = I32F32::from_be_bytes(
                        match upper_bound_vec.try_into() {
                            Ok(upper_bound_bytes) => upper_bound_bytes,
                            Err(err) => { 
                                return Err(StdError::generic_err(format!("{:?}", err))) 
                            },
                        }
                    );
                    *may_upper_bound = Some(upper_bound);
                    Ok(upper_bound)
                } else {
                    // default upper bound
                    let upper_bound = I32F32::min_value();
                    *may_upper_bound = Some(upper_bound);
                    Ok(upper_bound)
                }
            }
        }
    }

    fn set_upper_bound(
        &self,
        storage: &mut dyn Storage,
        upper_bound: I32F32,
    ) {
        let upper_bound_key = [self.as_slice(), UPPER_BOUND_KEY].concat();
        storage.set(&upper_bound_key, &upper_bound.to_be_bytes());
    }

    pub fn get_lower_bound(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<I32F32> {
        let mut may_lower_bound = self.lower_bound.lock().unwrap();
        match *may_lower_bound {
            Some(lower_bound) => {
                Ok(lower_bound)
            }
            None => {
                let lower_bound_key = [self.as_slice(), LOWER_BOUND_KEY].concat();
                if let Some(lower_bound_vec) = storage.get(&lower_bound_key) {
                    let lower_bound = I32F32::from_be_bytes(
                        match lower_bound_vec.try_into() {
                            Ok(lower_bound_bytes) => lower_bound_bytes,
                            Err(err) => { 
                                return Err(StdError::generic_err(format!("{:?}", err))) 
                            },
                        }
                    );
                    *may_lower_bound = Some(lower_bound);
                    Ok(lower_bound)
                } else {
                    // default upper bound
                    let lower_bound = I32F32::max_value();
                    *may_lower_bound = Some(lower_bound);
                    Ok(lower_bound)
                }
            }
        }
    }

    fn set_lower_bound(
        &self,
        storage: &mut dyn Storage,
        lower_bound: I32F32,
    ) {
        let lower_bound_key = [self.as_slice(), LOWER_BOUND_KEY].concat();
        storage.set(&lower_bound_key, &lower_bound.to_be_bytes());
    }

    pub fn get_epsilon(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<I32F32> {
        let mut may_epsilon = self.epsilon.lock().unwrap();
        match *may_epsilon {
            Some(epsilon) => {
                Ok(epsilon)
            }
            None => {
                let epsilon_key = [self.as_slice(), EPSILON_KEY].concat();
                if let Some(epsilon_vec) = storage.get(&epsilon_key) {
                    let epsilon = I32F32::from_be_bytes(
                        match epsilon_vec.try_into() {
                            Ok(epsilon_bytes) => epsilon_bytes,
                            Err(err) => { 
                                return Err(StdError::generic_err(format!("{:?}", err))) 
                            },
                        }
                    );
                    *may_epsilon = Some(epsilon);
                    Ok(epsilon)
                } else {
                    // default epsilon = 1
                    let epsilon = I32F32::from(1);
                    *may_epsilon = Some(epsilon);
                    Ok(epsilon)
                }
            }
        }
    }

    /// Set the epsilon
    pub fn set_epsilon(
        &self, 
        storage: &mut dyn Storage, 
        epsilon: I32F32,
    ) {
        let epsilon_key = [self.as_slice(), EPSILON_KEY].concat();
        storage.set(&epsilon_key, &epsilon.to_be_bytes());

        //let mut may_epsilon = self.epsilon.lock().unwrap();
        //*may_epsilon = Some(epsilon);
    }

    pub fn get_avg_sensitivity(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<Option<I32F32>> {
        let mut may_sensitivity = self.avg_sensitivity.lock().unwrap();
        match *may_sensitivity {
            Some(sensitivity) => {
                Ok(Some(sensitivity))
            }
            None => {
                let sensitivity_key = [self.as_slice(), SENSITIVITY_FOR_AVG_KEY].concat();
                if let Some(sensitivity_vec) = storage.get(&sensitivity_key) {
                    let sensitivity = I32F32::from_be_bytes(
                        match sensitivity_vec.try_into() {
                            Ok(sensitivity_bytes) => sensitivity_bytes,
                            Err(err) => { 
                                return Err(StdError::generic_err(format!("{:?}", err))) 
                            },
                        }
                    );
                    *may_sensitivity = Some(sensitivity);
                    Ok(Some(sensitivity))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Set the average sensitivity
    pub fn set_average_sensitivity(
        &self, 
        storage: &mut dyn Storage, 
        average_sensitivity: Option<I32F32>,
    ) -> StdResult<()> {
        let average_sensitivity_key = [self.as_slice(), SENSITIVITY_FOR_AVG_KEY].concat();
        let stored_average_sensitivity: Option<[u8;8]>;
        if let Some(may_sensitivity) = average_sensitivity {
            stored_average_sensitivity = Some(may_sensitivity.to_be_bytes());
        } else {
            stored_average_sensitivity = None;
        }
        let data = Ser::serialize(&stored_average_sensitivity)?;
        storage.set(&average_sensitivity_key, &data);
        Ok(())
    }

    pub fn get_privacy_budget(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<I32F32> {
        let mut may_privacy_budget = self.privacy_budget.lock().unwrap();
        match *may_privacy_budget {
            Some(privacy_budget) => {
                Ok(privacy_budget)
            }
            None => {
                let budget_key = [self.as_slice(), PRIVACY_BUDGET_KEY].concat();
                if let Some(budget_vec) = storage.get(&budget_key) {
                    let privacy_budget = I32F32::from_be_bytes(
                        match budget_vec.try_into() {
                            Ok(privacy_budget_bytes) => privacy_budget_bytes,
                            Err(err) => { 
                                return Err(StdError::generic_err(format!("{:?}", err))) 
                            },
                        }
                    );
                    *may_privacy_budget = Some(privacy_budget);
                    Ok(privacy_budget)
                } else {
                    // default privacy budget = 1
                    let privacy_budget = I32F32::from(1);
                    *may_privacy_budget = Some(privacy_budget);
                    Ok(privacy_budget)
                }
            }
        }
    }

    /// Set the privacy budget
    pub fn set_privacy_budget(
        &self, 
        storage: &mut dyn Storage, 
        budget: I32F32,
    ) {
        let budget_key = [self.as_slice(), PRIVACY_BUDGET_KEY].concat();
        storage.set(&budget_key, &budget.to_be_bytes());
    }

    pub fn get_status(
        &self,
        storage: &dyn Storage,
    ) -> StdResult<RunningStatsStatus> {
        let mut may_status = self.status.lock().unwrap();
        match *may_status {
            Some(status) => {
                Ok(status)
            }
            None => {
                let status_key = [self.as_slice(), STATUS_KEY].concat();
                if let Some(status_vec) = storage.get(&status_key) {
                    let status_byte = status_vec
                        .as_slice()
                        .try_into()
                        .map_err(|err| StdError::parse_err("u8", err))?;
                    let status = u8::from_be_bytes(status_byte);
                    let status = match status {
                        STATUS_COLLECTING_DATA => RunningStatsStatus::CollectingData,
                        STATUS_CALCULATING_STATS => RunningStatsStatus::CalculatingStats,
                        _ => { return Err(StdError::generic_err("Invalid u8 value for storage status")) }
                    };
                    *may_status = Some(status);
                    Ok(status)
                } else {
                    *may_status = Some(RunningStatsStatus::CollectingData);
                    Ok(RunningStatsStatus::CollectingData)
                }
            }
        }
    }

    pub fn set_status(
        &self,
        storage: &mut dyn Storage,
        status: RunningStatsStatus,
    ) -> StdResult<()> {
        let status_key = [self.as_slice(), STATUS_KEY].concat();
        match status { 
            RunningStatsStatus::CollectingData => {
                if self.get_status(storage)? == RunningStatsStatus::CalculatingStats {
                    return Err(StdError::generic_err("Cannot set status to collecting data after changing to calculating stats"));
                }
                storage.set(&status_key, &[STATUS_COLLECTING_DATA]);
            }
            RunningStatsStatus::CalculatingStats => {
                if self.get_count(storage)? == 0 {
                    return Err(StdError::generic_err("No data in running stats store"));
                }
                storage.set(&status_key, &[STATUS_CALCULATING_STATS]);
            }
        }
        Ok(())
    }

    pub fn clear(
        &self,
        storage: &mut dyn Storage,
    ) {
        self.set_count(storage, 0);
        self.set_sum(storage, I64F64::from(0));

    }

    pub fn add_observation(&self, storage: &mut dyn Storage, x: I32F32) -> StdResult<()> {
        if self.get_status(storage)? != RunningStatsStatus::CollectingData {
            return Err(StdError::generic_err("Status is not set to collecting data") );
        }

        let new_count = self.get_count(storage)?.checked_add(1).ok_or(
            StdError::generic_err("Count overflow")
        )?;
        self.set_count(storage, new_count);

        let new_sum = self.get_sum(storage)?.checked_add(I64F64::from_num(x)).ok_or(
            StdError::generic_err("Sum overflow")
        )?;
        self.set_sum(storage, new_sum);

        if self.get_upper_bound(storage)? < x {
            self.set_upper_bound(storage, x);
        }

        if self.get_lower_bound(storage)? > x {
            self.set_lower_bound(storage, x);
        }

        Ok(())
    }

    pub fn fuzzy_count(&self, storage: &mut dyn Storage, rng: &mut ChaChaRng) -> StdResult<I32F32> {
        if self.get_status(storage)? != RunningStatsStatus::CalculatingStats {
            return Err(StdError::generic_err("Status not set to calculating stats") );
        }

        if self.is_empty(storage)? {
            return Err(StdError::generic_err("No data to count"));
        }

        let epsilon = self.get_epsilon(storage)?;
        let privacy_budget = self.get_privacy_budget(storage)?;
        if privacy_budget < epsilon { // privacy cost of COUNT = 1 * epsilon
            return Err(StdError::generic_err("Privacy budget exhausted"));
        }

        // sensitivity is always 1 for COUNT queries
        let sensitivity = I32F32::from_num(1_u32);
        
        // calculate a fuzzy count
        let scale = sensitivity / epsilon;
        let noise = laplace(rng, scale);
        let fuzzy_count = I32F32::from_num(self.get_count(storage)?) + noise;

        // update the remaining privacy budget
        self.set_privacy_budget(storage, privacy_budget - epsilon);

        Ok(fuzzy_count)
    }

    pub fn fuzzy_average(&self, storage: &mut dyn Storage, rng: &mut ChaChaRng) -> StdResult<I32F32> {
        if self.get_status(storage)? != RunningStatsStatus::CalculatingStats {
            return Err(StdError::generic_err("Status not set to calculating stats") );
        }

        if self.is_empty(storage)? {
            return Err(StdError::generic_err("No data to count"));
        }

        // sequential queries for sum + count
        let epsilon = self.get_epsilon(storage)?;
        let privacy_cost: I32F32 = 2 * epsilon;
        let privacy_budget = self.get_privacy_budget(storage)?;
        if privacy_budget < privacy_cost { 
            return Err(StdError::generic_err("Privacy budget exhausted"));
        }

        let sensitivity: I32F32;
        if let Some(sensitivity_for_average) = self.get_avg_sensitivity(storage)? {
            sensitivity = sensitivity_for_average;
        } else {
            // using a bounded sensitivity for sum
            sensitivity = self.get_upper_bound(storage)? - self.get_lower_bound(storage)?;
        }

        let scale = sensitivity / epsilon;
    
        let sum_noise = laplace(rng, scale);
        let dp_sum = self.get_sum(storage)? + I64F64::from_num(sum_noise);
    
        // calculate fuzzy count
        let sensitivity = I32F32::from_num(1_u32);
        let scale = sensitivity / epsilon;
    
        let noise = laplace(rng, scale);
        let real_count = I32F32::from_num(self.get_count(storage)?);
        let dp_count = I64F64::from_num(real_count + noise);
    
        let dp_average = I32F32::from_num(dp_sum / dp_count);

        // update the remaining privacy budget
        self.set_privacy_budget(storage, privacy_budget - privacy_cost);

        Ok(dp_average)
    }
}