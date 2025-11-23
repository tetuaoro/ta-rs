//! # Quadro Volume Profile Indicator
//!
//! Implementation of the Quad Volume Profile Indicator by BigBeluga.
//! Volume profile indicator that divides price action into four quadrants
//! to analyze buying and selling pressure at different price levels.

use std::collections::VecDeque;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Close, High, Low, Next, Open, Volume};

/// Volume Profile Output containing all calculated values.
#[doc(alias = "QVPOuput")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct QuadroVolumeProfileOutput {
    /// Upper sell quadrant volume
    pub upper_sell_volume: f64,
    /// Upper buy quadrant volume
    pub upper_buy_volume: f64,
    /// Lower sell quadrant volume
    pub lower_sell_volume: f64,
    /// Lower buy quadrant volume
    pub lower_buy_volume: f64,
    /// Total volume
    pub total_volume: f64,
    /// Price of Point of Control (POC)
    pub poc_price: f64,
    /// Volume at POC
    pub poc_volume: f64,
}

/// # Quadro Volume Profile Indicator
///
/// Volume profile indicator that divides price action into four quadrants
/// to analyze buying and selling pressure at different price levels.
///
/// ### Parameters
/// - `lookback`: Number of bars to include in calculations (default: 200)
/// - `bins`: Number of price bins to divide the range (default: 60)
/// - `offset`: Bar offset for profile positioning (default: 60)
///
/// ### Output
/// Returns a struct containing:
/// - Upper/lower buy/sell volumes
/// - Total volume
/// - Point of Control (POC) price and volume
///
/// ### Example
/// ```
/// use ta::{DataItem, Next};
/// use ta::indicators::QuadroVolumeProfile;
///
/// let mut qvp = QuadroVolumeProfile::new(200, 60, 60);
/// let bar = DataItem::builder().open(105.0).high(105.0).low(95.0).close(95.0).volume(0.0).build().unwrap();
/// let output = qvp.next(&bar);
/// println!("QVP: {:?}", output);
/// ```
#[doc(alias = "QVP")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct QuadroVolumeProfile {
    lookback: usize,
    bins: usize,
    offset: usize,
    high_low_history: VecDeque<f64>,
    volume_history: VecDeque<f64>,
    close_history: VecDeque<f64>,
    open_history: VecDeque<f64>,
    initialized: bool,
}

impl QuadroVolumeProfile {
    /// Creates a new QuadroVolumeProfile indicator
    ///
    /// # Arguments
    /// * `lookback` - Number of bars to consider (200 by default)
    /// * `bins` - Number of price bins (60 by default)
    /// * `offset` - Offset from current bar (60 by default)
    pub fn new(lookback: usize, bins: usize, offset: usize) -> Self {
        Self {
            lookback,
            bins,
            offset,
            high_low_history: VecDeque::with_capacity(lookback * 2),
            volume_history: VecDeque::with_capacity(lookback),
            close_history: VecDeque::with_capacity(lookback),
            open_history: VecDeque::with_capacity(lookback),
            initialized: false,
        }
    }

    /// Calculates the volume profile for the current bar
    fn calculate_profile(&self) -> QuadroVolumeProfileOutput {
        let mut top = f64::MIN;
        let mut bottom = f64::MAX;
        let mut total_volume = 0.0;

        // Find price range and total volume from the lookback period
        for i in 0..self.lookback {
            let high = self.high_low_history[i * 2];
            let low = self.high_low_history[i * 2 + 1];
            top = top.max(high);
            bottom = bottom.min(low);
            total_volume += self.volume_history[i];
        }

        let step = (top - bottom) / self.bins as f64;
        let mut left_profile = vec![0.0; self.bins];
        let mut right_profile = vec![0.0; self.bins];
        let mut tot_buy1 = Vec::new();
        let mut tot_sell1 = Vec::new();
        let mut tot_buy2 = Vec::new();
        let mut tot_sell2 = Vec::new();

        // Get the current close price (with offset)
        let current_close_index = self.close_history.len() - 1;
        let current_close = if current_close_index >= self.offset {
            self.close_history[current_close_index - self.offset]
        } else {
            self.close_history[current_close_index]
        };

        // Calculate volume profiles
        for i in 0..self.lookback {
            let close = self.close_history[i];
            let open = self.open_history[i];
            let volume = self.volume_history[i];
            let is_bull = close > open;
            let bull_vol = if is_bull { volume } else { 0.0 };
            let bear_vol = if is_bull { 0.0 } else { volume };

            for k in 0..self.bins {
                let low = bottom + step * k as f64;
                let high = low + step;
                let middle = low + step / 2.0;

                if close >= low - step && close <= high + step {
                    if close > middle {
                        left_profile[k] += bull_vol;
                        right_profile[k] += bear_vol;
                    } else {
                        left_profile[k] += bear_vol;
                        right_profile[k] += bull_vol;
                    }
                }
            }
        }

        // Calculate quadrant volumes using the offset close price
        for k in 0..self.bins {
            let low = bottom + step * k as f64;
            let high = low + step;
            let vol1 = left_profile[k];
            let vol2 = right_profile[k];

            if current_close < high {
                tot_sell1.push(vol1);
                tot_buy1.push(vol2);
            }
            if current_close > low {
                tot_buy2.push(vol1);
                tot_sell2.push(vol2);
            }
        }

        // Find POC and quadrant volumes
        let mut poc_price = 0.0;
        let mut poc_volume = 0.0;
        let mut max_left = 0.0;
        let mut max_right = 0.0;

        for k in 0..self.bins {
            let low = bottom + step * k as f64;
            let middle = low + step / 2.0;
            let vol1 = left_profile[k];
            let vol2 = right_profile[k];

            if vol1 > max_left {
                max_left = vol1;
                poc_price = middle;
                poc_volume = vol1;
            }
            if vol2 > max_right {
                max_right = vol2;
                poc_price = middle;
                poc_volume = vol2;
            }
        }

        QuadroVolumeProfileOutput {
            upper_sell_volume: tot_sell1.iter().sum(),
            upper_buy_volume: tot_buy1.iter().sum(),
            lower_sell_volume: tot_sell2.iter().sum(),
            lower_buy_volume: tot_buy2.iter().sum(),
            total_volume,
            poc_price,
            poc_volume: poc_volume.max(max_right), // Take the maximum volume as POC
        }
    }
}

impl<T: Open + High + Low + Close + Volume> Next<&T> for QuadroVolumeProfile {
    type Output = QuadroVolumeProfileOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        let (open, close) = (input.open(), input.close());
        let (high, low) = (input.high(), input.low());
        let volume = input.volume();

        // Initialize history buffers
        if !self.initialized {
            for _ in 0..self.lookback {
                self.high_low_history.push_back(0.0);
                self.high_low_history.push_back(0.0);
                self.volume_history.push_back(0.0);
                self.close_history.push_back(0.0);
                self.open_history.push_back(0.0);
            }
            self.initialized = true;
        }

        // Update history
        self.high_low_history.push_back(high);
        self.high_low_history.push_back(low);
        self.volume_history.push_back(volume);
        self.close_history.push_back(close);
        self.open_history.push_back(open);

        // Remove oldest values if we have enough data
        if self.high_low_history.len() > self.lookback * 2 {
            self.high_low_history.pop_front();
            self.high_low_history.pop_front();
            self.volume_history.pop_front();
            self.close_history.pop_front();
            self.open_history.pop_front();
        }

        // Only calculate when we have enough data
        if self.close_history.len() >= self.lookback {
            self.calculate_profile()
        } else {
            QuadroVolumeProfileOutput {
                upper_sell_volume: 0.0,
                upper_buy_volume: 0.0,
                lower_sell_volume: 0.0,
                lower_buy_volume: 0.0,
                total_volume: 0.0,
                poc_price: 0.0,
                poc_volume: 0.0,
            }
        }
    }
}

impl Default for QuadroVolumeProfile {
    fn default() -> Self {
        Self::new(200, 60, 60)
    }
}
