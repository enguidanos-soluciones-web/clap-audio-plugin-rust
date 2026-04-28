use std::f64::consts::PI;

/// The difference equation is:
/// y[n] = x[n] - x[n-1] + R · y[n-1]
/// Where:
/// * x[n] is the current input.
/// * y[n] is the current output.
/// * R is a coefficient between 0.99 and 0.999.
/// * R = 1.0 - (2.0 * PI * cut_freq / sample_rate)
///
pub struct DcFilter {
    x_prev: f64,
    y_prev: f64,
    r: f64,
}

impl DcFilter {
    pub fn new(cut_freq: f64, sample_rate: f64) -> Self {
        let instance = Self {
            x_prev: 0.0,
            y_prev: 0.0,
            r: 1.0 - (2.0 * PI * cut_freq / sample_rate),
        };

        // println!("DC Filter working in R={} Cut={cut_freq:.1} Rate={sample_rate:0}", instance.r);

        instance
    }

    pub fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }

    /// Implementation of a first-order IIR DC removal filter (DC Blocker).
    pub fn process_sample(&mut self, input: f64) -> f64 {
        let output = input - self.x_prev + self.r * self.y_prev;
        self.x_prev = input;
        self.y_prev = output;
        output
    }
}
