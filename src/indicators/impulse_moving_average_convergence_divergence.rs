use crate::errors::Result;
use crate::indicators::ExponentialMovingAverage;
use crate::{Close, High, Low, Next};

pub struct ImpulseMACDParams {
    pub length_ma: usize,
    pub length_signal: usize,
}

pub struct ImpulseMACD {
    params: ImpulseMACDParams,
    hi_smma: Vec<f64>,
    lo_smma: Vec<f64>,
    mi_zlema: Vec<f64>,
    md: Vec<f64>,
    sb: Vec<f64>,
    sh: Vec<f64>,
    ema1: ExponentialMovingAverage,
    ema2: ExponentialMovingAverage,
}

impl ImpulseMACD {
    pub fn new(params: ImpulseMACDParams) -> Result<Self> {
        let lma = params.length_ma;
        let i_macd = ImpulseMACD {
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

        Ok(i_macd)
    }
}

impl<T: High + Low + Close> Next<&T> for ImpulseMACD {
    type Output = (f64, f64, f64, f64);

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

        (md, sb, sh, mi)
    }
}
