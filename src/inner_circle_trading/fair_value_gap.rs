use std::collections::VecDeque;
use std::fmt;

use crate::{Next, Reset};

/// Fair Value Gap (FVG) Indicator
///
/// The `FairValueGap` struct detects Fair Value Gaps (FVGs) between candlesticks.
/// An FVG occurs when there is a gap between the high/low of non-consecutive candles,
/// indicating potential imbalances in supply and demand.
#[doc(alias = "FVG")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct FairValueGap {
    history: VecDeque<(f64, f64)>,
}

impl Next<(f64, f64)> for FairValueGap {
    type Output = Option<(f64, f64)>;

    fn next(&mut self, input: (f64, f64)) -> Self::Output {
        self.history.push_back(input);
        if self.history.len() > 3 {
            self.history.pop_front();
        }

        if let (Some(prev), Some(next)) = (self.history.front(), self.history.get(2)) {
            // uptrend
            if prev.0 - next.1 > 0.0 {
                return Some((prev.0, next.1));
            }
            // downtrend
            if prev.1 - next.0 > 0.0 {
                return Some((prev.1, next.0));
            }
        }
        None
    }
}

impl fmt::Display for FairValueGap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FVG({:?})", self.history)
    }
}

impl Default for FairValueGap {
    fn default() -> Self {
        Self {
            history: VecDeque::with_capacity(4),
        }
    }
}

impl Reset for FairValueGap {
    fn reset(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fvg = FairValueGap::default();
        assert_eq!(fvg.history.len(), 0);
    }

    #[test]
    fn test_next_bullish_fvg() {
        let mut fvg = FairValueGap::default();
        fvg.next((105.0, 99.0));
        fvg.next((104.0, 100.0));
        let result = fvg.next((110.0, 104.0));

        assert!(result.is_some());
        let (high, low) = result.unwrap();
        assert_eq!(high, 105.0);
        assert_eq!(low, 104.0);
    }

    #[test]
    fn test_next_bearish_fvg() {
        let mut fvg = FairValueGap::default();
        fvg.next((105.0, 99.0));
        fvg.next((100.0, 95.0));
        let result = fvg.next((98.0, 90.0));

        assert!(result.is_some());
        let (high, low) = result.unwrap();
        assert_eq!(high, 105.0);
        assert_eq!(low, 90.0);
    }

    #[test]
    fn test_no_fvg() {
        let mut fvg = FairValueGap::default();
        fvg.next((105.0, 99.0));
        fvg.next((104.0, 100.0));
        let result = fvg.next((110.0, 106.0));

        assert!(result.is_none());
    }

    #[test]
    fn test_reset() {
        let mut fvg = FairValueGap::default();
        fvg.next((105.0, 99.0));
        fvg.next((104.0, 100.0));
        assert_eq!(fvg.history.len(), 2);

        fvg.reset();
        assert_eq!(fvg.history.len(), 0);
    }

    #[test]
    fn test_display() {
        let fvg = FairValueGap::default();
        assert_eq!(format!("{}", fvg), "FVG([])");
    }
}
