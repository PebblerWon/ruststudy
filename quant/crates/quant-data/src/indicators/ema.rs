use super::{Indicator, IndicatorError, IndicatorOutput};

/// 指数移动平均线
/// EMA_t = price_t × k + EMA_{t-1} × (1 - k)
/// 其中 k = 2 / (period + 1)（平滑系数）
pub struct Ema {
    period: usize,
    multiplier: f64,
}

impl Ema {
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        if period <= 0 {
            return Err(IndicatorError::InvalidParam("SMA period 必须大于 0".into()));
        }
        Ok(Self {
            period,
            multiplier: 2.0 / (period as f64 + 1.0),
        })
    }
}

impl Indicator for Ema {
    fn name(&self) -> &str {
        "EMA"
    }
    fn compute(&self, input: &[f64]) -> Result<IndicatorOutput, IndicatorError> {
        if input.len() < self.period {
            return Err(IndicatorError::InsufficientData {
                required: self.period,
                actual: input.len(),
            });
        }
        let mut result = Vec::with_capacity(input.len());

        result.extend(std::iter::repeat(f64::NAN).take(self.period - 1));
        let first_ema: f64 = input[..self.period].iter().sum::<f64>() / self.period as f64;
        result.push(first_ema);

        for i in self.period..input.len() {
            let prev = result[i - 1];
            let a: f64 = input[i] * self.multiplier + prev * (1.0 - self.multiplier);
            result.push(a);
        }
        Ok(IndicatorOutput::Single(result))
    }
}
