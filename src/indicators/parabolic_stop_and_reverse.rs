use std::fmt;

use crate::errors::Result;
use crate::{High, Low, Next, Reset};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Parabolic Stop and Reverse (Parabolic SAR) indicator.
///
/// The Parabolic SAR is a trend-following indicator used to determine the direction of an asset's momentum and potential reversal points.
/// It is displayed as a series of dots placed either above or below the price bars, indicating potential stop and reverse levels.
///
/// # Formula
/// The Parabolic SAR is calculated:
/// - If the trend is **up**, SAR is placed below the price and rises over time.
/// - If the trend is **down**, SAR is placed above the price and falls over time.
/// - The indicator starts with an initial acceleration factor (`step_af`) and increases it up to a maximum (`maximum_af`) as the trend extends.
///
/// The formula for the next SAR value is:
/// `SAR = previous_SAR + acceleration_factor * (extreme_point - previous_SAR)`
/// where `extreme_point` is the highest high (for uptrend) or lowest low (for downtrend) during the trend.
///
/// # Example
/// ```
/// use ta::{DataItem, Next};
/// use ta::indicators::ParabolicStopAndReverse;
///
/// let mut sar = ParabolicStopAndReverse::new(0.2, 0.02).unwrap();
/// let bar = DataItem::builder().open(105.0).high(105.0).low(95.0).close(95.0).volume(0.0).build().unwrap();
/// let sar_value = sar.next(&bar);
/// println!("Parabolic SAR: {}", sar_value);
/// ```
#[doc(alias = "ParabolicSAR")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]

pub struct ParabolicStopAndReverse {
    accelerator_factor: f64,
    maximum_af: f64,
    step_af: f64,
    extreme_point: f64,
    stop_and_reverse: f64,
    is_uptrend: bool,
    initialized: bool,
}

impl ParabolicStopAndReverse {
    pub fn new(max_af: f64, step_af: f64) -> Result<Self> {
        if max_af <= 0. || step_af <= 0. || max_af <= step_af {
            return Err(crate::errors::TaError::InvalidParameter);
        }

        Ok(Self {
            accelerator_factor: 0.0,
            maximum_af: max_af,
            step_af,
            extreme_point: 0.0,
            stop_and_reverse: 0.0,
            is_uptrend: false,
            initialized: false,
        })
    }

    pub fn is_uptrend(&self) -> bool {
        self.is_uptrend
    }
}

impl<T: High + Low> Next<&T> for ParabolicStopAndReverse {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        let (high, low) = (input.high(), input.low());

        if !self.initialized {
            self.is_uptrend = true; // uptrend by default
            self.extreme_point = high;
            self.stop_and_reverse = low;
            self.initialized = true;
            return self.stop_and_reverse;
        }

        let new_sar = self.stop_and_reverse
            + self.accelerator_factor * (self.extreme_point - self.stop_and_reverse);

        if self.is_uptrend && low < new_sar {
            self.is_uptrend = false;
            self.stop_and_reverse = self.extreme_point;
            self.extreme_point = low;
            self.accelerator_factor = 0.02;
            return self.stop_and_reverse;
        }

        if !self.is_uptrend && high > new_sar {
            self.is_uptrend = true;
            self.stop_and_reverse = self.extreme_point;
            self.extreme_point = high;
            self.accelerator_factor = 0.02;
            return self.stop_and_reverse;
        }

        self.stop_and_reverse = new_sar;
        let old_ep = self.extreme_point;

        self.extreme_point = if self.is_uptrend {
            self.extreme_point.max(high)
        } else {
            self.extreme_point.min(low)
        };

        if old_ep != self.extreme_point {
            self.accelerator_factor = (self.accelerator_factor + self.step_af).min(self.maximum_af);
        }

        self.stop_and_reverse
    }
}

impl Reset for ParabolicStopAndReverse {
    fn reset(&mut self) {
        self.accelerator_factor = 0.0;
        self.initialized = false;
    }
}

impl Default for ParabolicStopAndReverse {
    fn default() -> Self {
        Self {
            accelerator_factor: 0.02,
            maximum_af: 0.2,
            step_af: 0.02,
            extreme_point: 0.,
            stop_and_reverse: 0.,
            is_uptrend: false,
            initialized: false,
        }
    }
}

impl fmt::Display for ParabolicStopAndReverse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SAR({})", self.stop_and_reverse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    #[test]
    fn test_default() {
        ParabolicStopAndReverse::default();
    }

    #[test]
    fn test_initialization() {
        let mut sar = ParabolicStopAndReverse::default();
        let bar = Bar::new().high(15.0).low(5.0);
        assert_eq!(sar.next(&bar), 5.0);
        assert_eq!(sar.extreme_point, 15.0);
        assert!(sar.is_uptrend);
    }

    #[test]
    fn test_trend_up_to_down() {
        let mut sar = ParabolicStopAndReverse::new(0.2, 0.02).unwrap();
        let bar1 = Bar::new().high(15.0).low(5.0);
        sar.next(&bar1);
        assert!(sar.is_uptrend);
        let bar2 = Bar::new().high(16.0).low(4.0);
        let sar_value = sar.next(&bar2);
        assert!(!sar.is_uptrend);
        assert_eq!(sar_value, 15.0);
    }

    #[test]
    fn test_trend_down_to_up() {
        let mut sar = ParabolicStopAndReverse::new(0.2, 0.02).unwrap();
        let bar1 = Bar::new().high(15.0).low(5.0); // up by default
        let bar2 = Bar::new().high(16.0).low(4.0);
        sar.next(&bar1); // up
        sar.next(&bar2); // down
        assert!(!sar.is_uptrend);
        let bar3 = Bar::new().high(15.0).low(5.0);
        sar.next(&bar3); // re.up
        assert!(sar.is_uptrend);
    }

    #[test]
    fn test_acceleration_update() {
        let mut sar = ParabolicStopAndReverse::new(0.2, 0.02).unwrap();
        let bar1 = Bar::new().high(15.0).low(5.0);
        sar.next(&bar1);
        let initial_af = sar.accelerator_factor;

        let bar2 = Bar::new().high(18.0).low(12.0);
        sar.next(&bar2);
        assert!(sar.accelerator_factor > initial_af);
        assert!(sar.accelerator_factor <= 0.2);
    }

    #[test]
    fn test_display() {
        let sar = ParabolicStopAndReverse::new(0.2, 0.02).unwrap();
        assert_eq!(format!("{}", sar), "SAR(0)");
    }

    #[test]
    fn test_invalid_parameters() {
        let result = ParabolicStopAndReverse::new(0.0, 0.02);
        assert!(result.is_err());
        let result = ParabolicStopAndReverse::new(0.1, 0.2);
        assert!(result.is_err());
    }
}
