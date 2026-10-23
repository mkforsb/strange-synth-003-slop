//! Master filter (Moog-style ladder with LP12/LP24/BP/HP taps) and the 5-band EQ.

use std::f32::consts::PI;

/// Rational tanh approximation, exact at +-3 and saturating beyond.
#[inline]
pub fn fast_tanh(x: f32) -> f32 {
    let x = x.clamp(-3.0, 3.0);
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

#[derive(Clone, Copy, Default)]
pub struct Ladder {
    s: [f32; 4],
}

impl Ladder {
    /// `g` one-pole coefficient (0..1), `k` resonance feedback (0..~4),
    /// `mode` 0 = LP12, 1 = LP24, 2 = BP, 3 = HP.
    #[inline]
    pub fn process(&mut self, x: f32, g: f32, k: f32, drive: f32, mode: usize) -> f32 {
        let inp = fast_tanh(x * drive - k * self.s[3]);
        let s = &mut self.s;
        s[0] += g * (inp - fast_tanh(s[0]));
        s[1] += g * (fast_tanh(s[0]) - fast_tanh(s[1]));
        s[2] += g * (s[1] - s[2]);
        s[3] += g * (s[2] - s[3]);
        let comp = 1.0 + 0.3 * k;
        match mode {
            0 => s[1] * comp,
            1 => s[3] * comp,
            2 => 8.0 * (s[1] - 2.0 * s[2] + s[3]),
            _ => inp - 4.0 * s[0] + 6.0 * s[1] - 4.0 * s[2] + s[3],
        }
    }

    pub fn reset(&mut self) {
        self.s = [0.0; 4];
    }
}

#[inline]
pub fn ladder_g(cutoff: f32, sr: f32) -> f32 {
    (1.0 - (-2.0 * PI * cutoff.min(sr * 0.45) / sr).exp()).min(0.85)
}

#[derive(Clone, Copy, Default)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z: [[f32; 2]; 2],
}

impl Biquad {
    pub fn identity() -> Self {
        Biquad { b0: 1.0, ..Default::default() }
    }

    fn set(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        let r = 1.0 / a0;
        self.b0 = b0 * r;
        self.b1 = b1 * r;
        self.b2 = b2 * r;
        self.a1 = a1 * r;
        self.a2 = a2 * r;
    }

    pub fn lowpass(&mut self, f: f32, q: f32, sr: f32) {
        let w = 2.0 * PI * f.min(sr * 0.45) / sr;
        let (s, c) = w.sin_cos();
        let al = s / (2.0 * q);
        self.set((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0, 1.0 + al, -2.0 * c, 1.0 - al);
    }

    pub fn highpass(&mut self, f: f32, q: f32, sr: f32) {
        let w = 2.0 * PI * f.min(sr * 0.45) / sr;
        let (s, c) = w.sin_cos();
        let al = s / (2.0 * q);
        self.set((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0, 1.0 + al, -2.0 * c, 1.0 - al);
    }

    pub fn peak(&mut self, f: f32, q: f32, db: f32, sr: f32) {
        let a = 10f32.powf(db / 40.0);
        let w = 2.0 * PI * f.min(sr * 0.45) / sr;
        let (s, c) = w.sin_cos();
        let al = s / (2.0 * q);
        self.set(1.0 + al * a, -2.0 * c, 1.0 - al * a, 1.0 + al / a, -2.0 * c, 1.0 - al / a);
    }

    pub fn low_shelf(&mut self, f: f32, db: f32, sr: f32) {
        let a = 10f32.powf(db / 40.0);
        let w = 2.0 * PI * f.min(sr * 0.45) / sr;
        let (s, c) = w.sin_cos();
        let beta = 2.0 * a.sqrt() * (s / 2.0 * 2f32.sqrt());
        self.set(
            a * ((a + 1.0) - (a - 1.0) * c + beta),
            2.0 * a * ((a - 1.0) - (a + 1.0) * c),
            a * ((a + 1.0) - (a - 1.0) * c - beta),
            (a + 1.0) + (a - 1.0) * c + beta,
            -2.0 * ((a - 1.0) + (a + 1.0) * c),
            (a + 1.0) + (a - 1.0) * c - beta,
        );
    }

    pub fn high_shelf(&mut self, f: f32, db: f32, sr: f32) {
        let a = 10f32.powf(db / 40.0);
        let w = 2.0 * PI * f.min(sr * 0.45) / sr;
        let (s, c) = w.sin_cos();
        let beta = 2.0 * a.sqrt() * (s / 2.0 * 2f32.sqrt());
        self.set(
            a * ((a + 1.0) + (a - 1.0) * c + beta),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * c),
            a * ((a + 1.0) + (a - 1.0) * c - beta),
            (a + 1.0) - (a - 1.0) * c + beta,
            2.0 * ((a - 1.0) - (a + 1.0) * c),
            (a + 1.0) - (a - 1.0) * c - beta,
        );
    }

    #[inline]
    pub fn process(&mut self, ch: usize, x: f32) -> f32 {
        let z = &mut self.z[ch];
        let y = self.b0 * x + z[0];
        z[0] = self.b1 * x - self.a1 * y + z[1];
        z[1] = self.b2 * x - self.a2 * y;
        y
    }
}

/// Low cut -> low shelf -> parametric mid -> high shelf -> high cut.
pub struct Eq {
    low_cut: Biquad,
    lo: Biquad,
    mid: Biquad,
    hi: Biquad,
    high_cut: Biquad,
}

impl Eq {
    pub fn new() -> Self {
        Eq {
            low_cut: Biquad::identity(),
            lo: Biquad::identity(),
            mid: Biquad::identity(),
            hi: Biquad::identity(),
            high_cut: Biquad::identity(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(&mut self, sr: f32, low_cut: f32, lo_db: f32, mid_db: f32, mid_f: f32, hi_db: f32, high_cut: f32) {
        self.low_cut.highpass(low_cut, 0.707, sr);
        self.lo.low_shelf(120.0, lo_db, sr);
        self.mid.peak(mid_f, 0.9, mid_db, sr);
        self.hi.high_shelf(6000.0, hi_db, sr);
        self.high_cut.lowpass(high_cut, 0.707, sr);
    }

    #[inline]
    pub fn process(&mut self, ch: usize, x: f32) -> f32 {
        let x = self.low_cut.process(ch, x);
        let x = self.lo.process(ch, x);
        let x = self.mid.process(ch, x);
        let x = self.hi.process(ch, x);
        self.high_cut.process(ch, x)
    }
}

impl Default for Eq {
    fn default() -> Self {
        Self::new()
    }
}
