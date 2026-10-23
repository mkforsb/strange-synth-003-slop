//! Parameter table shared by the DSP engine and the UI.
//! Every parameter is stored normalised (0..1); `real()` maps to engine units.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Osc,
    Body,
    Voice,
    Motion,
    Mangle,
    Env,
    Filter,
    Eq,
    Reverb,
    Out,
}

impl Group {
    pub const ALL: [Group; 10] = [
        Group::Osc,
        Group::Body,
        Group::Voice,
        Group::Motion,
        Group::Mangle,
        Group::Env,
        Group::Filter,
        Group::Eq,
        Group::Reverb,
        Group::Out,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Group::Osc => "OSCILLATOR",
            Group::Body => "RESONANT BODY",
            Group::Voice => "THROAT",
            Group::Motion => "MOTION & DECAY",
            Group::Mangle => "MANGLER",
            Group::Env => "ENVELOPE",
            Group::Filter => "MASTER FILTER",
            Group::Eq => "EQ",
            Group::Reverb => "DATTORRO PLATE",
            Group::Out => "OUT",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Curve {
    Lin,
    Exp,
    /// Discrete choice with n options (real value is the index).
    Steps(u8),
}

#[derive(Clone, Copy, Debug)]
pub struct ParamDef {
    pub key: &'static str,
    pub name: &'static str,
    pub group: Group,
    pub min: f32,
    pub max: f32,
    pub curve: Curve,
    pub default: f32,
    pub unit: &'static str,
}

macro_rules! params {
    ($( $id:ident, $key:expr, $name:expr, $grp:ident, $min:expr, $max:expr, $curve:expr, $def:expr, $unit:expr; )*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(usize)]
        #[allow(clippy::enum_variant_names)]
        pub enum P { $($id),* }
        pub const PARAMS: &[ParamDef] = &[
            $( ParamDef { key: $key, name: $name, group: Group::$grp, min: $min, max: $max,
                          curve: $curve, default: $def, unit: $unit } ),*
        ];
    };
}

use Curve::*;

params! {
    // ---- oscillator
    Wave,      "wave",     "Wave",       Osc, 0.0, 3.0,   Lin, 0.40, "";
    Warp,      "warp",     "Warp",       Osc, 0.0, 1.0,   Lin, 0.0,  "";
    Fold,      "fold",     "Fold",       Osc, 0.0, 1.0,   Lin, 0.0,  "";
    Feedback,  "fb",       "Feedback",   Osc, 0.0, 2.0,   Lin, 0.0,  "";
    FmRatio,   "fmr",      "FM Ratio",   Osc, 0.25, 16.0, Exp, 0.5,  "x";
    FmDepth,   "fmd",      "FM Depth",   Osc, 0.0, 8.0,   Lin, 0.0,  "";
    Swarm,     "swarm",    "Swarm",      Osc, 0.0, 1.0,   Lin, 0.0,  "";
    Ring,      "ring",     "Ring Mod",   Osc, 0.0, 1.0,   Lin, 0.0,  "";
    RingRatio, "ringr",    "Ring Ratio", Osc, 0.25, 12.0, Exp, 0.5,  "x";
    Chaos,     "chaos",    "Chaos",      Osc, 0.0, 1.0,   Lin, 0.0,  "";
    ChaosR,    "chaosr",   "Chaos r",    Osc, 3.45, 3.999, Lin, 0.7, "";
    Noise,     "noise",    "Noise",      Osc, 0.0, 1.0,   Lin, 0.0,  "";
    // ---- resonant body (8-mode modal bank)
    Body,      "body",     "Body",       Body, 0.0, 1.0,  Lin, 0.0,  "";
    Material,  "mat",      "Material",   Body, 0.0, 4.0,  Lin, 0.5,  "";
    BodyTune,  "btune",    "Tune",       Body, -24.0, 24.0, Lin, 0.5, "st";
    BodyDecay, "bdecay",   "Decay",      Body, 0.05, 15.0, Exp, 0.5, "s";
    BodyDamp,  "bdamp",    "Damping",    Body, 0.0, 1.0,  Lin, 0.3,  "";
    Inharm,    "inharm",   "Inharm",     Body, 0.0, 1.0,  Lin, 0.0,  "";
    Strike,    "strike",   "Strike",     Body, 0.0, 1.0,  Lin, 0.0,  "";
    Bow,       "bow",      "Bow",        Body, 0.0, 1.0,  Lin, 0.0,  "";
    // ---- throat
    Vowel,     "vowel",    "Vowel",      Voice, 0.0, 4.0, Lin, 0.0,  "";
    Formant,   "formant",  "Formant",    Voice, 0.0, 1.0, Lin, 0.0,  "";
    Growl,     "growl",    "Growl",      Voice, 0.0, 1.0, Lin, 0.0,  "";
    GrowlRate, "growlr",   "Growl Rate", Voice, 4.0, 120.0, Exp, 0.5, "Hz";
    // ---- motion
    Dive,      "dive",     "Dive",       Motion, -48.0, 48.0, Lin, 0.5, "st";
    DiveTime,  "divet",    "Dive Time",  Motion, 0.01, 4.0, Exp, 0.4, "s";
    Glide,     "glide",    "Glide",      Motion, 0.002, 2.0, Exp, 0.0, "s";
    Drift,     "drift",    "Drift",      Motion, 0.0, 1.0, Lin, 0.0, "";
    Wobble,    "wobble",   "Wobble",     Motion, 0.0, 1.0, Lin, 0.0, "";
    WobbleRate,"wobr",     "Wob Rate",   Motion, 0.05, 40.0, Exp, 0.5, "Hz";
    WobbleShape,"wobs",    "Wob Shape",  Motion, 0.0, 1.0, Lin, 0.0, "";
    Stutter,   "stutter",  "Stutter",    Motion, 0.0, 1.0, Lin, 0.0, "";
    StutterRate,"stutr",   "Stut Rate",  Motion, 0.5, 40.0, Exp, 0.5, "Hz";
    Erode,     "erode",    "Erode",      Motion, 0.0, 1.0, Lin, 0.0, "";
    // ---- mangler
    Crush,     "crush",    "Crush",      Mangle, 0.0, 1.0, Lin, 0.0, "";
    Decimate,  "decim",    "Decimate",   Mangle, 0.0, 1.0, Lin, 0.0, "";
    // ---- envelope
    Attack,    "atk",      "Attack",     Env, 0.001, 5.0, Exp, 0.1,  "s";
    Decay,     "dec",      "Decay",      Env, 0.02, 10.0, Exp, 0.6,  "s";
    Sustain,   "sus",      "Sustain",    Env, 0.0, 1.0,   Lin, 0.8,  "";
    Release,   "rel",      "Release",    Env, 0.02, 12.0, Exp, 0.4,  "s";
    Spread,    "spread",   "Spread",     Env, 0.0, 1.0,   Lin, 0.5,  "";
    // ---- master filter
    FilterType,"ftype",    "Type",       Filter, 0.0, 4.0, Steps(4), 0.3334, "";
    Cutoff,    "fcut",     "Cutoff",     Filter, 20.0, 20000.0, Exp, 1.0, "Hz";
    Resonance, "fres",     "Resonance",  Filter, 0.0, 1.0, Lin, 0.1, "";
    FilterDrive,"fdrive",  "Drive",      Filter, 0.0, 1.0, Lin, 0.1, "";
    FilterWob, "fwob",     "Filt Wob",   Filter, 0.0, 1.0, Lin, 0.0, "";
    // ---- eq
    LowCut,    "lowcut",   "Low Cut",    Eq, 20.0, 800.0, Exp, 0.0, "Hz";
    LoGain,    "lo",       "Lo",         Eq, -15.0, 15.0, Lin, 0.5, "dB";
    MidGain,   "mid",      "Mid",        Eq, -15.0, 15.0, Lin, 0.5, "dB";
    MidFreq,   "midf",     "Mid Freq",   Eq, 200.0, 5000.0, Exp, 0.5, "Hz";
    HiGain,    "hi",       "Hi",         Eq, -15.0, 15.0, Lin, 0.5, "dB";
    HiCut,     "hicut",    "Hi Cut",     Eq, 1500.0, 20000.0, Exp, 1.0, "Hz";
    // ---- reverb
    RvMix,     "rvmix",    "Mix",        Reverb, 0.0, 1.0, Lin, 0.2, "";
    RvDecay,   "rvdec",    "Decay",      Reverb, 0.0, 1.0, Lin, 0.5, "";
    RvDamp,    "rvdamp",   "Damping",    Reverb, 0.0, 1.0, Lin, 0.4, "";
    RvBandwidth,"rvbw",    "Bandwidth",  Reverb, 0.0, 1.0, Lin, 0.8, "";
    RvPre,     "rvpre",    "Predelay",   Reverb, 0.0, 150.0, Lin, 0.0, "ms";
    RvMod,     "rvmod",    "Mod",        Reverb, 0.0, 1.0, Lin, 0.3, "";
    // ---- out
    Volume,    "vol",      "Volume",     Out, 0.0, 1.0, Lin, 0.45, "";
}

pub const COUNT: usize = PARAMS.len();

pub const FILTER_TYPES: [&str; 4] = ["LP12", "LP24", "BP", "HP"];
pub const VOWELS: [&str; 5] = ["A", "E", "I", "O", "U"];

pub fn index_of(key: &str) -> Option<usize> {
    PARAMS.iter().position(|p| p.key == key)
}

pub fn defaults() -> [f32; COUNT] {
    let mut d = [0.0; COUNT];
    for (i, p) in PARAMS.iter().enumerate() {
        d[i] = p.default;
    }
    d
}

/// Normalised -> engine units.
pub fn real(i: usize, n: f32) -> f32 {
    let p = &PARAMS[i];
    let n = n.clamp(0.0, 1.0);
    match p.curve {
        Lin => p.min + n * (p.max - p.min),
        Exp => p.min * (p.max / p.min).powf(n),
        Steps(k) => (n * (k as f32 - 1.0)).round(),
    }
}

/// Engine units -> normalised.
pub fn normalise(i: usize, v: f32) -> f32 {
    let p = &PARAMS[i];
    match p.curve {
        Lin => (v - p.min) / (p.max - p.min),
        Exp => (v / p.min).ln() / (p.max / p.min).ln(),
        Steps(k) => v / (k as f32 - 1.0),
    }
    .clamp(0.0, 1.0)
}

pub fn is_discrete(i: usize) -> bool {
    matches!(PARAMS[i].curve, Steps(_))
}

/// Range (in normalised units) that RND is allowed to wander in, so a
/// mutation never lands on silence or an ear-splitting level.
pub fn rnd_range(i: usize) -> (f32, f32) {
    match i {
        x if x == P::Cutoff as usize => (0.35, 1.0),
        x if x == P::Volume as usize => (0.08, 0.9),
        x if x == P::Sustain as usize => (0.35, 1.0),
        x if x == P::Attack as usize => (0.0, 0.55),
        x if x == P::Release as usize => (0.1, 0.8),
        x if x == P::Decay as usize => (0.2, 0.9),
        x if x == P::HiCut as usize => (0.5, 1.0),
        x if x == P::LowCut as usize => (0.0, 0.5),
        x if x == P::Resonance as usize => (0.0, 0.85),
        x if x == P::RvMix as usize => (0.0, 0.6),
        x if x == P::RvDecay as usize => (0.1, 0.9),
        x if x == P::BodyDecay as usize => (0.1, 0.9),
        _ => (0.0, 1.0),
    }
}

/// The RND button: mutates every unlocked parameter. Discrete parameters get a
/// fresh random choice, volume a small nudge, everything else either a full
/// re-roll (30%) or a random walk that always moves a noticeable distance.
pub fn mutate(cur: &[f32], locked: &[bool], mut rnd: impl FnMut() -> f32) -> Vec<f32> {
    let mut out = cur.to_vec();
    for i in 0..COUNT {
        if locked[i] {
            continue;
        }
        let (lo, hi) = rnd_range(i);
        let v = if is_discrete(i) {
            rnd()
        } else if PARAMS[i].key == "vol" {
            // drift toward a moderate level so hot/quiet presets don't compound
            cur[i] + (0.28 - cur[i]) * 0.25 + (rnd() - 0.5) * 0.06
        } else if rnd() < 0.3 {
            lo + rnd() * (hi - lo)
        } else {
            let step = (0.12 + rnd() * 0.38) * if rnd() < 0.5 { -1.0 } else { 1.0 } * (hi - lo);
            let n = cur[i] + step;
            if n < lo || n > hi {
                cur[i] - step
            } else {
                n
            }
        };
        out[i] = v.clamp(lo.min(hi), lo.max(hi)).clamp(0.0, 1.0);
    }
    // band/high-pass filters with a high cutoff would silence the synth
    let (ft, fc) = (P::FilterType as usize, P::Cutoff as usize);
    if !locked[fc] {
        let kind = real(ft, out[ft]) as usize;
        let max_cut = match kind {
            2 => 0.75,
            3 => 0.6,
            _ => 1.0,
        };
        out[fc] = out[fc].min(max_cut);
    }
    out
}

pub fn format_value(i: usize, n: f32) -> String {
    let p = &PARAMS[i];
    let v = real(i, n);
    if i == P::FilterType as usize {
        return FILTER_TYPES[v as usize].to_string();
    }
    if i == P::Vowel as usize {
        let k = v.floor().min(3.0) as usize;
        let f = v - k as f32;
        return format!("{}>{} {:.0}%", VOWELS[k], VOWELS[k + 1], f * 100.0);
    }
    if i == P::Material as usize {
        const NAMES: [&str; 5] = ["string", "bar", "sheet", "bell", "pipe"];
        let k = v.round().clamp(0.0, 4.0) as usize;
        return NAMES[k].to_string();
    }
    match p.unit {
        "Hz" => {
            if v >= 1000.0 {
                format!("{:.2}k", v / 1000.0)
            } else if v >= 100.0 {
                format!("{:.0}", v)
            } else {
                format!("{:.1}", v)
            }
        }
        "s" => {
            if v < 1.0 {
                format!("{:.0}ms", v * 1000.0)
            } else {
                format!("{:.2}s", v)
            }
        }
        "st" => format!("{:+.1}", v),
        "dB" => format!("{:+.1}", v),
        "ms" => format!("{:.0}ms", v),
        "x" => format!("{:.2}x", v),
        _ => format!("{:.0}%", n * 100.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_counts() {
        assert!(COUNT >= 30);
        for i in 0..COUNT {
            let d = PARAMS[i].default;
            assert!((0.0..=1.0).contains(&d), "{}", PARAMS[i].key);
            if !is_discrete(i) {
                let r = real(i, d);
                assert!((normalise(i, r) - d).abs() < 1e-4, "{}", PARAMS[i].key);
            }
        }
        let mut keys: Vec<_> = PARAMS.iter().map(|p| p.key).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), COUNT);
    }
}
