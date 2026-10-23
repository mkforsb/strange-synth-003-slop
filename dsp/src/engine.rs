//! Polyphonic engine: voices -> master filter -> EQ -> Dattorro reverb -> soft limiter.

use crate::filters::{fast_tanh, ladder_g, Eq, Ladder};
use crate::params::{self, P};
use crate::reverb::Reverb;
use crate::voice::{Pr, Rng, Voice};
use std::f32::consts::PI;

pub const BLOCK: usize = 128;
pub const VOICES: usize = 8;

pub struct Engine {
    sr: f32,
    norm: Vec<f32>,
    smooth: Vec<f32>,
    voices: Vec<Voice>,
    ladders: [Ladder; 2],
    eq: Eq,
    reverb: Reverb,
    dc: [(f32, f32); 2],
    lfo_ph: f32,
    lfo_sh: f32,
    lfo_sh_target: f32,
    rng: Rng,
    hold: bool,
    last_note: Option<i32>,
    clock: u64,
    /// Planar stereo output: left[BLOCK] then right[BLOCK].
    pub out: Vec<f32>,
    bus_l: Vec<f32>,
    bus_r: Vec<f32>,
    lfo: Vec<f32>,
}

impl Engine {
    pub fn new(sr: f32) -> Self {
        let norm: Vec<f32> = params::defaults().to_vec();
        Engine {
            sr,
            smooth: norm.clone(),
            norm,
            voices: (0..VOICES).map(|i| Voice::new(0x9E37_79B9u32.wrapping_mul(i as u32 + 1))).collect(),
            ladders: [Ladder::default(); 2],
            eq: Eq::new(),
            reverb: Reverb::new(sr),
            dc: [(0.0, 0.0); 2],
            lfo_ph: 0.0,
            lfo_sh: 0.0,
            lfo_sh_target: 0.0,
            rng: Rng(0xC0FF_EE11),
            hold: false,
            last_note: None,
            clock: 0,
            out: vec![0.0; BLOCK * 2],
            bus_l: vec![0.0; BLOCK],
            bus_r: vec![0.0; BLOCK],
            lfo: vec![0.0; BLOCK],
        }
    }

    pub fn set_param(&mut self, i: usize, v: f32) {
        if i < self.norm.len() {
            self.norm[i] = v.clamp(0.0, 1.0);
            if params::is_discrete(i) {
                self.smooth[i] = self.norm[i];
            }
        }
    }

    pub fn note_on(&mut self, note: i32, vel: f32) {
        self.clock += 1;
        let glide = params::real(P::Glide as usize, self.smooth[P::Glide as usize]);
        let from = if glide > 0.01 { self.last_note.map(|n| n as f32) } else { None };
        self.last_note = Some(note);
        let idx = self
            .voices
            .iter()
            .position(|v| v.is_active() && v.note == note)
            .or_else(|| self.voices.iter().position(|v| !v.is_active()))
            .unwrap_or_else(|| {
                // steal: quietest released voice, else the oldest
                let mut best = 0;
                let mut best_score = f32::MAX;
                for (i, v) in self.voices.iter().enumerate() {
                    let score = v.level() + if v.is_gated() { 10.0 } else { 0.0 } + v.age as f32 * 1e-9;
                    if score < best_score {
                        best_score = score;
                        best = i;
                    }
                }
                best
            });
        let clock = self.clock;
        self.voices[idx].note_on(note, vel, from, clock);
    }

    pub fn note_off(&mut self, note: i32) {
        for v in self.voices.iter_mut() {
            if v.is_gated() && v.note == note {
                if self.hold {
                    v.pedal = true;
                } else {
                    v.note_off();
                }
            }
        }
    }

    pub fn set_hold(&mut self, on: bool) {
        self.hold = on;
        if !on {
            for v in self.voices.iter_mut() {
                if v.pedal {
                    v.pedal = false;
                    v.note_off();
                }
            }
        }
    }

    pub fn all_off(&mut self) {
        for v in self.voices.iter_mut() {
            v.pedal = false;
            v.note_off();
        }
    }

    pub fn panic(&mut self) {
        for v in self.voices.iter_mut() {
            v.kill();
        }
    }

    pub fn process(&mut self, n: usize) {
        let n = n.min(BLOCK);
        let sr = self.sr;

        for i in 0..self.smooth.len() {
            self.smooth[i] += (self.norm[i] - self.smooth[i]) * 0.3;
        }
        let pr = Pr::from_norm(&self.smooth);

        // ---- shared LFO (sine <-> sample & hold)
        let rate = pr.g(P::WobbleRate);
        let shape = pr.g(P::WobbleShape);
        for i in 0..n {
            self.lfo_ph += rate / sr;
            if self.lfo_ph >= 1.0 {
                self.lfo_ph -= 1.0;
                self.lfo_sh_target = self.rng.next();
            }
            self.lfo_sh += (self.lfo_sh_target - self.lfo_sh) * 0.004;
            let s = (2.0 * PI * self.lfo_ph).sin();
            self.lfo[i] = s * (1.0 - shape) + self.lfo_sh * shape;
        }

        // ---- voices
        self.bus_l[..n].fill(0.0);
        self.bus_r[..n].fill(0.0);
        for v in self.voices.iter_mut() {
            if v.is_active() {
                v.process(&pr, &self.lfo, n, sr, &mut self.bus_l, &mut self.bus_r);
            }
        }

        // ---- master section parameters
        let ftype = pr.g(P::FilterType) as usize;
        let cutoff = pr.g(P::Cutoff);
        let res = pr.g(P::Resonance);
        let k = res * 3.95;
        let drive = 1.0 + pr.g(P::FilterDrive) * 5.0;
        let fwob = pr.g(P::FilterWob);
        let g_static = ladder_g(cutoff, sr);

        self.eq.update(
            sr,
            pr.g(P::LowCut),
            pr.g(P::LoGain),
            pr.g(P::MidGain),
            pr.g(P::MidFreq),
            pr.g(P::HiGain),
            pr.g(P::HiCut),
        );
        self.reverb.mix = pr.g(P::RvMix);
        self.reverb.decay = 0.1 + pr.g(P::RvDecay) * 0.88;
        self.reverb.damping = pr.g(P::RvDamp) * 0.95;
        self.reverb.bandwidth = 0.1 + pr.g(P::RvBandwidth) * 0.8995;
        self.reverb.predelay_ms = pr.g(P::RvPre);
        self.reverb.mod_depth = pr.g(P::RvMod);
        let vol = pr.g(P::Volume);
        let gain = vol * vol * 5.0;

        for i in 0..n {
            let g = if fwob > 0.001 {
                ladder_g(cutoff * (self.lfo[i] * fwob * 3.0).exp2(), sr)
            } else {
                g_static
            };
            let mut ch = [self.bus_l[i], self.bus_r[i]];
            for (c, x) in ch.iter_mut().enumerate() {
                // DC blocker
                let (xp, yp) = self.dc[c];
                let y = *x - xp + 0.995 * yp;
                self.dc[c] = (*x, y);
                let y = self.ladders[c].process(y, g, k, drive, ftype);
                *x = self.eq.process(c, y);
            }
            let (l, r) = self.reverb.process(ch[0], ch[1]);
            self.out[i] = fast_tanh(l * gain);
            self.out[BLOCK + i] = fast_tanh(r * gain);
        }
        for i in n..BLOCK {
            self.out[i] = 0.0;
            self.out[BLOCK + i] = 0.0;
        }
        if !self.out[0].is_finite() || !self.out[BLOCK].is_finite() {
            self.ladders = [Ladder::default(); 2];
        }
    }
}
