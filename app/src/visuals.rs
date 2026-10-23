//! Canvas visuals: a phosphor "orb" (butterfly spectrum ring + XY goniometer +
//! waveform) and a scrolling spectrogram waterfall. Driven by requestAnimationFrame.

use crate::audio;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub const ORB_W: u32 = 1200;
pub const ORB_H: u32 = 280;
pub const FALL_W: u32 = 1200;
pub const FALL_H: u32 = 120;
const FFT: usize = 2048;

struct State {
    #[allow(dead_code)]
    orb: HtmlCanvasElement,
    orb_ctx: CanvasRenderingContext2d,
    fall: HtmlCanvasElement,
    fall_ctx: CanvasRenderingContext2d,
    l: Vec<f32>,
    r: Vec<f32>,
    spec: Vec<u8>,
    energy: f64,
    flash: f64,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
    static STARTED: RefCell<bool> = const { RefCell::new(false) };
}

fn canvas(id: &str) -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let doc = web_sys::window()?.document()?;
    let c: HtmlCanvasElement = doc.get_element_by_id(id)?.dyn_into().ok()?;
    let ctx: CanvasRenderingContext2d = c.get_context("2d").ok()??.dyn_into().ok()?;
    Some((c, ctx))
}

pub fn start() {
    if STARTED.with(|s| std::mem::replace(&mut *s.borrow_mut(), true)) {
        return;
    }
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move |t: f64| {
        frame(t * 0.001);
        if let Some(w) = web_sys::window() {
            let _ = w.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }));
    if let Some(w) = web_sys::window() {
        let _ = w.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }
}

fn frame(t: f64) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.is_none() {
            if let (Some((orb, orb_ctx)), Some((fall, fall_ctx))) = (canvas("orb"), canvas("fall")) {
                orb_ctx.set_fill_style_str("#05030e");
                orb_ctx.fill_rect(0.0, 0.0, ORB_W as f64, ORB_H as f64);
                fall_ctx.set_fill_style_str("#000");
                fall_ctx.fill_rect(0.0, 0.0, FALL_W as f64, FALL_H as f64);
                *s = Some(State {
                    orb,
                    orb_ctx,
                    fall,
                    fall_ctx,
                    l: vec![0.0; FFT],
                    r: vec![0.0; FFT],
                    spec: vec![0; FFT / 2],
                    energy: 0.0,
                    flash: 0.0,
                });
            }
        }
        let Some(st) = s.as_mut() else { return };
        fetch(st, t);
        draw_orb(st, t);
        draw_fall(st);
    });
}

/// Pull analyser data, or synthesise a gentle idle animation before audio starts.
fn fetch(st: &mut State, t: f64) {
    let live = audio::with(|a| {
        a.left.get_float_time_domain_data(&mut st.l);
        a.right.get_float_time_domain_data(&mut st.r);
        a.mono.get_byte_frequency_data(&mut st.spec);
    })
    .is_some();
    if !live {
        for i in 0..FFT {
            let p = i as f64 / FFT as f64 * 2.0 * PI * 3.0;
            st.l[i] = (0.25 * (p + t).sin()) as f32;
            st.r[i] = (0.25 * (p * 1.5 + t * 0.7).cos()) as f32;
        }
        for (i, b) in st.spec.iter_mut().enumerate() {
            let x = i as f64;
            *b = (40.0 + 30.0 * ((x * 0.05 - t * 2.0).sin()) * (-x * 0.004).exp() * 3.0).clamp(0.0, 255.0) as u8;
        }
    }
    let rms = (st.l.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / FFT as f64).sqrt();
    let prev = st.energy;
    st.energy += (rms - st.energy) * 0.2;
    st.flash = (st.flash * 0.9).max(((st.energy - prev) * 18.0).clamp(0.0, 1.0));
}

fn bin_for(frac: f64) -> usize {
    ((2.0 * (500.0f64).powf(frac)) as usize).min(FFT / 2 - 1)
}

fn draw_orb(st: &State, t: f64) {
    let c = &st.orb_ctx;
    let (w, h) = (ORB_W as f64, ORB_H as f64);
    let (cx, cy) = (w / 2.0, h / 2.0);

    c.set_global_composite_operation("source-over").ok();
    c.set_fill_style_str("rgba(5,3,14,0.2)");
    c.fill_rect(0.0, 0.0, w, h);
    c.set_global_composite_operation("lighter").ok();

    // wide waveform ribbon
    c.set_line_width(1.5);
    c.set_stroke_style_str(&format!("hsla({:.0},90%,60%,0.35)", 280.0 + st.flash * 60.0));
    c.begin_path();
    for i in 0..FFT / 2 {
        let x = i as f64 / (FFT / 2) as f64 * w;
        let y = cy + (st.l[i * 2] + st.r[i * 2]) as f64 * 0.5 * h * 0.42;
        if i == 0 {
            c.move_to(x, y);
        } else {
            c.line_to(x, y);
        }
    }
    c.stroke();

    // butterfly spectrum ring
    let n = 120;
    let base = 62.0 + st.energy * 80.0;
    c.set_line_width(3.2);
    c.set_line_cap("round");
    for b in 0..n {
        let frac = b as f64 / n as f64;
        let mag = st.spec[bin_for(frac)] as f64 / 255.0;
        let len = mag.powf(1.6) * 118.0;
        let hue = (190.0 + 170.0 * frac + t * 12.0) % 360.0;
        c.set_stroke_style_str(&format!("hsla({:.0},95%,{:.0}%,0.85)", hue, 45.0 + mag * 25.0));
        let a = frac * PI - PI / 2.0 + (t * 0.1).sin() * 0.05;
        for side in [-1.0, 1.0] {
            let ang = if side > 0.0 { a } else { PI - a };
            let (s, co) = ang.sin_cos();
            c.begin_path();
            c.move_to(cx + co * base, cy + s * base);
            c.line_to(cx + co * (base + 3.0 + len), cy + s * (base + 3.0 + len));
            c.stroke();
        }
    }

    // XY goniometer (mid/side rotated)
    let scale = 78.0 + st.energy * 120.0;
    c.set_line_width(1.6);
    c.set_stroke_style_str(&format!("hsla({:.0},100%,70%,0.9)", 165.0 - st.flash * 100.0));
    c.begin_path();
    for i in (0..FFT).step_by(2) {
        let (l, r) = (st.l[i] as f64, st.r[i] as f64);
        let x = cx + (l - r) * 0.7071 * scale;
        let y = cy - (l + r) * 0.7071 * scale;
        if i == 0 {
            c.move_to(x, y);
        } else {
            c.line_to(x, y);
        }
    }
    c.stroke();

    // core glow
    let glow = 10.0 + st.energy * 260.0 + st.flash * 30.0;
    if let Ok(g) = c.create_radial_gradient(cx, cy, 0.0, cx, cy, glow) {
        let _ = g.add_color_stop(0.0, "rgba(255,255,255,0.35)");
        let _ = g.add_color_stop(0.4, &format!("hsla({:.0},100%,60%,0.16)", 300.0 + st.flash * 40.0));
        let _ = g.add_color_stop(1.0, "rgba(0,0,0,0)");
        #[allow(deprecated)]
        c.set_fill_style(&g);
        c.begin_path();
        let _ = c.arc(cx, cy, glow, 0.0, 2.0 * PI);
        c.fill();
    }
    c.set_global_composite_operation("source-over").ok();
}

fn draw_fall(st: &State) {
    let c = &st.fall_ctx;
    let (w, h) = (FALL_W as f64, FALL_H as f64);
    let _ = c.draw_image_with_html_canvas_element(&st.fall, -2.0, 0.0);
    let rows = FALL_H as usize;
    for y in 0..rows {
        let frac = 1.0 - y as f64 / rows as f64;
        let mag = st.spec[bin_for(frac)] as f64 / 255.0;
        let v = ((mag - 0.12) / 0.88).clamp(0.0, 1.0);
        let col = format!("hsl({:.0},100%,{:.0}%)", 270.0 - v * 230.0, v.powf(0.8) * 62.0);
        c.set_fill_style_str(&col);
        c.fill_rect(w - 2.0, y as f64 * h / rows as f64, 2.0, h / rows as f64 + 1.0);
    }
}
