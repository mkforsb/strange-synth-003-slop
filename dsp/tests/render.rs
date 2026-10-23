use strange_dsp::presets::PRESETS;
use strange_dsp::Engine;

fn render(params: &[f32], note: i32, secs: f32, hold: f32) -> Vec<f32> {
    let sr = 48000.0;
    let mut e = Engine::new(sr);
    for (i, v) in params.iter().enumerate() {
        e.set_param(i, *v);
    }
    // let smoothing settle
    for _ in 0..40 { e.process(128); }
    e.note_on(note, 0.8);
    let total = (secs * sr) as usize / 128;
    let off = (hold * sr) as usize / 128;
    let mut out = Vec::new();
    for b in 0..total {
        if b == off { e.note_off(note); }
        e.process(128);
        for i in 0..128 { out.push(e.out[i]); out.push(e.out[128 + i]); }
    }
    out
}

fn stats(buf: &[f32]) -> (f32, f32, bool) {
    let mut peak = 0.0f32; let mut sum = 0.0f64; let mut ok = true;
    for &x in buf { if !x.is_finite() { ok = false; } peak = peak.max(x.abs()); sum += (x * x) as f64; }
    (peak, (sum / buf.len() as f64).sqrt() as f32, ok)
}

#[test]
fn init_makes_sound() {
    let p = PRESETS[0].to_params();
    let b = render(&p, 48, 1.0, 0.5);
    let (peak, rms, ok) = stats(&b);
    println!("init peak {peak} rms {rms}");
    assert!(ok && rms > 0.01);
}

#[test]
fn all_presets_audit() {
    let mut bad = Vec::new();
    for p in PRESETS {
        let b = render(&p.to_params(), p.note as i32, 3.0, 1.5);
        let (peak, rms, ok) = stats(&b);
        // rms of the first 1.5 s (key held)
        let (_, rms_held, _) = stats(&b[..b.len() / 2]);
        println!("{:48} peak {:5.2} rms {:6.3} held {:6.3} {}", p.name, peak, rms, rms_held, if ok { "" } else { "NaN!" });
        if !ok || rms_held < 0.02 || peak > 1.001 { bad.push(p.name); }
    }
    assert!(bad.is_empty(), "problem presets: {:?}", bad);
}

#[test]
fn random_params_stay_finite() {
    let mut s = 12345u32;
    let mut rnd = || { s ^= s << 13; s ^= s >> 17; s ^= s << 5; (s as f32) / (u32::MAX as f32) };
    for case in 0..150 {
        let p: Vec<f32> = (0..strange_dsp::params::COUNT).map(|_| { let r = rnd(); if r < 0.2 { 0.0 } else if r > 0.8 { 1.0 } else { rnd() } }).collect();
        let b = render(&p, 24 + (rnd() * 70.0) as i32, 1.2, 0.6);
        let (peak, _, ok) = stats(&b);
        assert!(ok && peak <= 1.001, "case {case} peak {peak} ok {ok}");
    }
}

fn window_rms(buf: &[f32]) -> f32 {
    let w = 48000 / 4 * 2;
    let mut best = 0.0f32;
    let mut i = 0;
    while i + w <= buf.len() {
        let (_, r, _) = stats(&buf[i..i + w]);
        best = best.max(r);
        i += w / 2;
    }
    best
}

/// `cargo test --release calibrate -- --ignored --nocapture` prints a vol for each preset.
#[test]
#[ignore]
fn calibrate() {
    for p in PRESETS {
        let base = p.to_params();
        let (mut lo, mut hi) = (0.03f32, 0.95f32);
        for _ in 0..14 {
            let mid = (lo + hi) / 2.0;
            let mut pr = base;
            pr[strange_dsp::params::P::Volume as usize] = mid;
            let b = render(&pr, p.note as i32, 3.0, 1.5);
            let (peak, _, _) = stats(&b);
            if window_rms(&b) > 0.17 || peak > 0.93 { hi = mid } else { lo = mid }
        }
        println!("CAL\t{}\t{:.3}", p.name, lo);
    }
}

#[test]
#[ignore]
fn probe() {
    let name = std::env::var("PRESET").unwrap();
    let p = PRESETS.iter().find(|p| p.name == name).unwrap();
    let base = p.to_params();
    let d = strange_dsp::params::defaults();
    let b = render(&base, p.note as i32, 3.0, 1.5);
    println!("base {:.3}", window_rms(&b));
    for i in 0..base.len() {
        if (base[i] - d[i]).abs() < 1e-4 { continue; }
        let mut q = base; q[i] = d[i];
        let b = render(&q, p.note as i32, 3.0, 1.5);
        println!("reset {:8} -> {:.3}", strange_dsp::params::PARAMS[i].key, window_rms(&b));
    }
}

#[test]
fn rnd_mutations_are_audible_and_safe() {
    use strange_dsp::params::{mutate, COUNT};
    let mut s = 987654321u32;
    let mut rnd = move || { s ^= s << 13; s ^= s >> 17; s ^= s << 5; (s as f32) / (u32::MAX as f32) };
    let mut silent = 0;
    let n = 120;
    for k in 0..n {
        let pre = &PRESETS[k % PRESETS.len()];
        let mut p = pre.to_params().to_vec();
        for _ in 0..3 { p = mutate(&p, &[false; COUNT], &mut rnd); }
        let b = render(&p, pre.note as i32, 2.5, 1.2);
        let (peak, _, ok) = stats(&b);
        assert!(ok && peak <= 1.001);
        let w = window_rms(&b); if w < 0.003 { silent += 1; println!("quiet {w:.4} from {} sus={:.2} dec={:.2} fcut={:.2} ftype={:.2} atk={:.2} lowcut={:.2} hicut={:.2} body={:.2} vol={:.2}", pre.name, p[strange_dsp::params::P::Sustain as usize], p[strange_dsp::params::P::Decay as usize], p[strange_dsp::params::P::Cutoff as usize], p[strange_dsp::params::P::FilterType as usize], p[strange_dsp::params::P::Attack as usize], p[strange_dsp::params::P::LowCut as usize], p[strange_dsp::params::P::HiCut as usize],p[strange_dsp::params::P::Body as usize],p[strange_dsp::params::P::Volume as usize]); }
    }
    println!("silent mutations: {silent}/{n}");
    assert!(silent * 10 < n, "too many silent RND results: {silent}/{n}");
}

#[test]
fn locked_params_survive_mutation() {
    use strange_dsp::params::{mutate, COUNT};
    let mut locks = [false; COUNT];
    for i in (0..COUNT).step_by(3) { locks[i] = true; }
    let cur = PRESETS[5].to_params();
    let mut r = 0.123f32;
    let out = mutate(&cur, &locks, || { r = (r * 7.31 + 0.377).fract(); r });
    for i in 0..COUNT {
        if locks[i] { assert_eq!(out[i], cur[i]); } else { assert!(out[i] != cur[i] || (out[i] - cur[i]).abs() < 1e-6 && false, "param {i} unchanged"); }
    }
}
