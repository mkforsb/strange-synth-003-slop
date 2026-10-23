mod audio;
mod visuals;

use dioxus::prelude::*;
use strange_dsp::params::{self, Group, COUNT, PARAMS};
use strange_dsp::presets::PRESETS;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const WORKLET_JS: Asset = asset!("/assets/worklet.js");
const DSP_WASM: Asset = asset!("/assets/dsp.wasm");

/// Asset URL relative to the page, so the bundle works from any subfolder (Dioxus always emits `/assets/...`).
fn asset_url(a: Asset) -> String {
    a.to_string().trim_start_matches('/').to_string()
}

/// Preset loaded on startup.
const START_PRESET: usize = 2;

#[derive(Clone, Copy, PartialEq)]
struct Drag {
    idx: usize,
    y0: f64,
    v0: f32,
}

/// All UI state, shared through context. Every field is a (Copy) signal.
#[derive(Clone, Copy)]
struct Synth {
    params: Signal<Vec<f32>>,
    locks: Signal<Vec<bool>>,
    preset: Signal<Option<usize>>,
    hold: Signal<bool>,
    audition: Signal<bool>,
    drag: Signal<Option<Drag>>,
    held: Signal<Vec<u8>>,
    octave: Signal<i32>,
    running: Signal<bool>,
    status: Signal<String>,
}

fn rand() -> f32 {
    js_sys::Math::random() as f32
}

impl Synth {
    fn cur_preset(&self) -> Option<usize> {
        *self.preset.read()
    }

    fn set_param(&self, i: usize, v: f32) {
        let v = v.clamp(0.0, 1.0);
        let mut p = self.params;
        p.write()[i] = v;
        audio::param(i, v);
    }

    fn toggle_lock(&self, i: usize) {
        let mut l = self.locks;
        let mut l = l.write();
        l[i] = !l[i];
    }

    fn is_locked(&self, i: usize) -> bool {
        self.locks.read()[i]
    }

    fn lock_count(&self) -> usize {
        self.locks.read().iter().filter(|l| **l).count()
    }

    fn unlock_all(&self) {
        let mut l = self.locks;
        l.write().iter_mut().for_each(|l| *l = false);
    }

    fn push_all(&self) {
        for (i, v) in self.params.read().iter().enumerate() {
            audio::param(i, *v);
        }
    }

    /// Loads a preset into every unlocked parameter.
    fn load_preset(&self, idx: usize) {
        let idx = idx % PRESETS.len();
        let p = PRESETS[idx].to_params();
        for (i, v) in p.iter().enumerate() {
            if !self.is_locked(i) {
                self.set_param(i, *v);
            }
        }
        let mut pr = self.preset;
        pr.set(Some(idx));
        if *self.audition.read() {
            self.play_for(PRESETS[idx].note, 900);
        }
    }

    fn load_init(&self) {
        let d = params::defaults();
        for (i, v) in d.iter().enumerate() {
            if !self.is_locked(i) {
                self.set_param(i, *v);
            }
        }
        let mut pr = self.preset;
        pr.set(Some(0));
    }

    /// RND: mutate every unlocked parameter (see `params::mutate`).
    fn randomize(&self) {
        let next = params::mutate(&self.params.read(), &self.locks.read(), rand);
        for (i, v) in next.iter().enumerate() {
            self.set_param(i, *v);
        }
        let mut pr = self.preset;
        pr.set(None);
        if *self.audition.read() {
            let note = self.preset_note();
            self.play_for(note, 900);
        }
    }

    fn preset_note(&self) -> u8 {
        self.cur_preset().map(|i| PRESETS[i].note).unwrap_or(48)
    }

    fn play_for(&self, note: u8, ms: u32) {
        let s = *self;
        s.note_on(note);
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(ms).await;
            s.note_off(note);
        });
    }

    fn note_on(&self, note: u8) {
        audio::note_on(note, 0.8);
        let mut h = self.held;
        let mut v = h.write();
        if !v.contains(&note) {
            v.push(note);
        }
    }

    fn note_off(&self, note: u8) {
        audio::note_off(note);
        let mut h = self.held;
        h.write().retain(|n| *n != note);
    }

    fn base_note(&self) -> u8 {
        (12 * (*self.octave.read() + 1)).clamp(0, 108) as u8
    }

    fn preset_label(&self) -> String {
        match self.cur_preset() {
            Some(i) => PRESETS[i].name.to_string(),
            None => "~ mutated ~".to_string(),
        }
    }
}

fn key_to_semitone(key: &str) -> Option<i32> {
    const LOWER: &str = "zsxdcvgbhnjm,l.;/";
    const UPPER: &str = "q2w3er5t6y7ui9o0p";
    if key.chars().count() != 1 {
        return None;
    }
    let c = key.to_lowercase().chars().next()?;
    LOWER
        .chars()
        .position(|k| k == c)
        .map(|p| p as i32)
        .or_else(|| UPPER.chars().position(|k| k == c).map(|p| p as i32 + 12))
}

fn install_keyboard(s: Synth) {
    let Some(window) = web_sys::window() else { return };
    let typing = |e: &web_sys::KeyboardEvent| {
        e.target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
            .map(|el| matches!(el.tag_name().as_str(), "INPUT" | "SELECT" | "TEXTAREA"))
            .unwrap_or(false)
    };
    let down = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        if e.repeat() || e.ctrl_key() || e.meta_key() || e.alt_key() || typing(&e) {
            return;
        }
        let key = e.key();
        if key == "[" || key == "]" {
            let mut o = s.octave;
            let d = if key == "[" { -1 } else { 1 };
            let cur = *o.read();
            o.set((cur + d).clamp(0, 7));
            return;
        }
        if let Some(st) = key_to_semitone(&key) {
            s.note_on((s.base_note() as i32 + st).clamp(0, 127) as u8);
        }
    });
    let up = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        if let Some(st) = key_to_semitone(&e.key()) {
            // release every octave variant so an octave change mid-note can't stick
            let n = (s.base_note() as i32 + st).clamp(0, 127) as u8;
            s.note_off(n);
        }
    });
    let _ = window.add_event_listener_with_callback("keydown", down.as_ref().unchecked_ref());
    let _ = window.add_event_listener_with_callback("keyup", up.as_ref().unchecked_ref());
    down.forget();
    up.forget();
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let initial = PRESETS[START_PRESET].to_params().to_vec();
    let synth = Synth {
        params: use_signal(|| initial.clone()),
        locks: use_signal(|| vec![false; COUNT]),
        preset: use_signal(|| Some(START_PRESET)),
        hold: use_signal(|| false),
        audition: use_signal(|| true),
        drag: use_signal(|| None),
        held: use_signal(Vec::new),
        octave: use_signal(|| 3),
        running: use_signal(|| false),
        status: use_signal(String::new),
    };
    use_context_provider(|| synth);

    use_effect(move || {
        visuals::start();
        install_keyboard(synth);
    });

    rsx! {
        document::Stylesheet { href: asset_url(MAIN_CSS) }
        div {
            class: "app",
            onpointermove: move |e: PointerEvent| {
                let drag = *synth.drag.read();
                if let Some(d) = drag {
                    let dy = d.y0 - e.client_coordinates().y;
                    let sens = if e.modifiers().ctrl() || e.modifiers().alt() { 0.0007 } else { 0.006 };
                    synth.set_param(d.idx, d.v0 + dy as f32 * sens);
                }
            },
            onpointerup: move |_| { let mut d = synth.drag; d.set(None); },
            onpointercancel: move |_| { let mut d = synth.drag; d.set(None); },
            if !*synth.running.read() {
                Overlay {}
            }
            Header {}
            Visuals {}
            div { class: "panels",
                for (gi, g) in Group::ALL.iter().enumerate() {
                    GroupPanel { group: *g, hue: (gi as i32 * 37 + 190) % 360 }
                }
            }
            Keyboard {}
            PresetBrowser {}
            footer {
                "Rust · Dioxus · WebAudio AudioWorklet (wasm DSP). "
                b { "Shift-click" }
                " a knob to lock it against RND and preset changes · drag to turn (Ctrl/Alt = fine) · double-click resets · "
                "keys Z–M / Q–P play, [ ] change octave."
            }
        }
    }
}

#[component]
fn Overlay() -> Element {
    let s = use_context::<Synth>();
    rsx! {
        div { class: "overlay",
            div { class: "overlay-inner",
                h1 { class: "glitch", "STRANGELET" }
                p { "a very strange synth — 59 parameters, 58 presets, one metal elevator" }
                button {
                    class: "enter",
                    onclick: move |_| {
                        let mut running = s.running;
                        let mut status = s.status;
                        spawn(async move {
                            status.set("waking the machine…".into());
                            match audio::start(asset_url(WORKLET_JS), asset_url(DSP_WASM)).await {
                                Ok(()) => {
                                    s.push_all();
                                    running.set(true);
                                    s.play_for(s.preset_note(), 1200);
                                }
                                Err(e) => status.set(format!("audio failed: {e:?}")),
                            }
                        });
                    },
                    "ENTER THE MACHINE"
                }
                p { class: "status", "{s.status}" }
            }
        }
    }
}

#[component]
fn Header() -> Element {
    let s = use_context::<Synth>();
    let cur = s.cur_preset().unwrap_or(0);
    let locks = s.lock_count();
    rsx! {
        header {
            div { class: "logo", span { class: "glitch", "STRANGELET" } small { "a very strange synth" } }
            div { class: "presetbar",
                button { onclick: move |_| s.load_preset(cur + PRESETS.len() - 1), "◀" }
                select {
                    value: "{cur}",
                    onchange: move |e| { if let Ok(i) = e.value().parse::<usize>() { s.load_preset(i); } },
                    for (i, p) in PRESETS.iter().enumerate() {
                        option { value: "{i}", selected: s.cur_preset() == Some(i),
                            "{i + 1:02} · {p.name}"
                        }
                    }
                }
                button { onclick: move |_| s.load_preset(cur + 1), "▶" }
                span { class: "plabel", "{s.preset_label()}" }
            }
            div { class: "actions",
                button { class: "rnd", onclick: move |_| s.randomize(), title: "mutate every unlocked parameter", "RND" }
                button { onclick: move |_| s.load_init(), "INIT" }
                button {
                    class: if *s.hold.read() { "on" } else { "" },
                    onclick: move |_| {
                        let mut h = s.hold;
                        let v = !*h.read();
                        h.set(v);
                        audio::hold(v);
                    },
                    "HOLD"
                }
                button {
                    class: if *s.audition.read() { "on" } else { "" },
                    title: "play a note when choosing a preset or mutating",
                    onclick: move |_| { let mut a = s.audition; let v = !*a.read(); a.set(v); },
                    "AUTO▶"
                }
                button { onclick: move |_| { audio::panic(); let mut h = s.held; h.write().clear(); }, "PANIC" }
                button {
                    class: if locks > 0 { "lockbtn on" } else { "lockbtn" },
                    title: "clear all locks",
                    onclick: move |_| s.unlock_all(),
                    "LOCKS {locks}"
                }
            }
        }
    }
}

#[component]
fn Visuals() -> Element {
    rsx! {
        div { class: "visuals",
            canvas { id: "orb", width: "{visuals::ORB_W}", height: "{visuals::ORB_H}" }
            canvas { id: "fall", width: "{visuals::FALL_W}", height: "{visuals::FALL_H}" }
        }
    }
}

#[component]
fn GroupPanel(group: Group, hue: i32) -> Element {
    let ids: Vec<usize> = (0..COUNT).filter(|i| PARAMS[*i].group == group).collect();
    rsx! {
        section { class: "panel", style: "--h: {hue}",
            h2 { "{group.name()}" }
            div { class: "knobs",
                for i in ids {
                    if params::is_discrete(i) {
                        Choice { key: "{i}", idx: i }
                    } else {
                        Knob { key: "{i}", idx: i }
                    }
                }
            }
        }
    }
}

fn arc_path(a0: f64, a1: f64) -> String {
    let pt = |a: f64| (32.0 + 24.0 * a.to_radians().sin(), 32.0 - 24.0 * a.to_radians().cos());
    let (x0, y0) = pt(a0);
    let (x1, y1) = pt(a1);
    let large = if (a1 - a0).abs() > 180.0 { 1 } else { 0 };
    format!("M {x0:.2} {y0:.2} A 24 24 0 {large} 1 {x1:.2} {y1:.2}")
}

#[component]
fn Knob(idx: usize) -> Element {
    let s = use_context::<Synth>();
    let v = s.params.read()[idx];
    let locked = s.locks.read()[idx];
    let def = &PARAMS[idx];
    let ang = -135.0 + 270.0 * v as f64;
    let (px, py) = (32.0 + 17.0 * ang.to_radians().sin(), 32.0 - 17.0 * ang.to_radians().cos());
    let (qx, qy) = (32.0 + 7.0 * ang.to_radians().sin(), 32.0 - 7.0 * ang.to_radians().cos());
    let value_arc = if v > 0.003 { arc_path(-135.0, ang) } else { String::new() };
    let dragging = (*s.drag.read()).map(|d| d.idx == idx).unwrap_or(false);
    rsx! {
        div {
            class: format_args!("knob{}{}", if locked { " locked" } else { "" }, if dragging { " active" } else { "" }),
            title: "{def.name} — drag · shift-click: lock · double-click: reset",
            onpointerdown: move |e: PointerEvent| {
                e.prevent_default();
                if e.modifiers().shift() {
                    s.toggle_lock(idx);
                } else {
                    let mut d = s.drag;
                    d.set(Some(Drag { idx, y0: e.client_coordinates().y, v0: s.params.read()[idx] }));
                }
            },
            ondoubleclick: move |_| s.set_param(idx, PARAMS[idx].default),
            onwheel: move |e: WheelEvent| {
                e.prevent_default();
                let dy = e.delta().strip_units().y;
                let cur = s.params.read()[idx];
                s.set_param(idx, cur - dy.signum() as f32 * 0.02);
            },
            svg { view_box: "0 0 64 64", width: "60", height: "60",
                path { class: "trk", d: arc_path(-135.0, 135.0), fill: "none", stroke_width: "5", stroke_linecap: "round" }
                if !value_arc.is_empty() {
                    path { class: "val", d: "{value_arc}", fill: "none", stroke_width: "5", stroke_linecap: "round" }
                }
                line { class: "ptr", x1: "{qx:.2}", y1: "{qy:.2}", x2: "{px:.2}", y2: "{py:.2}", stroke_width: "3", stroke_linecap: "round" }
                if locked {
                    rect { class: "lockicon", x: "27", y: "29", width: "10", height: "8", rx: "1.5" }
                    path { class: "lockicon-arc", d: "M 29.5 29 V 26.5 A 2.5 2.5 0 0 1 34.5 26.5 V 29", fill: "none", stroke_width: "1.6" }
                }
            }
            div { class: "kv", "{params::format_value(idx, v)}" }
            div { class: "kn", "{def.name}" }
        }
    }
}

#[component]
fn Choice(idx: usize) -> Element {
    let s = use_context::<Synth>();
    let v = s.params.read()[idx];
    let locked = s.locks.read()[idx];
    let cur = params::real(idx, v) as usize;
    rsx! {
        div { class: if locked { "choice locked" } else { "choice" },
            title: "{PARAMS[idx].name} — shift-click: lock",
            div { class: "opts",
                for (k, name) in params::FILTER_TYPES.iter().enumerate() {
                    button {
                        class: if k == cur { "sel" } else { "" },
                        onpointerdown: move |e: PointerEvent| {
                            if e.modifiers().shift() {
                                s.toggle_lock(idx);
                            } else {
                                s.set_param(idx, k as f32 / (params::FILTER_TYPES.len() - 1) as f32);
                            }
                        },
                        "{name}"
                    }
                }
            }
            div { class: "kn", if locked { "LOCKED · " } "{PARAMS[idx].name}" }
        }
    }
}

#[component]
fn Keyboard() -> Element {
    let s = use_context::<Synth>();
    let base = s.base_note();
    let octave = *s.octave.read();
    let held = s.held.read().clone();
    const WHITE: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
    const BLACK: [(i32, usize); 5] = [(1, 0), (3, 1), (6, 3), (8, 4), (10, 5)];
    let octaves = 3;
    let nwhite = octaves * 7;
    let key_handlers = move |note: u8| {
        (
            move |_e: PointerEvent| s.note_on(note),
            move |_e: PointerEvent| s.note_off(note),
        )
    };
    rsx! {
        div { class: "keyboard-wrap",
            div { class: "oct",
                button { onclick: move |_| { let mut o = s.octave; let c = *o.read(); o.set((c - 1).max(0)); }, "–" }
                span { "OCT {octave}" }
                button { onclick: move |_| { let mut o = s.octave; let c = *o.read(); o.set((c + 1).min(7)); }, "+" }
            }
            div { class: "keyboard",
                for o in 0..octaves {
                    for (wi, st) in WHITE.iter().enumerate() {
                        {
                            let note = (base as i32 + o as i32 * 12 + st).min(127) as u8;
                            let (on, off) = key_handlers(note);
                            rsx! {
                                div {
                                    class: if held.contains(&note) { "wk down" } else { "wk" },
                                    style: "left: {(o * 7 + wi) as f32 * 100.0 / nwhite as f32}%; width: {100.0 / nwhite as f32}%",
                                    onpointerdown: on,
                                    onpointerup: off,
                                    onpointerleave: move |_| s.note_off(note),
                                }
                            }
                        }
                    }
                    for (st, after) in BLACK.iter() {
                        {
                            let note = (base as i32 + o as i32 * 12 + st).min(127) as u8;
                            let (on, off) = key_handlers(note);
                            let w = 100.0 / nwhite as f32;
                            rsx! {
                                div {
                                    class: if held.contains(&note) { "bk down" } else { "bk" },
                                    style: "left: {((o * 7 + after + 1) as f32) * w - w * 0.3}%; width: {w * 0.6}%",
                                    onpointerdown: on,
                                    onpointerup: off,
                                    onpointerleave: move |_| s.note_off(note),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PresetBrowser() -> Element {
    let s = use_context::<Synth>();
    let mut cats: Vec<&str> = Vec::new();
    for p in PRESETS {
        if !cats.contains(&p.cat) {
            cats.push(p.cat);
        }
    }
    rsx! {
        details { class: "browser", open: true,
            summary { "FACTORY PRESETS · {PRESETS.len()}" }
            for cat in cats {
                div { class: "cat",
                    h3 { "{cat}" }
                    div { class: "chips",
                        for (i, p) in PRESETS.iter().enumerate().filter(|(_, p)| p.cat == cat) {
                            button {
                                class: if s.cur_preset() == Some(i) { "chip sel" } else { "chip" },
                                onclick: move |_| s.load_preset(i),
                                "{p.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
