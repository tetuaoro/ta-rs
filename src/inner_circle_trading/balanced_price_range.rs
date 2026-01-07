use std::collections::VecDeque;
use std::fmt;

use super::FairValueGap;
use crate::{Next, Reset};

/// Balanced Price Range (BPR) Concept
///
/// The `BalancedPriceRange` struct detects balanced price ranges between Fair Value Gaps over a given period.
/// A BPR occurs when there is an overlap between two opposite FVGs within the period,
/// indicating a balanced area between supply and demand.
#[doc(alias = "BPR")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct BalancedPriceRange {
    period: usize,
    counter: usize,
    fvg: FairValueGap,          // Uses FairValueGap to detect FVGs
    fvgs: VecDeque<(f64, f64)>, // Stores detected FVGs (high, low)
}

impl BalancedPriceRange {
    /// Creates a new `BalancedPriceRange` with a given period.
    pub fn new(period: usize) -> Self {
        Self {
            period,
            counter: 0,
            fvg: FairValueGap::default(),
            fvgs: VecDeque::with_capacity(period + 1),
        }
    }

    /// Detects if there is an overlap between two FVGs.
    fn is_overlap(fvg1: &(f64, f64), fvg2: &(f64, f64)) -> Option<(f64, f64)> {
        let (high1, low1) = fvg1;
        let (high2, low2) = fvg2;
        if high2 < high1 && high2 > low1 {
            return Some((*high2, *low1));
        }
        if high1 < high2 && high1 > low2 {
            return Some((*high1, *low2));
        }
        None
    }
}

impl Next<(f64, f64)> for BalancedPriceRange {
    type Output = Option<(f64, f64)>;

    fn next(&mut self, input: (f64, f64)) -> Self::Output {
        self.counter += 1;

        if self.counter > self.period {
            self.counter = 0;
            self.fvgs.pop_front();
        }

        if let Some(fvg) = self.fvg.next(input) {
            self.fvgs.push_back(fvg);

            if let (Some(fvg2), Some(fvg1)) = (
                self.fvgs.back(),
                self.fvgs.get(self.fvgs.len().saturating_sub(2)),
            ) {
                return Self::is_overlap(fvg1, fvg2);
            }
        }
        None
    }
}

impl fmt::Display for BalancedPriceRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BPR(period={}, fvgs={:?})", self.period, self.fvgs)
    }
}

impl Default for BalancedPriceRange {
    fn default() -> Self {
        Self::new(7)
    }
}

impl Reset for BalancedPriceRange {
    fn reset(&mut self) {
        self.counter = 0;
        self.fvgs.clear();
        self.fvg.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bpr = BalancedPriceRange::new(7);
        assert_eq!(bpr.period, 7);
        assert_eq!(bpr.fvgs.len(), 0);
    }

    #[test]
    fn test_next_bpr() {
        let mut bpr = BalancedPriceRange::new(7);
        // Setup candles to create FVGs and then a BPR
        bpr.next((105.0, 99.0)); // Candle 1
        bpr.next((106.0, 100.0)); // Candle 2
        bpr.next((110.0, 108.0)); // Candle 3 (creates a bullish FVG)
        bpr.next((109.0, 106.0)); // Candle 4
        bpr.next((108.0, 106.0)); // Candle 5
        let result = bpr.next((104.0, 103.0)); // Candle 6 (creates a bearish FVG overlapping with the bullish FVG)

        assert!(result.is_some());
        let (high, low) = result.unwrap();
        assert_eq!(high, 106.0);
        assert_eq!(low, 105.0);
    }

    #[test]
    fn test_no_bpr() {
        let mut bpr = BalancedPriceRange::new(7);
        // Setup candles with no overlapping FVGs
        bpr.next((105.0, 99.0));
        bpr.next((104.0, 100.0));
        bpr.next((110.0, 106.0)); // Bullish FVG
        bpr.next((111.0, 107.0));
        bpr.next((112.0, 108.0));
        let result = bpr.next((113.0, 109.0)); // No overlapping FVG

        assert!(result.is_none());
    }

    #[test]
    fn test_reset() {
        let mut bpr = BalancedPriceRange::new(7);
        bpr.next((105.0, 99.0));
        bpr.next((104.0, 100.0));
        bpr.next((110.0, 106.0));
        assert!(!bpr.fvgs.is_empty());

        bpr.reset();
        assert!(bpr.fvgs.is_empty());
    }

    #[test]
    fn test_display() {
        let bpr = BalancedPriceRange::new(7);
        assert_eq!(format!("{}", bpr), "BPR(period=7, fvgs=[])");
    }
}
