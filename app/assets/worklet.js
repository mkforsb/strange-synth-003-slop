// AudioWorklet host for the Rust DSP core (strange-dsp, compiled to wasm32).
// The main thread compiles the module and hands it over in processorOptions;
// all messages are small arrays: ['p', index, value], ['on', note, vel], ...
class StrangeSynth extends AudioWorkletProcessor {
  constructor(options) {
    super();
    const module = options.processorOptions.module;
    this.x = new WebAssembly.Instance(module, {}).exports;
    this.eng = this.x.engine_new(sampleRate);
    this.outPtr = this.x.engine_out(this.eng);
    this.mem = null;
    this.view = null;
    this.port.onmessage = (e) => {
      const m = e.data;
      const x = this.x, g = this.eng;
      switch (m[0]) {
        case 'p': x.engine_param(g, m[1], m[2]); break;
        case 'on': x.engine_note_on(g, m[1], m[2]); break;
        case 'off': x.engine_note_off(g, m[1]); break;
        case 'hold': x.engine_hold(g, m[1]); break;
        case 'alloff': x.engine_all_off(g); break;
        case 'panic': x.engine_panic(g); break;
      }
    };
  }

  process(_inputs, outputs) {
    const out = outputs[0];
    const left = out[0], right = out[1] || out[0];
    const total = left.length;
    for (let off = 0; off < total; off += 128) {
      const n = Math.min(128, total - off);
      this.x.engine_process(this.eng, n);
      // wasm memory may grow (detaching old views), so re-wrap when it changes
      if (this.mem !== this.x.memory.buffer) {
        this.mem = this.x.memory.buffer;
        this.view = new Float32Array(this.mem, this.outPtr, 256);
      }
      left.set(this.view.subarray(0, n), off);
      right.set(this.view.subarray(128, 128 + n), off);
    }
    return true;
  }
}
registerProcessor('strange-synth', StrangeSynth);
