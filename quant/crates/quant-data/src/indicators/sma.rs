use crate::indicators::{Indicator, IndicatorError, IndicatorOutput};

/// 简单移动平均线
/// SMA = (P1 + P2 + ... + Pn) / n
pub struct Sma {
    /// 计算周期
    period: usize,
}

impl Sma {
    pub fn new(period: usize) -> Result<Self, IndicatorError> {
        if period == 0 {
            return Err(IndicatorError::InvalidParam(
                ("SMA period 必须大于 0".into()),
            ));
        }
        Ok(Sma { period })
    }
}

impl Indicator for Sma {
    fn name(&self) -> &str {
        "SMA"
    }

    fn compute(&self, input: &[f64]) -> Result<IndicatorOutput, IndicatorError> {
        if input.len() < self.period {
            return Err(IndicatorError::InsufficientData {
                required: self.period,
                actual: input.len(),
            });
        }

        let mut result = Vec::with_capacity(input.len());

        // for i in 0..self.period - 1 {
        //     result.push(f64::NAN);
        // }
        let a = std::iter::repeat(f64::NAN).take(self.period - 1);
        result.extend(a);

        // 使用滑动窗口计算 SMA
        // 维护窗口内元素之和，避免每次重新求和

        let mut window_sum: f64 = input[..self.period].iter().sum();
        result.push(window_sum / self.period as f64);

        for i in self.period..input.len() {
            window_sum += input[i] - input[i - self.period];
            result.push(window_sum / self.period as f64);
        }
        Ok(IndicatorOutput::Single(result))
    }
}
