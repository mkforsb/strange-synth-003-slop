//! One polyphonic voice: swarm of feedback-FM oscillators -> chaos/ring/noise
//! -> 8-mode resonant body (with bow feedback) -> formants -> growl, mangle, stutter.

use crate::filters::fast_tanh;
use crate::params::{real, P};
use std::f32::consts::PI;

const TAU: f32 = 2.0 * PI;
pub const MODES: usize = 8;

/// Mode frequency ratios for string, free bar, sheet, bell, closed pipe.
const MATERIALS: [[f32; MODES]; 5] = [
    [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
    [1.0, 2.756, 5.404, 8.933, 13.34, 18.64, 24.81, 31.87],
    [1.0, 1.58, 2.14, 2.65, 2.92, 3.43, 3.62, 4.03],
    [0.56, 0.92, 1.19, 1.71, 2.0, 2.74, 3.0, 3.76],
    [1.0, 3.0, 5.0, 7.0, 9.0, 11.0, 13.0, 15.0],
];

const VOWEL_F: [[f32; 3]; 5] = [
    [730.0, 1090.0, 2440.0],
    [530.0, 1840.0, 2480.0],
    [270.0, 2290.0, 3010.0],
    [570.0, 840.0, 2410.0],
    [300.0, 870.0, 2240.0],
];

/// Block-rate snapshot of every parameter in engine units.
#[derive(Clone, Default)]
pub struct Pr {
    pub v: Vec<f32>,
}

impl Pr {
    pub fn from_norm(n: &[f32]) -> Pr {
        Pr { v: (0..n.len()).map(|i| real(i, n[i])).collect() }
    }
    #[inline]
    pub fn g(&self, p: P) -> f32 {
        self.v[p as usize]
    }
}

pub struct Rng(pub u32);

impl Rng {
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / 2147483648.0) - 1.0
    }
}

#[derive(Clone, Copy, Default)]
struct Mode {
    y1: f32,
    y2: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    w: f32,
}

#[derive(Clone, Copy, Default)]
struct Svf {
    low: f32,
    band: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Idle,
    Attack,
    Decay,
    Release,
}

pub struct Voice {
    pub note: i32,
    pub pedal: bool,
    pub age: u64,
    stage: Stage,
    env: f32,
    vel: f32,
    t: f32,
    pitch: f32,
    drift: f32,
    ph: [f32; 3],
    mph: [f32; 3],
    yprev: [f32; 3],
    ring_ph: f32,
    chaos_x: f32,
    chaos_acc: f32,
    chaos_out: f32,
    modes: [Mode; MODES],
    body_prev: f32,
    body_env: f32,
    strike_env: f32,
    form: [Svf; 3],
    growl_ph: f32,
    stut_ph: f32,
    stut_s: f32,
    erode_cnt: i32,
    erode_t: f32,
    erode_g: f32,
    hold_val: f32,
    hold_cnt: f32,
    pan: f32,
    rng: Rng,
}

impl Voice {
    pub fn new(seed: u32) -> Self {
        Voice {
            note: -1,
            pedal: false,
            age: 0,
            stage: Stage::Idle,
            env: 0.0,
            vel: 1.0,
            t: 0.0,
            pitch: 60.0,
            drift: 0.0,
            ph: [0.0; 3],
            mph: [0.0; 3],
            yprev: [0.0; 3],
            ring_ph: 0.0,
            chaos_x: 0.37,
            chaos_acc: 0.0,
            chaos_out: 0.0,
            modes: [Mode::default(); MODES],
            body_prev: 0.0,
            body_env: 0.0,
            strike_env: 0.0,
            form: [Svf::default(); 3],
            growl_ph: 0.0,
            stut_ph: 0.0,
            stut_s: 1.0,
            erode_cnt: 0,
            erode_t: 1.0,
            erode_g: 1.0,
            hold_val: 0.0,
            hold_cnt: 0.0,
            pan: 0.0,
            rng: Rng(seed | 1),
        }
    }

    pub fn is_active(&self) -> bool {
        self.stage != Stage::Idle
    }

    pub fn is_gated(&self) -> bool {
        matches!(self.stage, Stage::Attack | Stage::Decay)
    }

    pub fn level(&self) -> f32 {
        self.env + self.body_env
    }

    pub fn note_on(&mut self, note: i32, vel: f32, from_pitch: Option<f32>, age: u64) {
        let fresh = self.stage == Stage::Idle;
        self.note = note;
        self.vel = 0.35 + 0.65 * vel;
        self.stage = Stage::Attack;
        self.pedal = false;
        self.t = 0.0;
        self.age = age;
        self.strike_env = 1.0;
        self.pitch = from_pitch.unwrap_or(note as f32);
        if fresh {
            self.env = 0.0;
            self.chaos_x = 0.2 + 0.6 * (self.rng.next() * 0.5 + 0.5);
            self.pan = self.rng.next();
            self.modes = [Mode::default(); MODES];
            self.body_prev = 0.0;
            self.body_env = 0.0;
            self.drift = 0.0;
            self.form = [Svf::default(); 3];
        }
        self.stut_ph = 0.0;
    }

    pub fn note_off(&mut self) {
        if self.is_gated() {
            self.stage = Stage::Release;
        }
    }

    pub fn kill(&mut self) {
        self.stage = Stage::Idle;
        self.env = 0.0;
    }

    /// Adds `n` samples into the stereo bus.
    #[allow(clippy::too_many_arguments)]
    pub fn process(&mut self, p: &Pr, lfo: &[f32], n: usize, sr: f32, out_l: &mut [f32], out_r: &mut [f32]) {
        let wave = p.g(P::Wave);
        let warp = p.g(P::Warp);
        let fold = p.g(P::Fold);
        let fbk = p.g(P::Feedback);
        let fmr = p.g(P::FmRatio);
        let fmd = p.g(P::FmDepth);
        let swarm = p.g(P::Swarm);
        let ring = p.g(P::Ring);
        let ringr = p.g(P::RingRatio);
        let chaos = p.g(P::Chaos);
        let chaos_r = p.g(P::ChaosR);
        let noise = p.g(P::Noise);
        let body = p.g(P::Body);
        let strike = p.g(P::Strike);
        let bow = p.g(P::Bow);
        let formant = p.g(P::Formant);
        let growl = p.g(P::Growl);
        let growl_rate = p.g(P::GrowlRate);
        let dive = p.g(P::Dive);
        let dive_t = p.g(P::DiveTime);
        let glide = p.g(P::Glide);
        let drift_amt = p.g(P::Drift);
        let wobble = p.g(P::Wobble);
        let wob_depth = wobble * wobble * 12.0;
        let stutter = p.g(P::Stutter);
        let stut_rate = p.g(P::StutterRate);
        let erode = p.g(P::Erode);
        let crush = p.g(P::Crush);
        let decim = p.g(P::Decimate);
        let atk = p.g(P::Attack);
        let dec = p.g(P::Decay);
        let sus = p.g(P::Sustain);
        let rel = p.g(P::Release);
        let spread = p.g(P::Spread);

        let glide_c = 1.0 - (-1.0 / (glide * sr)).exp();
        let dec_c = (-5.0 / (dec * sr)).exp();
        let rel_c = (-5.0 / (rel * sr)).exp();
        let atk_inc = 1.0 / (atk * sr);
        let strike_c = (-1.0 / (0.0012 * sr)).exp();
        let levels = 2f32.powf(15.0 - crush * 13.0);
        let hold_len = 1.0 + decim * decim * 47.0;
        let swarm_on = swarm > 0.002;
        let wv = wave.clamp(0.0, 2.9999);
        let wi = wv.floor() as usize;
        let wf = wv - wi as f32;
        let warp_pow = 1.0 + warp * 3.0;
        let fold_g = 1.0 + fold * 9.0;
        let fold_mix = (fold * 8.0).min(1.0);
        let pan_a = (self.pan * spread + 1.0) * PI / 4.0;
        let (pan_r, pan_l) = pan_a.sin_cos();

        // ---- block-rate body / formant coefficients
        let semis0 = self.pitch + dive * (-self.t / dive_t).exp();
        let f0 = (440.0 * ((semis0 - 69.0) / 12.0).exp2()).clamp(8.0, 8000.0);
        if body > 0.001 {
            let tune = (p.g(P::BodyTune) / 12.0).exp2();
            let decay = p.g(P::BodyDecay);
            let damp = p.g(P::BodyDamp);
            let inh = p.g(P::Inharm);
            let mat = p.g(P::Material).clamp(0.0, 3.9999);
            let mi = mat.floor() as usize;
            let mf = mat - mi as f32;
            for k in 0..MODES {
                let ratio = MATERIALS[mi][k] * (1.0 - mf) + MATERIALS[mi + 1][k] * mf;
                let ratio = ratio.powf(1.0 + 0.35 * inh) * (1.0 + inh * 0.021 * (k as f32 * 2.7).sin());
                let fk = f0 * tune * ratio;
                let m = &mut self.modes[k];
                if fk < sr * 0.45 {
                    let t60 = (decay / (1.0 + damp * 3.0 * k as f32)).max(0.004);
                    let r = (-6.9078 / (t60 * sr)).exp();
                    let th = TAU * fk / sr;
                    m.a1 = 2.0 * r * th.cos();
                    m.a2 = -r * r;
                    m.b0 = (1.0 - r) * (1.0 - 2.0 * r * (2.0 * th).cos() + r * r).sqrt();
                    m.w = 1.0 / (1.0 + 0.3 * k as f32);
                } else {
                    m.b0 = 0.0;
                    m.w = 0.0;
                }
            }
        }
        let vo = p.g(P::Vowel).clamp(0.0, 3.9999);
        let vi = vo.floor() as usize;
        let vf = vo - vi as f32;
        let mut form_f = [0.0f32; 3];
        for (j, ff) in form_f.iter_mut().enumerate() {
            let hz = VOWEL_F[vi][j] * (1.0 - vf) + VOWEL_F[vi + 1][j] * vf;
            *ff = 2.0 * (PI * hz / sr).sin();
        }
        const FORM_Q: f32 = 0.14;
        const FORM_G: [f32; 3] = [1.0, 0.75, 0.5];

        for i in 0..n {
            if self.stage == Stage::Idle {
                break;
            }
            // ---- envelope
            match self.stage {
                Stage::Attack => {
                    self.env += atk_inc;
                    if self.env >= 1.0 {
                        self.env = 1.0;
                        self.stage = Stage::Decay;
                    }
                }
                Stage::Decay => {
                    self.env = sus + (self.env - sus) * dec_c;
                    if sus < 0.0005 && self.env < 0.0005 {
                        self.stage = Stage::Release;
                    }
                }
                Stage::Release => {
                    self.env *= rel_c;
                    if self.env < 1e-4 {
                        self.env = 0.0;
                        if self.body_env < 2e-4 {
                            self.stage = Stage::Idle;
                            break;
                        }
                    }
                }
                Stage::Idle => {}
            }

            // ---- pitch
            self.t += 1.0 / sr;
            self.pitch += (self.note as f32 - self.pitch) * glide_c;
            self.drift = self.drift * 0.9999 + self.rng.next() * 0.012;
            let semis = self.pitch
                + dive * (-self.t / dive_t).exp()
                + lfo[i] * wob_depth
                + self.drift * drift_amt * 1.2;
            let f = (440.0 * ((semis - 69.0) / 12.0).exp2()).clamp(4.0, sr * 0.4);

            // ---- oscillator swarm with feedback FM, warp, fold
            let mut acc = 0.0;
            let mut wsum = 1.0;
            let voices = if swarm_on { 3 } else { 1 };
            for u in 0..voices {
                let det = [0.0, 1.0, -1.3][u] * swarm * 45.0;
                let fu = f * (det / 1200.0).exp2();
                self.ph[u] += fu / sr;
                self.ph[u] -= self.ph[u].floor();
                self.mph[u] += fu * fmr / sr;
                self.mph[u] -= self.mph[u].floor();
                let m = (TAU * self.mph[u]).sin();
                let mut ph = self.ph[u] + 0.5 * (fmd * m + fbk * self.yprev[u]);
                ph -= ph.floor();
                if warp > 0.001 {
                    ph = ph.powf(warp_pow);
                }
                let shape = |k: usize| -> f32 {
                    match k {
                        0 => (TAU * ph).sin(),
                        1 => 1.0 - 4.0 * (ph - 0.5).abs(),
                        2 => 2.0 * ph - 1.0,
                        _ => {
                            if ph < 0.5 {
                                1.0
                            } else {
                                -1.0
                            }
                        }
                    }
                };
                let mut y = shape(wi) * (1.0 - wf) + shape(wi + 1) * wf;
                if fold > 0.001 {
                    y = y * (1.0 - fold_mix) + (y * fold_g * std::f32::consts::FRAC_PI_2).sin() * fold_mix;
                }
                self.yprev[u] = y;
                let w = if u == 0 { 1.0 } else { swarm };
                if u > 0 {
                    wsum += swarm;
                }
                acc += y * w;
            }
            let mut y = acc / wsum;

            // ---- ring mod, chaos, noise
            if ring > 0.001 {
                self.ring_ph += f * ringr / sr;
                self.ring_ph -= self.ring_ph.floor();
                y *= 1.0 - ring + ring * (TAU * self.ring_ph).sin();
            }
            if chaos > 0.001 {
                self.chaos_acc += f * 2.0 / sr;
                if self.chaos_acc >= 1.0 {
                    self.chaos_acc -= 1.0;
                    self.chaos_x = (chaos_r * self.chaos_x * (1.0 - self.chaos_x)).clamp(0.001, 0.999);
                }
                let target = self.chaos_x * 2.0 - 1.0;
                self.chaos_out += (target - self.chaos_out) * (f * 5.0 / sr).min(1.0);
                y = y * (1.0 - chaos) + self.chaos_out * 1.3 * chaos;
            }
            let white = self.rng.next();
            y += noise * white * 0.6;

            let xin = y * self.env * self.vel;

            // ---- resonant body
            let mut mixed = xin;
            if body > 0.001 {
                let sb = if self.strike_env > 1e-4 {
                    self.strike_env *= strike_c;
                    strike * self.strike_env * self.rng.next() * 0.04
                } else {
                    0.0
                };
                let u = xin + bow * self.env * 1.6 * fast_tanh(self.body_prev * 2.0);
                let mut sum = 0.0;
                for m in self.modes.iter_mut() {
                    let yk = m.b0 * u + m.a1 * m.y1 + m.a2 * m.y2 + sb;
                    m.y2 = m.y1;
                    m.y1 = yk;
                    sum += yk * m.w;
                }
                let b = sum * 0.5;
                self.body_prev = b;
                self.body_env = (b.abs()).max(self.body_env * 0.9997);
                mixed = xin * (1.0 - body) + b * body * 2.5;
            }

            // ---- formants
            if formant > 0.001 {
                let mut fs = 0.0;
                for j in 0..3 {
                    let s = &mut self.form[j];
                    s.low += form_f[j] * s.band;
                    let high = mixed - s.low - FORM_Q * s.band;
                    s.band += form_f[j] * high;
                    fs += s.band * FORM_Q * FORM_G[j] * 2.5;
                }
                mixed = mixed * (1.0 - formant) + fs * formant;
            }

            // ---- growl (amplitude roughness)
            if growl > 0.001 {
                self.growl_ph += growl_rate / sr;
                self.growl_ph -= self.growl_ph.floor();
                let g = 0.5 + 0.5 * (TAU * self.growl_ph).sin();
                mixed *= 1.0 - growl * g * g;
            }

            // ---- mangle
            if decim > 0.001 {
                self.hold_cnt += 1.0;
                if self.hold_cnt >= hold_len {
                    self.hold_cnt -= hold_len;
                    self.hold_val = mixed;
                }
                mixed = self.hold_val;
            }
            if crush > 0.001 {
                mixed = (mixed.clamp(-1.0, 1.0) * levels).round() / levels;
            }

            // ---- stutter & erode gates
            if stutter > 0.001 {
                self.stut_ph += stut_rate / sr;
                self.stut_ph -= self.stut_ph.floor();
                let target = if self.stut_ph < 0.5 { 1.0 } else { 1.0 - stutter };
                self.stut_s += (target - self.stut_s) * 0.03;
                mixed *= self.stut_s;
            }
            if erode > 0.001 {
                self.erode_cnt -= 1;
                if self.erode_cnt <= 0 {
                    self.erode_cnt = (sr / 160.0 * (0.5 + 0.5 * (self.rng.next() + 1.0))) as i32 + 1;
                    self.erode_t = if self.rng.next() * 0.5 + 0.5 < erode * 0.8 { 0.0 } else { 1.0 };
                }
                self.erode_g += (self.erode_t - self.erode_g) * 0.02;
                mixed *= self.erode_g;
            }

            if !mixed.is_finite() {
                mixed = 0.0;
                self.kill_state();
            }
            out_l[i] += mixed * pan_l;
            out_r[i] += mixed * pan_r;
        }
    }

    fn kill_state(&mut self) {
        self.modes = [Mode::default(); MODES];
        self.body_prev = 0.0;
        self.body_env = 0.0;
        self.yprev = [0.0; 3];
        self.form = [Svf::default(); 3];
        self.chaos_out = 0.0;
        self.hold_val = 0.0;
    }
}
