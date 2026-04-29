pub struct LowPassFilter {
    alpha: f64,
    prev: f64,
}

impl LowPassFilter {
    pub fn new(cutoff_hz: f64, sample_rate: f64) -> Self {
        let alpha = 1.0 - (-2.0 * std::f64::consts::PI * cutoff_hz / sample_rate).exp();
        Self { alpha, prev: 0.0 }
    }

    pub fn process_sample(&mut self, input: f64) -> f64 {
        self.prev = self.alpha * input + (1.0 - self.alpha) * self.prev;
        self.prev
    }

    pub fn reset(&mut self) {
        self.prev = 0.0;
    }

    pub fn set_cutoff(&mut self, cutoff_hz: f64, sample_rate: f64) {
        self.alpha = 1.0 - (-2.0 * std::f64::consts::PI * cutoff_hz / sample_rate).exp();
    }
}
