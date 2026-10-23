//! Strange synth DSP core. Compiled to wasm32 and driven from an AudioWorklet
//! through the plain C ABI exports at the bottom, and as an rlib for the UI's
//! parameter/preset tables and for native tests.

pub mod engine;
pub mod filters;
pub mod params;
pub mod presets;
pub mod reverb;
pub mod voice;

pub use engine::Engine;

// ---- C ABI for the AudioWorklet ------------------------------------------

#[no_mangle]
pub extern "C" fn engine_new(sample_rate: f32) -> *mut Engine {
    Box::into_raw(Box::new(Engine::new(sample_rate)))
}

/// Pointer to the planar stereo output block (128 left then 128 right).
///
/// # Safety
/// `e` must come from `engine_new`.
#[no_mangle]
pub unsafe extern "C" fn engine_out(e: *mut Engine) -> *const f32 {
    (*e).out.as_ptr()
}

/// # Safety
/// `e` must come from `engine_new`.
#[no_mangle]
pub unsafe extern "C" fn engine_process(e: *mut Engine, n: u32) {
    (*e).process(n as usize)
}

/// # Safety
/// `e` must come from `engine_new`.
#[no_mangle]
pub unsafe extern "C" fn engine_param(e: *mut Engine, i: u32, v: f32) {
    (*e).set_param(i as usize, v)
}

/// # Safety
/// `e` must come from `engine_new`.
#[no_mangle]
pub unsafe extern "C" fn engine_note_on(e: *mut Engine, note: i32, vel: f32) {
    (*e).note_on(note, vel)
}

/// # Safety
/// `e` must come from `engine_new`.
#[no_mangle]
pub unsafe extern "C" fn engine_note_off(e: *mut Engine, note: i32) {
    (*e).note_off(note)
}

/// # Safety
/// `e` must come from `engine_new`.
#[no_mangle]
pub unsafe extern "C" fn engine_hold(e: *mut Engine, on: i32) {
    (*e).set_hold(on != 0)
}

/// # Safety
/// `e` must come from `engine_new`.
#[no_mangle]
pub unsafe extern "C" fn engine_all_off(e: *mut Engine) {
    (*e).all_off()
}

/// # Safety
/// `e` must come from `engine_new`.
#[no_mangle]
pub unsafe extern "C" fn engine_panic(e: *mut Engine) {
    (*e).panic()
}
