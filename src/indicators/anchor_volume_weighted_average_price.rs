use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{Close, Next, Reset, Volume};

/// Anchor Volume Weighted Average Price (AVWAP)
///
/// This indicator calculates the Volume Weighted Average Price (VWAP) starting from a specific anchor point (first input).
/// The anchor point is typically the first input received, but it can be reset to any desired point.
///
/// # Formula
///
/// AVWAP = sum(Price * Volume) / sum(Volume)
///
/// The calculation starts from the first input and accumulates the weighted sum and total volume.
///
/// # Example
/// ```
/// use ta::indicators::AnchorVolumeWeightedAveragePrice;
/// use ta::DataItem;
/// use ta::Next;
///
/// let mut anchor_vwap = AnchorVolumeWeightedAveragePrice::new();
/// let bar = DataItem::builder().open(1.0).high(1.0).low(1.0).close(1.0).volume(1.0).build().unwrap();
/// let vwap_value = anchor_vwap.next(&bar);
/// println!("Anchor VWAP: {}", vwap_value);
/// ```
#[doc(alias = "AnchorVWAP")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct AnchorVolumeWeightedAveragePrice {
    cumulative_volume: f64,
    cumulative_weighted_sum: f64,
}

impl AnchorVolumeWeightedAveragePrice {
    /// Creates a new `AnchorVolumeWeightedAveragePrice` instance.
    ///
    /// # Returns
    /// A new instance of `AnchorVolumeWeightedAveragePrice`.
    pub fn new() -> Self {
        Self {
            cumulative_volume: 0.0,
            cumulative_weighted_sum: 0.0,
        }
    }
}

impl<T: Close + Volume> Next<&T> for AnchorVolumeWeightedAveragePrice {
    type Output = f64;

    fn next(&mut self, input: &T) -> Self::Output {
        let price = input.close();
        let volume = input.volume();

        // Update cumulative sums
        self.cumulative_volume += volume;
        self.cumulative_weighted_sum += price * volume;

        // Calculate VWAP
        if self.cumulative_volume == 0.0 {
            0.0
        } else {
            self.cumulative_weighted_sum / self.cumulative_volume
        }
    }
}

impl Reset for AnchorVolumeWeightedAveragePrice {
    fn reset(&mut self) {
        self.cumulative_volume = 0.0;
        self.cumulative_weighted_sum = 0.0;
    }
}

impl Default for AnchorVolumeWeightedAveragePrice {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AnchorVolumeWeightedAveragePrice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.cumulative_volume == 0.0 {
            write!(f, "AnchorVolumeWeightedAveragePrice(0)")
        } else {
            write!(
                f,
                "AnchorVolumeWeightedAveragePrice({:.2})",
                self.cumulative_weighted_sum / self.cumulative_volume
            )
        }
    }
}
