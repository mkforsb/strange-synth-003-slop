//! Dattorro plate reverb (J. Dattorro, "Effect Design Part 1", JAES 45(9), 1997).
//! Delay lengths are specified for 29761 Hz and rescaled to the host rate.

use std::f32::consts::PI;

struct Delay {
    buf: Vec<f32>,
    w: usize,
}

impl Delay {
    fn new(len: usize) -> Self {
        Delay { buf: vec![0.0; len.max(4)], w: 0 }
    }

    #[inline]
    fn push(&mut self, x: f32) {
        self.buf[self.w] = x;
        self.w += 1;
        if self.w == self.buf.len() {
            self.w = 0;
        }
    }

    /// Sample written `d` pushes ago (d >= 1).
    #[inline]
    fn read(&self, d: usize) -> f32 {
        let n = self.buf.len();
        self.buf[(self.w + n - d % n) % n]
    }

    #[inline]
    fn read_frac(&self, d: f32) -> f32 {
        let i = d.floor();
        let f = d - i;
        let i = i as usize;
        self.read(i) * (1.0 - f) + self.read(i + 1) * f
    }
}

struct Allpass {
    d: Delay,
    len: f32,
}

impl Allpass {
    fn new(len: usize, headroom: usize) -> Self {
        Allpass { d: Delay::new(len + headroom + 4), len: len as f32 }
    }

    /// Schroeder allpass: v = x - g z; y = z + g v.
    #[inline]
    fn process(&mut self, x: f32, g: f32, modulation: f32) -> f32 {
        let z = self.d.read_frac((self.len + modulation).max(1.0));
        let v = x - g * z;
        self.d.push(v);
        z + g * v
    }
}

pub struct Reverb {
    sr: f32,
    predelay: Delay,
    bw_state: f32,
    input_ap: [Allpass; 4],
    l_mod_ap: Allpass,
    l_d1: Delay,
    l_lp: f32,
    l_ap2: Allpass,
    l_d2: Delay,
    r_mod_ap: Allpass,
    r_d1: Delay,
    r_lp: f32,
    r_ap2: Allpass,
    r_d2: Delay,
    l_fb: f32,
    r_fb: f32,
    lfo: f32,
    scale: f32,
    // parameters
    pub mix: f32,
    pub decay: f32,
    pub damping: f32,
    pub bandwidth: f32,
    pub predelay_ms: f32,
    pub mod_depth: f32,
}

impl Reverb {
    pub fn new(sr: f32) -> Self {
        let s = sr / 29761.0;
        let sz = |n: f32| (n * s).round() as usize;
        let head = (16.0 * s * 1.5) as usize + 4;
        Reverb {
            sr,
            predelay: Delay::new((sr * 0.2) as usize),
            bw_state: 0.0,
            input_ap: [
                Allpass::new(sz(142.0), 0),
                Allpass::new(sz(107.0), 0),
                Allpass::new(sz(379.0), 0),
                Allpass::new(sz(277.0), 0),
            ],
            l_mod_ap: Allpass::new(sz(672.0), head),
            l_d1: Delay::new(sz(4453.0) + 4),
            l_lp: 0.0,
            l_ap2: Allpass::new(sz(1800.0), 0),
            l_d2: Delay::new(sz(3720.0) + 4),
            r_mod_ap: Allpass::new(sz(908.0), head),
            r_d1: Delay::new(sz(4217.0) + 4),
            r_lp: 0.0,
            r_ap2: Allpass::new(sz(2656.0), 0),
            r_d2: Delay::new(sz(3163.0) + 4),
            l_fb: 0.0,
            r_fb: 0.0,
            lfo: 0.0,
            scale: s,
            mix: 0.2,
            decay: 0.5,
            damping: 0.4,
            bandwidth: 0.8,
            predelay_ms: 0.0,
            mod_depth: 0.3,
        }
    }

    #[inline]
    pub fn process(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let s = self.scale;
        let tap = |n: f32| (n * s).round() as usize;

        // input stage
        let mono = (in_l + in_r) * 0.5;
        self.predelay.push(mono);
        let pd = (self.predelay_ms * 0.001 * self.sr).max(1.0);
        let x = self.predelay.read_frac(pd);
        self.bw_state += self.bandwidth * (x - self.bw_state);
        let mut d = self.bw_state;
        d = self.input_ap[0].process(d, 0.75, 0.0);
        d = self.input_ap[1].process(d, 0.75, 0.0);
        d = self.input_ap[2].process(d, 0.625, 0.0);
        d = self.input_ap[3].process(d, 0.625, 0.0);

        // tank
        self.lfo += 1.0 / self.sr;
        if self.lfo > 1000.0 {
            self.lfo -= 1000.0;
        }
        let exc = 16.0 * s * self.mod_depth;
        let m_l = (2.0 * PI * 0.9 * self.lfo).sin() * exc;
        let m_r = (2.0 * PI * 1.13 * self.lfo + 1.7).sin() * exc;
        let dec = self.decay;
        let dd2 = (dec + 0.15).clamp(0.25, 0.5);

        let l_in = d + self.r_fb * dec;
        let r_in = d + self.l_fb * dec;

        let a = self.l_mod_ap.process(l_in, -0.7, m_l);
        self.l_d1.push(a);
        let b = self.l_d1.read(tap(4453.0));
        self.l_lp = (1.0 - self.damping) * b + self.damping * self.l_lp;
        let c = self.l_ap2.process(self.l_lp * dec, dd2, 0.0);
        self.l_d2.push(c);
        let l_out = self.l_d2.read(tap(3720.0));

        let a = self.r_mod_ap.process(r_in, -0.7, m_r);
        self.r_d1.push(a);
        let b = self.r_d1.read(tap(4217.0));
        self.r_lp = (1.0 - self.damping) * b + self.damping * self.r_lp;
        let c = self.r_ap2.process(self.r_lp * dec, dd2, 0.0);
        self.r_d2.push(c);
        let r_out = self.r_d2.read(tap(3163.0));

        self.l_fb = l_out;
        self.r_fb = r_out;

        // output taps (per the paper's table)
        let yl = self.r_d1.read(tap(266.0)) + self.r_d1.read(tap(2974.0))
            - self.r_ap2.d.read(tap(1913.0))
            + self.r_d2.read(tap(1996.0))
            - self.l_d1.read(tap(1990.0))
            - self.l_ap2.d.read(tap(187.0))
            - self.l_d2.read(tap(1066.0));
        let yr = self.l_d1.read(tap(353.0)) + self.l_d1.read(tap(3627.0))
            - self.l_ap2.d.read(tap(1228.0))
            + self.l_d2.read(tap(2673.0))
            - self.r_d1.read(tap(2111.0))
            - self.r_ap2.d.read(tap(335.0))
            - self.r_d2.read(tap(121.0));

        let wet = self.mix;
        let dry = 1.0 - 0.4 * wet;
        (in_l * dry + yl * 0.6 * wet * 1.5, in_r * dry + yr * 0.6 * wet * 1.5)
    }
}
