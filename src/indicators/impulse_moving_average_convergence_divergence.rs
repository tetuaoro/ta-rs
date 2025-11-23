//! # Impulse MACD Indicator
//!
//! Implementation of the Impulse MACD indicator by LazyBear.
//! This indicator combines smoothed moving averages with MACD concepts
//! to identify momentum and trend strength.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::indicators::ExponentialMovingAverage;
use crate::{Close, High, Low, Next};

/// Parameters for ImpulseMACD indicator.
#[doc(alias = "ImpulseMACDParams")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct ImpulseMovingAverageConvergenceDivergenceParameters {
    /// Length for moving averages
    pub length_ma: usize,
    /// Length for signal line
    pub length_signal: usize,
}

impl Default for ImpulseMovingAverageConvergenceDivergenceParameters {
    fn default() -> Self {
        Self {
            length_ma: 9,
            length_signal: 3,
        }
    }
}

/// Impulse MACD indicator.
///
/// Implementation of the Impulse MACD indicator.
/// This indicator combines smoothed moving averages with MACD concepts
/// to identify momentum and trend strength.
///
/// ### Parameters
/// - `length_ma`: Length for the moving averages (default: 9)
/// - `length_signal`: Length for the signal line (default: 3)
///
/// ### Output
/// Returns a tuple of (md, sb, sh, mi) where:
/// - `md`: Momentum divergence
/// - `sb`: Smoothed momentum
/// - `sh`: Momentum histogram
/// - `mi`: Zero-lag EMA
///
/// ### Example
/// ```
/// use ta::{DataItem, Next};
/// use ta::indicators::ImpulseMovingAverageConvergenceDivergence;
/// use ta::indicators::ImpulseMovingAverageConvergenceDivergenceParameters;
///
/// let params = ImpulseMovingAverageConvergenceDivergenceParameters { length_ma: 20, length_signal: 50 };
/// let mut imacd = ImpulseMovingAverageConvergenceDivergence::new(params).unwrap();
/// let bar = DataItem::builder().open(105.0).high(105.0).low(95.0).close(95.0).volume(0.0).build().unwrap();
/// let output = imacd.next(&bar);
/// println!("ImpulseMACD: {:?}", output);
/// ```
#[doc(alias = "ImpulseMACD")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ImpulseMovingAverageConvergenceDivergence {
    params: ImpulseMovingAverageConvergenceDivergenceParameters,
    hi_smma: Vec<f64>,
    lo_smma: Vec<f64>,
    mi_zlema: Vec<f64>,
    md: Vec<f64>,
    sb: Vec<f64>,
    sh: Vec<f64>,
    ema1: ExponentialMovingAverage,
    ema2: ExponentialMovingAverage,
}

impl ImpulseMovingAverageConvergenceDivergence {
    pub fn new(params: ImpulseMovingAverageConvergenceDivergenceParameters) -> Result<Self> {
        let lma = params.length_ma;
        let imacd = Self {
            params,
            hi_smma: Vec::new(),
            lo_smma: Vec::new(),
            mi_zlema: Vec::new(),
            md: Vec::new(),
            sb: Vec::new(),
            sh: Vec::new(),
            ema1: ExponentialMovingAverage::new(lma)?,
            ema2: ExponentialMovingAverage::new(lma)?,
        };

        Ok(imacd)
    }
}

/// Output for ImpulseMACD indicator.
#[doc(alias = "ImpulseMACDOutput")]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct ImpulseMovingAverageConvergenceDivergenceOutput {
    pub md: f64,
    pub sb: f64,
    pub sh: f64,
    pub mi: f64,
}

impl<T: High + Low + Close> Next<&T> for ImpulseMovingAverageConvergenceDivergence {
    type Output = ImpulseMovingAverageConvergenceDivergenceOutput;

    fn next(&mut self, input: &T) -> Self::Output {
        let (high, low, close) = (input.high(), input.low(), input.close());

        let src = (high + low + close) / 3.0;
        let hi = if self.hi_smma.is_empty() {
            high
        } else {
            let last_hi = *self.hi_smma.last().unwrap();
            (last_hi * (self.params.length_ma - 1) as f64 + high) / self.params.length_ma as f64
        };
        self.hi_smma.push(hi);

        let lo = if self.lo_smma.is_empty() {
            low
        } else {
            let last_lo = *self.lo_smma.last().unwrap();
            (last_lo * (self.params.length_ma - 1) as f64 + low) / self.params.length_ma as f64
        };
        self.lo_smma.push(lo);

        let ema1_val = self.ema1.next(src);
        let ema2_val = self.ema2.next(ema1_val);
        let mi = ema1_val + (ema1_val - ema2_val);
        self.mi_zlema.push(mi);

        let md = if mi > hi {
            mi - hi
        } else if mi < lo {
            mi - lo
        } else {
            0.0
        };
        self.md.push(md);

        let sb = if self.md.len() <= self.params.length_signal {
            self.md.iter().sum::<f64>() / self.md.len() as f64
        } else {
            let last_md: Vec<f64> = self
                .md
                .iter()
                .rev()
                .take(self.params.length_signal)
                .copied()
                .collect();
            last_md.iter().sum::<f64>() / last_md.len() as f64
        };
        self.sb.push(sb);

        let sh = md - sb;
        self.sh.push(sh);

        ImpulseMovingAverageConvergenceDivergenceOutput { md, sb, sh, mi }
    }
}

impl Default for ImpulseMovingAverageConvergenceDivergence {
    fn default() -> Self {
        Self::new(Default::default()).unwrap()
    }
}
