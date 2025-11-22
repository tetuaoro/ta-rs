use std::fmt;

use crate::{Close, Next, Open, Reset, Volume};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Cumulative Volume Delta (CVD) indicator.
///
/// This indicator calculates the cumulative difference between buying and selling volume
/// for each price movement. It is often used to gauge the strength of a trend by analyzing
/// whether volume is predominantly buying or selling.
///
/// # Example
/// ```
/// use ta::{Next, indicators::CumulativeVolumeDelta, DataItem};
/// let mut cvd = CumulativeVolumeDelta::new();
///
/// // Assume `data` is a struct implementing `Open`, `Close`, and `Volume` traits.
/// let data = DataItem::builder().open(1.0).high(1.0).low(1.0).close(1.0).volume(1.0).build().unwrap();
/// // or cvd.next((1.0, 2.0)); as `ask` and `bid` input
/// let delta = cvd.next(&data);
/// println!("Current CVD: {}", cvd);
/// ```
#[doc(alias = "CVD")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct CumulativeVolumeDelta {
    cumulative_delta: f64,
    history: Vec<f64>,
}

impl CumulativeVolumeDelta {
    /// Creates a new `CumulativeVolumeDelta` instance.
    ///
    /// # Returns
    /// A new instance of `CumulativeVolumeDelta`.
    pub fn new() -> Self {
        Self {
            cumulative_delta: 0.0,
            history: Vec::new(),
        }
    }

    /// The cumulative sum of volume deltas.
    pub fn cumulative_delta(&self) -> f64 {
        self.cumulative_delta
    }

    /// Historical record of individual volume deltas.
    pub fn history(&self) -> Vec<f64> {
        self.history.clone()
    }
}

impl Next<(f64, f64)> for CumulativeVolumeDelta {
    type Output = f64;

    fn next(&mut self, (ask, bid): (f64, f64)) -> Self::Output {
        let delta = if ask < bid {
            // Positive price movement: assume buying pressure
            bid
        } else if ask > bid {
            // Negative price movement: assume selling pressure
            -ask
        } else {
            // No price movement: delta is zero
            0.0
        };

        self.cumulative_delta += delta;
        self.history.push(delta);
        delta
    }
}

impl<T: Open + Close + Volume> Next<&T> for CumulativeVolumeDelta {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        let (open, close, volume) = (input.open(), input.close(), input.volume());
        let delta = if close > open {
            // Positive price movement: assume buying pressure
            volume
        } else if close < open {
            // Negative price movement: assume selling pressure
            -volume
        } else {
            // No price movement: delta is zero
            0.0
        };

        self.cumulative_delta += delta;
        self.history.push(delta);
        delta
    }
}

impl Reset for CumulativeVolumeDelta {
    fn reset(&mut self) {
        self.cumulative_delta = 0.0;
        self.history.clear();
    }
}

impl Default for CumulativeVolumeDelta {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CumulativeVolumeDelta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CVD({})", self.cumulative_delta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::*;

    #[test]
    fn test_new_cvd() {
        let cvd = CumulativeVolumeDelta::new();
        assert_eq!(cvd.cumulative_delta, 0.0);
        assert!(cvd.history.is_empty());
    }

    #[test]
    fn test_next_positive_price_movement() {
        let mut cvd = CumulativeVolumeDelta::new();
        let data = Bar::new().open(100.0).close(110.0).volume(1000.0);
        let delta = cvd.next(&data);
        assert!(delta > 0.0);
        assert_eq!(cvd.history.len(), 1);
        assert_eq!(cvd.cumulative_delta, delta);
    }

    #[test]
    fn test_next_negative_price_movement() {
        let mut cvd = CumulativeVolumeDelta::new();
        let data = Bar::new().open(100.0).close(90.0).volume(1000.0);
        let delta = cvd.next(&data);
        assert!(delta < 0.0);
        assert_eq!(cvd.history.len(), 1);
        assert_eq!(cvd.cumulative_delta, delta);
    }

    #[test]
    fn test_next_no_price_movement() {
        let mut cvd = CumulativeVolumeDelta::new();
        let data = Bar::new().open(100.0).close(100.0).volume(1000.0);
        let delta = cvd.next(&data);
        assert_eq!(delta, 0.0);
        assert_eq!(cvd.history.len(), 1);
        assert_eq!(cvd.cumulative_delta, 0.0);
    }

    #[test]
    fn test_cumulative_delta() {
        let mut cvd = CumulativeVolumeDelta::new();
        let data1 = Bar::new().open(100.0).close(110.0).volume(1000.0);
        let data2 = Bar::new().open(110.0).close(100.0).volume(1000.0);
        let delta1 = cvd.next(&data1);
        let delta2 = cvd.next(&data2);
        assert_eq!(cvd.cumulative_delta, delta1 + delta2);
        assert_eq!(cvd.history.len(), 2);
    }

    #[test]
    fn test_reset() {
        let mut cvd = CumulativeVolumeDelta::new();
        let data = Bar::new().open(100.0).close(110.0).volume(1000.0);
        cvd.next(&data);
        assert_eq!(cvd.history.len(), 1);
        assert_ne!(cvd.cumulative_delta, 0.0);

        cvd.reset();
        assert_eq!(cvd.cumulative_delta, 0.0);
        assert!(cvd.history.is_empty());
    }
}
