use std::f64::consts::PI;

/// Models the input buffer stage of the Klon Centaur pedal in the digital domain.
///
/// Two processing stages:
///
/// 1. **High-pass filter** (~15 Hz, first-order IIR) — models the input coupling capacitor
///    (C1 ≈ 1 µF) interacting with the JFET input impedance.  Removes sub-sonic content
///    and gives the characteristic "firm" low-end of buffered-bypass circuits.
///
/// 2. **FET-style compressor** — approximates the gentle dynamic shaping of a JFET gain
///    stage running at low bias.  Attack 0.5 ms / release 50 ms / ratio 3:1 / soft knee 6 dB
///    at −18 dBFS.  No makeup gain is applied; the intent is subtle density, not loudness.
pub struct KlonBuffer {
    // High-pass state
    hp_x_prev: f64,
    hp_y_prev: f64,
    hp_r: f64,

    // FET compressor state
    envelope: f64,
    attack_coeff: f64,
    release_coeff: f64,
}

const HP_CUTOFF_HZ: f64 = 15.0;
const COMP_ATTACK_MS: f64 = 0.5;
const COMP_RELEASE_MS: f64 = 50.0;
const COMP_THRESHOLD_DB: f64 = -18.0;
const COMP_RATIO: f64 = 3.0;
const COMP_KNEE_DB: f64 = 6.0;

impl KlonBuffer {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            hp_x_prev: 0.0,
            hp_y_prev: 0.0,
            hp_r: 1.0 - (2.0 * PI * HP_CUTOFF_HZ / sample_rate),
            envelope: 0.0,
            attack_coeff: (-1.0 / (COMP_ATTACK_MS * 0.001 * sample_rate)).exp(),
            release_coeff: (-1.0 / (COMP_RELEASE_MS * 0.001 * sample_rate)).exp(),
        }
    }

    pub fn reset(&mut self) {
        self.hp_x_prev = 0.0;
        self.hp_y_prev = 0.0;
        self.envelope = 0.0;
    }

    pub fn process_sample(&mut self, input: f64) -> f64 {
        let hp = self.highpass(input);
        self.compress(hp)
    }

    /// First-order IIR high-pass: y[n] = R · (y[n-1] + x[n] − x[n-1])
    fn highpass(&mut self, input: f64) -> f64 {
        let output = self.hp_r * (self.hp_y_prev + input - self.hp_x_prev);
        self.hp_x_prev = input;
        self.hp_y_prev = output;
        output
    }

    /// Peak envelope follower + soft-knee gain computer.
    fn compress(&mut self, input: f64) -> f64 {
        let abs_input = input.abs();

        // Ballistics: fast attack, slower release
        if abs_input > self.envelope {
            self.envelope = self.attack_coeff * self.envelope + (1.0 - self.attack_coeff) * abs_input;
        } else {
            self.envelope = self.release_coeff * self.envelope;
        }

        // Gain computer in dB with soft knee
        let env_db = 20.0 * self.envelope.max(1e-10_f64).log10();
        let half_knee = COMP_KNEE_DB / 2.0;
        let gain_db = if env_db < COMP_THRESHOLD_DB - half_knee {
            0.0
        } else if env_db <= COMP_THRESHOLD_DB + half_knee {
            let x = env_db - COMP_THRESHOLD_DB + half_knee;
            (1.0 / COMP_RATIO - 1.0) * x * x / (2.0 * COMP_KNEE_DB)
        } else {
            COMP_THRESHOLD_DB + (env_db - COMP_THRESHOLD_DB) / COMP_RATIO - env_db
        };

        input * 10f64.powf(gain_db / 20.0)
    }
}
