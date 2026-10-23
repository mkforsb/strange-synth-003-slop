//! WebAudio plumbing: AudioContext + AudioWorklet hosting the Rust DSP wasm,
//! plus analysers that feed the visuals.

use js_sys::{Array, Object, Reflect, WebAssembly};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AnalyserNode, AudioContext, AudioWorkletNode, AudioWorkletNodeOptions, Response};

pub struct Audio {
    #[allow(dead_code)]
    pub ctx: AudioContext,
    node: AudioWorkletNode,
    pub left: AnalyserNode,
    pub right: AnalyserNode,
    pub mono: AnalyserNode,
}

thread_local! {
    static AUDIO: RefCell<Option<Audio>> = const { RefCell::new(None) };
}

pub fn is_running() -> bool {
    AUDIO.with(|a| a.borrow().is_some())
}

pub fn with<R>(f: impl FnOnce(&Audio) -> R) -> Option<R> {
    AUDIO.with(|a| a.borrow().as_ref().map(f))
}

fn analyser(ctx: &AudioContext, fft: u32, smoothing: f64) -> Result<AnalyserNode, JsValue> {
    let a = ctx.create_analyser()?;
    a.set_fft_size(fft);
    a.set_smoothing_time_constant(smoothing);
    Ok(a)
}

pub async fn start(worklet_url: String, wasm_url: String) -> Result<(), JsValue> {
    if is_running() {
        return Ok(());
    }
    let ctx = AudioContext::new()?;
    let _ = ctx.resume()?;
    JsFuture::from(ctx.audio_worklet()?.add_module(&worklet_url)?).await?;

    let window = web_sys::window().ok_or("no window")?;
    let resp: Response = JsFuture::from(window.fetch_with_str(&wasm_url)).await?.dyn_into()?;
    let bytes = JsFuture::from(resp.array_buffer()?).await?;
    let module = JsFuture::from(WebAssembly::compile(&bytes)).await?;

    let processor_options = Object::new();
    Reflect::set(&processor_options, &"module".into(), &module)?;
    let opts = AudioWorkletNodeOptions::new();
    opts.set_number_of_inputs(0);
    opts.set_number_of_outputs(1);
    opts.set_output_channel_count(&Array::of1(&2.into()));
    opts.set_processor_options(Some(&processor_options));
    let node = AudioWorkletNode::new_with_options(&ctx, "strange-synth", &opts)?;

    let left = analyser(&ctx, 2048, 0.0)?;
    let right = analyser(&ctx, 2048, 0.0)?;
    let mono = analyser(&ctx, 2048, 0.8)?;
    let splitter = ctx.create_channel_splitter_with_number_of_outputs(2)?;
    node.connect_with_audio_node(&splitter)?;
    splitter.connect_with_audio_node_and_output(&left, 0)?;
    splitter.connect_with_audio_node_and_output(&right, 1)?;
    node.connect_with_audio_node(&mono)?;
    node.connect_with_audio_node(&ctx.destination())?;

    AUDIO.with(|a| *a.borrow_mut() = Some(Audio { ctx, node, left, right, mono }));
    Ok(())
}

fn send(msg: &[JsValue]) {
    with(|a| {
        if let Ok(port) = a.node.port() {
            let arr = Array::new();
            for m in msg {
                arr.push(m);
            }
            let _ = port.post_message(&arr);
        }
    });
}

pub fn param(i: usize, v: f32) {
    send(&["p".into(), (i as f64).into(), (v as f64).into()]);
}

pub fn note_on(note: u8, vel: f32) {
    send(&["on".into(), (note as f64).into(), (vel as f64).into()]);
}

pub fn note_off(note: u8) {
    send(&["off".into(), (note as f64).into()]);
}

pub fn hold(on: bool) {
    send(&["hold".into(), (on as i32 as f64).into()]);
}

pub fn panic() {
    send(&["panic".into()]);
}
