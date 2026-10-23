//! Factory presets: sparse lists of (param key, normalised value) over the defaults.
use crate::params::{self, COUNT};

pub struct Preset {
    pub name: &'static str,
    pub cat: &'static str,
    /// MIDI note the preset sounds best on (used by the audition tests/UI hint).
    pub note: u8,
    pub values: &'static [(&'static str, f32)],
}

impl Preset {
    pub fn to_params(&self) -> [f32; COUNT] {
        let mut p = params::defaults();
        for (k, v) in self.values {
            let i = params::index_of(k).unwrap_or_else(|| panic!("unknown param {k} in {}", self.name));
            p[i] = *v;
        }
        p
    }
}


macro_rules! pre {
    ($name:expr, $cat:expr, $note:expr, [$(($k:expr, $v:expr)),* $(,)?]) => {
        Preset { name: $name, cat: $cat, note: $note, values: &[$(($k, $v)),*] }
    };
}

pub const PRESETS: &[Preset] = &[
    pre!("Init", "Basic", 48, [ ("vol", 0.354)]),
    // ---------------------------------------------------------------- wishlist
    pre!("Metal-on-Metal Screeching Industrial Elevator", "Industrial", 45, [
        ("wave", 0.67), ("fb", 0.5), ("ring", 0.4), ("ringr", 0.62), ("noise", 0.12),
        ("body", 0.85), ("mat", 0.5), ("btune", 0.55), ("bdecay", 0.8), ("bdamp", 0.15),
        ("inharm", 0.6), ("bow", 0.75), ("strike", 0.5), ("dive", 0.56), ("divet", 0.85),
        ("glide", 0.6), ("drift", 0.5), ("wobble", 0.35), ("wobr", 0.35), ("wobs", 0.6),
        ("fwob", 0.2), ("atk", 0.5), ("sus", 1.0), ("rel", 0.7), ("fcut", 0.78), ("fres", 0.45),
        ("fdrive", 0.5), ("crush", 0.12), ("rvmix", 0.3), ("rvdec", 0.7), ("hi", 0.58), ("vol", 0.240)]),
    pre!("Struck Bowed Metal Sheet", "Metal", 50, [
        ("wave", 0.1), ("body", 1.0), ("mat", 0.5), ("bdecay", 0.85), ("bdamp", 0.2),
        ("inharm", 0.3), ("strike", 1.0), ("bow", 0.62), ("atk", 0.3), ("dec", 0.8),
        ("sus", 0.55), ("rel", 0.8), ("drift", 0.2), ("rvmix", 0.35), ("rvdec", 0.65), ("hi", 0.56), ("vol", 0.163)]),
    pre!("UFO Engine Room", "Sci-Fi", 36, [
        ("wave", 0.55), ("swarm", 0.8), ("fmr", 0.35), ("fmd", 0.4), ("fb", 0.25), ("chaos", 0.2),
        ("body", 0.5), ("mat", 1.0), ("bdecay", 0.8), ("bow", 0.3), ("growl", 0.3), ("growlr", 0.35),
        ("wobble", 0.5), ("wobr", 0.55), ("drift", 0.6), ("fwob", 0.45), ("fcut", 0.58), ("fres", 0.5),
        ("atk", 0.7), ("sus", 1.0), ("rel", 0.8), ("rvmix", 0.45), ("rvdec", 0.7), ("vol", 0.760)]),
    pre!("T-Rex Roar", "Creature", 33, [
        ("wave", 0.67), ("fb", 0.6), ("noise", 0.3), ("formant", 0.85), ("vowel", 0.6), ("growl", 0.9),
        ("growlr", 0.52), ("dive", 0.62), ("divet", 0.55), ("drift", 0.7), ("wobble", 0.25), ("wobr", 0.85),
        ("wobs", 0.7), ("crush", 0.25), ("fcut", 0.68), ("fres", 0.3), ("fdrive", 0.8), ("atk", 0.3),
        ("dec", 0.75), ("sus", 0.65), ("rel", 0.6), ("lowcut", 0.3), ("lo", 0.62), ("rvmix", 0.25), ("rvdec", 0.6), ("vol", 0.423)]),
    pre!("Godzilla Laser Beam", "Sci-Fi", 57, [
        ("wave", 0.67), ("fb", 0.8), ("fmr", 0.67), ("fmd", 0.45), ("fold", 0.4), ("swarm", 0.6),
        ("ring", 0.3), ("body", 0.6), ("mat", 0.25), ("bow", 0.5), ("formant", 0.4), ("vowel", 0.5),
        ("growl", 0.5), ("growlr", 0.6), ("dive", 0.88), ("divet", 0.42), ("wobble", 0.3), ("wobr", 0.8),
        ("atk", 0.1), ("sus", 1.0), ("rel", 0.55), ("fcut", 0.85), ("fres", 0.35), ("rvmix", 0.4), ("rvdec", 0.6), ("vol", 0.950)]),
    pre!("Resonating Metal Clothes Rack", "Metal", 55, [
        ("wave", 0.4), ("noise", 0.2), ("body", 1.0), ("mat", 0.3), ("bdecay", 0.8), ("bdamp", 0.2),
        ("inharm", 0.5), ("strike", 1.0), ("atk", 0.0), ("dec", 0.4), ("sus", 0.0), ("rel", 0.6),
        ("rvmix", 0.3), ("rvdec", 0.5), ("hi", 0.6), ("spread", 0.8), ("vol", 0.209)]),
    pre!("Hitting Muted Strings With A Plastic Pipe", "Percussion", 40, [
        ("wave", 0.0), ("noise", 0.1), ("body", 1.0), ("mat", 0.0), ("bdecay", 0.3), ("bdamp", 0.8),
        ("inharm", 0.15), ("strike", 0.9), ("atk", 0.0), ("dec", 0.3), ("sus", 0.0), ("crush", 0.1),
        ("mid", 0.62), ("midf", 0.5), ("rvmix", 0.15), ("rvdec", 0.3), ("vol", 0.314)]),
    pre!("Haunting Singing Glass", "Glass", 69, [
        ("wave", 0.05), ("body", 0.9), ("mat", 0.75), ("btune", 0.75), ("bdecay", 0.8), ("inharm", 0.1),
        ("bow", 0.8), ("formant", 0.3), ("vowel", 0.5), ("drift", 0.6), ("wobble", 0.12), ("wobr", 0.3),
        ("atk", 0.7), ("sus", 1.0), ("rel", 0.9), ("rvmix", 0.6), ("rvdec", 0.85), ("rvdamp", 0.3), ("hi", 0.6), ("vol", 0.679)]),
    pre!("Thoughts of the Machine", "Machine", 52, [
        ("wave", 0.67), ("chaos", 0.7), ("chaosr", 0.85), ("fmd", 0.2), ("crush", 0.5), ("decim", 0.4),
        ("stutter", 0.4), ("stutr", 0.6), ("erode", 0.3), ("wobble", 0.2), ("wobr", 0.5), ("wobs", 1.0),
        ("ftype", 0.6667), ("fcut", 0.6), ("fres", 0.6), ("fwob", 0.6), ("rvmix", 0.5), ("rvdec", 0.6), ("vol", 0.422)]),
    // -------------------------------------------------------------- the rest
    pre!("Cathedral Bell Hammer", "Bell", 48, [
        ("wave", 0.2), ("body", 1.0), ("mat", 0.75), ("bdecay", 0.92), ("bdamp", 0.25), ("strike", 0.9),
        ("atk", 0.0), ("dec", 0.5), ("sus", 0.0), ("rel", 0.8), ("rvmix", 0.5), ("rvdec", 0.85), ("vol", 0.092)]),
    pre!("Tin Can Telephone", "Voice", 55, [
        ("wave", 0.67), ("noise", 0.1), ("formant", 0.95), ("vowel", 0.45), ("crush", 0.45), ("decim", 0.3),
        ("ftype", 0.6667), ("fcut", 0.62), ("fres", 0.45), ("atk", 0.2), ("sus", 0.9), ("rvmix", 0.1), ("vol", 0.480)]),
    pre!("Rust Choir", "Voice", 52, [
        ("wave", 0.67), ("swarm", 0.9), ("formant", 0.9), ("vowel", 0.3), ("drift", 0.8), ("noise", 0.05),
        ("wobble", 0.1), ("wobr", 0.4), ("atk", 0.6), ("sus", 1.0), ("rel", 0.75), ("fcut", 0.8),
        ("rvmix", 0.45), ("rvdec", 0.75), ("spread", 1.0), ("vol", 0.420)]),
    pre!("Dying Hard Drive", "Machine", 43, [
        ("wave", 1.0), ("fb", 0.4), ("decim", 0.55), ("crush", 0.4), ("stutter", 0.8), ("stutr", 0.55),
        ("erode", 0.5), ("dive", 0.3), ("divet", 0.8), ("glide", 0.7), ("atk", 0.2), ("sus", 1.0), ("rel", 0.5),
        ("body", 0.4), ("mat", 0.5), ("bdecay", 0.4), ("fcut", 0.7), ("fres", 0.4), ("rvmix", 0.15), ("vol", 0.396)]),
    pre!("Alien Whale Song", "Creature", 62, [
        ("wave", 0.05), ("fb", 0.3), ("fmr", 0.5), ("fmd", 0.2), ("formant", 0.5), ("vowel", 0.9),
        ("dive", 0.65), ("divet", 0.9), ("glide", 0.8), ("drift", 0.5), ("wobble", 0.3), ("wobr", 0.25),
        ("atk", 0.7), ("sus", 1.0), ("rel", 0.9), ("rvmix", 0.65), ("rvdec", 0.9), ("rvmod", 0.7), ("vol", 0.235)]),
    pre!("Sewer Gong", "Metal", 31, [
        ("wave", 0.3), ("body", 1.0), ("mat", 0.5), ("btune", 0.3), ("bdecay", 0.95), ("bdamp", 0.1),
        ("inharm", 0.7), ("strike", 1.0), ("atk", 0.0), ("dec", 0.6), ("sus", 0.0), ("rel", 0.9),
        ("crush", 0.15), ("fcut", 0.7), ("rvmix", 0.4), ("rvdec", 0.7), ("rvdamp", 0.6), ("vol", 0.223)]),
    pre!("Cardboard Orchestra", "Texture", 55, [
        ("wave", 0.67), ("noise", 0.5), ("swarm", 0.7), ("ring", 0.3), ("ringr", 0.4), ("formant", 0.6),
        ("vowel", 0.7), ("atk", 0.5), ("sus", 0.9), ("rel", 0.6), ("ftype", 0.6667), ("fcut", 0.55), ("fres", 0.2),
        ("rvmix", 0.3), ("lo", 0.4), ("vol", 0.449)]),
    pre!("Insect Swarm", "Creature", 72, [
        ("wave", 1.0), ("swarm", 1.0), ("fmr", 0.9), ("fmd", 0.35), ("stutter", 0.7), ("stutr", 0.75),
        ("erode", 0.6), ("wobble", 0.2), ("wobr", 0.9), ("wobs", 0.8), ("drift", 0.8), ("ftype", 0.6667),
        ("fcut", 0.75), ("fres", 0.5), ("atk", 0.4), ("sus", 1.0), ("rvmix", 0.2), ("spread", 1.0), ("vol", 0.445)]),
    pre!("Welding Torch", "Industrial", 60, [
        ("wave", 0.67), ("noise", 0.8), ("fold", 0.5), ("fb", 0.5), ("erode", 0.5), ("wobble", 0.1), ("wobr", 0.95),
        ("wobs", 1.0), ("ftype", 1.0), ("fcut", 0.7), ("fres", 0.35), ("fdrive", 0.7), ("crush", 0.2), ("atk", 0.1),
        ("sus", 1.0), ("rvmix", 0.15), ("vol", 0.331)]),
    pre!("Submarine Ping", "Sci-Fi", 72, [
        ("wave", 0.0), ("fmr", 0.55), ("fmd", 0.2), ("body", 0.8), ("mat", 1.0), ("bdecay", 0.75), ("strike", 0.4),
        ("dive", 0.46), ("divet", 0.4), ("atk", 0.0), ("dec", 0.5), ("sus", 0.0), ("rel", 0.8), ("rvmix", 0.6),
        ("rvdec", 0.9), ("rvpre", 0.2), ("vol", 0.507)]),
    pre!("Glass Harmonica Fog", "Glass", 64, [
        ("wave", 0.0), ("swarm", 0.5), ("body", 0.7), ("mat", 0.75), ("btune", 0.5), ("bdecay", 0.85), ("bow", 0.6),
        ("drift", 0.8), ("atk", 0.8), ("sus", 1.0), ("rel", 0.9), ("rvmix", 0.7), ("rvdec", 0.9), ("rvdamp", 0.5),
        ("spread", 1.0), ("vol", 0.429)]),
    pre!("Clockwork Heart", "Machine", 38, [
        ("wave", 0.3), ("body", 0.9), ("mat", 0.0), ("bdecay", 0.35), ("bdamp", 0.5), ("strike", 0.8),
        ("stutter", 0.9), ("stutr", 0.45), ("atk", 0.0), ("dec", 0.5), ("sus", 0.8), ("rel", 0.5), ("fcut", 0.6),
        ("rvmix", 0.2), ("vol", 0.249)]),
    pre!("Wet Concrete Bass", "Bass", 31, [
        ("wave", 0.6), ("warp", 0.6), ("fold", 0.35), ("fb", 0.3), ("body", 0.3), ("mat", 0.5), ("bdecay", 0.3),
        ("fcut", 0.5), ("fres", 0.4), ("fdrive", 0.7), ("atk", 0.05), ("dec", 0.5), ("sus", 0.7), ("rel", 0.3),
        ("lo", 0.65), ("rvmix", 0.08), ("vol", 0.273)]),
    pre!("Subway Brakes", "Industrial", 65, [
        ("wave", 0.67), ("noise", 0.3), ("ring", 0.5), ("ringr", 0.7), ("formant", 0.4), ("vowel", 0.4),
        ("body", 0.6), ("mat", 0.5), ("bow", 0.8), ("bdecay", 0.6), ("dive", 0.3), ("divet", 0.85), ("glide", 0.7),
        ("atk", 0.4), ("sus", 1.0), ("rel", 0.6), ("fcut", 0.8), ("fres", 0.4), ("rvmix", 0.3), ("vol", 0.751)]),
    pre!("Radio From Hell", "Texture", 55, [
        ("wave", 0.67), ("chaos", 0.5), ("chaosr", 0.9), ("noise", 0.4), ("formant", 0.6), ("vowel", 0.5),
        ("decim", 0.5), ("crush", 0.35), ("erode", 0.4), ("wobble", 0.1), ("wobs", 1.0), ("wobr", 0.5),
        ("ftype", 0.6667), ("fcut", 0.62), ("fres", 0.5), ("fwob", 0.4), ("rvmix", 0.35), ("rvdec", 0.7), ("vol", 0.512)]),
    pre!("Rubber Band Cathedral", "Strange Keys", 50, [
        ("wave", 0.67), ("warp", 0.8), ("body", 0.9), ("mat", 0.0), ("bdecay", 0.6), ("bdamp", 0.3), ("strike", 0.5),
        ("dive", 0.3), ("divet", 0.35), ("atk", 0.0), ("dec", 0.5), ("sus", 0.3), ("rel", 0.6), ("rvmix", 0.55),
        ("rvdec", 0.85), ("vol", 0.177)]),
    pre!("Iron Lung", "Machine", 38, [
        ("wave", 0.5), ("noise", 0.35), ("formant", 0.7), ("vowel", 0.8), ("growl", 0.7), ("growlr", 0.25),
        ("body", 0.4), ("mat", 1.0), ("bdecay", 0.4), ("stutter", 0.3), ("stutr", 0.3), ("atk", 0.5), ("sus", 1.0),
        ("rel", 0.5), ("fcut", 0.55), ("fres", 0.3), ("rvmix", 0.3), ("vol", 0.787)]),
    pre!("Moth Wing Bells", "Bell", 84, [
        ("wave", 0.0), ("body", 1.0), ("mat", 0.75), ("btune", 0.6), ("bdecay", 0.5), ("strike", 0.8), ("atk", 0.0),
        ("dec", 0.3), ("sus", 0.0), ("rel", 0.5), ("erode", 0.2), ("rvmix", 0.5), ("rvdec", 0.8), ("hi", 0.6),
        ("spread", 1.0), ("vol", 0.437)]),
    pre!("Hydraulic Press", "Industrial", 31, [
        ("wave", 0.67), ("fb", 0.5), ("fold", 0.3), ("noise", 0.25), ("body", 0.7), ("mat", 0.5), ("btune", 0.4),
        ("bdecay", 0.6), ("strike", 1.0), ("dive", 0.2), ("divet", 0.5), ("atk", 0.0), ("dec", 0.55), ("sus", 0.0),
        ("crush", 0.2), ("fcut", 0.55), ("fdrive", 0.8), ("lo", 0.7), ("rvmix", 0.2), ("vol", 0.124)]),
    pre!("Drowned Piano", "Strange Keys", 52, [
        ("wave", 0.2), ("swarm", 0.4), ("body", 1.0), ("mat", 0.0), ("bdecay", 0.75), ("bdamp", 0.5), ("strike", 0.7),
        ("inharm", 0.2), ("drift", 0.6), ("wobble", 0.15), ("wobr", 0.2), ("atk", 0.0), ("dec", 0.6), ("sus", 0.0),
        ("rel", 0.7), ("fcut", 0.45), ("fres", 0.2), ("rvmix", 0.6), ("rvdec", 0.85), ("rvdamp", 0.7), ("vol", 0.228)]),
    pre!("Plasma Harp", "Sci-Fi", 64, [
        ("wave", 0.0), ("fmr", 0.8), ("fmd", 0.3), ("fb", 0.2), ("body", 0.8), ("mat", 1.0), ("bdecay", 0.6),
        ("strike", 0.9), ("atk", 0.0), ("dec", 0.4), ("sus", 0.0), ("rel", 0.5), ("fcut", 0.9), ("rvmix", 0.45),
        ("rvdec", 0.75), ("spread", 0.9), ("vol", 0.300)]),
    pre!("Bone Flute", "Voice", 67, [
        ("wave", 0.0), ("noise", 0.3), ("body", 0.9), ("mat", 1.0), ("bdecay", 0.5), ("bow", 0.55), ("formant", 0.3),
        ("vowel", 0.7), ("drift", 0.4), ("wobble", 0.1), ("wobr", 0.5), ("atk", 0.5), ("sus", 1.0), ("rel", 0.5),
        ("rvmix", 0.4), ("vol", 0.308)]),
    pre!("Rattlesnake Tail", "Creature", 70, [
        ("wave", 1.0), ("noise", 0.9), ("stutter", 1.0), ("stutr", 0.95), ("erode", 0.6), ("ftype", 1.0),
        ("fcut", 0.8), ("fres", 0.3), ("atk", 0.3), ("sus", 1.0), ("rel", 0.4), ("rvmix", 0.15), ("vol", 0.646)]),
    pre!("Cathedral of Static", "Texture", 48, [
        ("wave", 0.67), ("noise", 0.7), ("body", 0.9), ("mat", 0.75), ("bdecay", 0.9), ("bow", 0.4), ("chaos", 0.3),
        ("atk", 0.8), ("sus", 1.0), ("rel", 0.9), ("fcut", 0.7), ("rvmix", 0.75), ("rvdec", 0.95), ("rvdamp", 0.3), ("vol", 0.506)]),
    pre!("Vacuum Tube Sunrise", "Pad", 48, [
        ("wave", 0.45), ("swarm", 0.7), ("drift", 0.6), ("fb", 0.1), ("atk", 0.75), ("dec", 0.8), ("sus", 0.9),
        ("rel", 0.85), ("fcut", 0.6), ("fres", 0.15), ("wobble", 0.06), ("wobr", 0.2), ("rvmix", 0.4), ("rvdec", 0.8),
        ("spread", 1.0), ("vol", 0.419)]),
    pre!("Frozen Pipe Organ", "Pad", 36, [
        ("wave", 0.67), ("body", 0.8), ("mat", 1.0), ("bdecay", 0.7), ("bow", 0.35), ("swarm", 0.3), ("atk", 0.45),
        ("sus", 1.0), ("rel", 0.7), ("fcut", 0.65), ("rvmix", 0.55), ("rvdec", 0.9), ("lo", 0.6), ("vol", 0.316)]),
    pre!("Broken Music Box", "Bell", 76, [
        ("wave", 0.0), ("body", 1.0), ("mat", 0.75), ("btune", 0.6), ("bdecay", 0.45), ("strike", 0.7), ("atk", 0.0),
        ("dec", 0.3), ("sus", 0.0), ("crush", 0.45), ("decim", 0.25), ("erode", 0.3), ("drift", 0.8), ("glide", 0.6),
        ("rvmix", 0.4), ("rvdec", 0.7), ("vol", 0.476)]),
    pre!("Kraken Wakes", "Creature", 29, [
        ("wave", 0.5), ("fb", 0.5), ("noise", 0.4), ("formant", 0.85), ("vowel", 0.95), ("growl", 0.8),
        ("growlr", 0.3), ("dive", 0.42), ("divet", 0.9), ("glide", 0.8), ("body", 0.5), ("mat", 0.5), ("btune", 0.2),
        ("bdecay", 0.8), ("bow", 0.4), ("atk", 0.7), ("sus", 1.0), ("rel", 0.8), ("fcut", 0.45), ("fres", 0.4),
        ("lo", 0.7), ("rvmix", 0.4), ("rvdec", 0.8), ("vol", 0.950)]),
    pre!("Lunar Rover", "Machine", 60, [
        ("wave", 1.0), ("fmr", 0.6), ("fmd", 0.4), ("decim", 0.5), ("crush", 0.3), ("dive", 0.4), ("divet", 0.3),
        ("atk", 0.0), ("dec", 0.4), ("sus", 0.3), ("rel", 0.4), ("ftype", 0.0), ("fcut", 0.7), ("rvmix", 0.3),
        ("rvpre", 0.3), ("vol", 0.340)]),
    pre!("Asteroid Belt", "Sci-Fi", 43, [
        ("wave", 0.67), ("noise", 0.5), ("body", 0.9), ("mat", 0.5), ("bdecay", 0.85), ("inharm", 0.8), ("bow", 0.5),
        ("erode", 0.5), ("wobble", 0.3), ("wobr", 0.4), ("wobs", 1.0), ("fwob", 0.5), ("atk", 0.7), ("sus", 1.0),
        ("rel", 0.8), ("fcut", 0.7), ("fres", 0.4), ("rvmix", 0.5), ("rvdec", 0.8), ("vol", 0.424)]),
    pre!("Gamelan Short Circuit", "Percussion", 60, [
        ("wave", 0.2), ("fmr", 0.7), ("fmd", 0.2), ("body", 1.0), ("mat", 0.4), ("bdecay", 0.5), ("inharm", 0.6),
        ("strike", 0.8), ("atk", 0.0), ("dec", 0.35), ("sus", 0.0), ("rel", 0.5), ("crush", 0.2), ("rvmix", 0.35),
        ("rvdec", 0.6), ("spread", 0.9), ("vol", 0.302)]),
    pre!("Bees In The Chimney", "Creature", 45, [
        ("wave", 0.67), ("swarm", 0.9), ("body", 0.7), ("mat", 1.0), ("bdecay", 0.6), ("bow", 0.4), ("growl", 0.6),
        ("growlr", 0.75), ("drift", 0.9), ("atk", 0.4), ("sus", 1.0), ("rel", 0.5), ("fcut", 0.65), ("fres", 0.35),
        ("rvmix", 0.3), ("vol", 0.689)]),
    pre!("Spring Reverb Poltergeist", "Texture", 55, [
        ("wave", 0.2), ("body", 1.0), ("mat", 0.0), ("bdecay", 0.8), ("bdamp", 0.6), ("inharm", 0.9), ("strike", 1.0),
        ("dive", 0.75), ("divet", 0.35), ("atk", 0.0), ("dec", 0.5), ("sus", 0.0), ("rel", 0.8), ("rvmix", 0.7),
        ("rvdec", 0.85), ("rvmod", 0.9), ("vol", 0.153)]),
    pre!("Fax Machine Opera", "Machine", 74, [
        ("wave", 0.0), ("fmr", 0.85), ("fmd", 0.6), ("fb", 0.3), ("formant", 0.5), ("vowel", 0.2), ("crush", 0.3),
        ("wobble", 0.2), ("wobr", 0.7), ("wobs", 1.0), ("stutter", 0.5), ("stutr", 0.6), ("atk", 0.1), ("sus", 1.0),
        ("ftype", 0.6667), ("fcut", 0.8), ("fres", 0.3), ("rvmix", 0.2), ("vol", 0.419)]),
    pre!("Robot Gargle", "Voice", 43, [
        ("wave", 0.67), ("ring", 0.4), ("ringr", 0.45), ("formant", 0.9), ("vowel", 0.15), ("growl", 1.0),
        ("growlr", 0.7), ("crush", 0.2), ("atk", 0.1), ("sus", 1.0), ("fcut", 0.7), ("fres", 0.3), ("rvmix", 0.2), ("vol", 0.665)]),
    pre!("Tectonic Plates", "Drone", 24, [
        ("wave", 0.4), ("noise", 0.45), ("body", 0.8), ("mat", 0.5), ("btune", 0.3), ("bdecay", 0.9), ("bow", 0.6),
        ("inharm", 0.5), ("drift", 0.8), ("wobble", 0.2), ("wobr", 0.1), ("atk", 0.9), ("sus", 1.0), ("rel", 0.95),
        ("fcut", 0.5), ("fres", 0.3), ("lo", 0.8), ("rvmix", 0.5), ("rvdec", 0.9), ("vol", 0.595)]),
    pre!("Swamp Cello", "Strange Keys", 36, [
        ("wave", 0.67), ("body", 0.8), ("mat", 0.0), ("bdecay", 0.6), ("bdamp", 0.4), ("bow", 0.7), ("formant", 0.5),
        ("vowel", 0.7), ("drift", 0.6), ("wobble", 0.15), ("wobr", 0.45), ("atk", 0.55), ("sus", 1.0),
        ("rel", 0.6), ("fcut", 0.6), ("fres", 0.25), ("rvmix", 0.35), ("rvdec", 0.7), ("vol", 0.531)]),
    pre!("Scrap Yard Wind Chimes", "Metal", 79, [
        ("wave", 0.3), ("swarm", 0.5), ("body", 1.0), ("mat", 0.3), ("btune", 0.6), ("bdecay", 0.7), ("inharm", 0.5),
        ("strike", 1.0), ("erode", 0.3), ("drift", 0.5), ("atk", 0.0), ("dec", 0.4), ("sus", 0.0), ("rel", 0.7),
        ("rvmix", 0.4), ("rvdec", 0.65), ("hi", 0.6), ("spread", 1.0), ("vol", 0.353)]),
    pre!("Neon Morse", "Sci-Fi", 72, [
        ("wave", 0.0), ("fmr", 0.67), ("fmd", 0.15), ("stutter", 1.0), ("stutr", 0.6), ("glide", 0.6), ("atk", 0.05),
        ("sus", 1.0), ("rel", 0.3), ("ftype", 0.6667), ("fcut", 0.8), ("fres", 0.4), ("rvmix", 0.35), ("rvpre", 0.2), ("vol", 0.282)]),
    pre!("Uranium Lullaby", "Bell", 67, [
        ("wave", 0.0), ("chaos", 0.25), ("chaosr", 0.6), ("body", 1.0), ("mat", 0.75), ("bdecay", 0.6), ("strike", 0.6),
        ("drift", 0.7), ("atk", 0.0), ("dec", 0.4), ("sus", 0.0), ("rel", 0.6), ("fcut", 0.8), ("rvmix", 0.55),
        ("rvdec", 0.8), ("vol", 0.282)]),
    pre!("Reactor Core Meltdown", "Industrial", 36, [
        ("wave", 0.67), ("fb", 0.9), ("fmr", 0.4), ("fmd", 0.7), ("fold", 0.6), ("swarm", 0.8), ("ring", 0.5),
        ("noise", 0.4), ("chaos", 0.3), ("body", 0.7), ("mat", 0.5), ("bow", 0.7), ("growl", 0.5), ("growlr", 0.6),
        ("crush", 0.35), ("erode", 0.3), ("wobble", 0.4), ("wobr", 0.7), ("fwob", 0.5), ("atk", 0.4), ("sus", 1.0),
        ("fcut", 0.7), ("fres", 0.6), ("fdrive", 0.9), ("rvmix", 0.3), ("vol", 0.551)]),
    pre!("Piano Eaten By Moths", "Strange Keys", 55, [
        ("wave", 0.2), ("body", 1.0), ("mat", 0.0), ("bdecay", 0.7), ("bdamp", 0.5), ("strike", 0.8), ("erode", 0.7),
        ("decim", 0.3), ("atk", 0.0), ("dec", 0.6), ("sus", 0.0), ("rel", 0.6), ("fcut", 0.55), ("rvmix", 0.5),
        ("rvdec", 0.8), ("vol", 0.299)]),
    pre!("Whalebone Marimba", "Percussion", 60, [
        ("wave", 0.1), ("body", 1.0), ("mat", 0.25), ("bdecay", 0.4), ("bdamp", 0.6), ("strike", 0.9), ("atk", 0.0),
        ("dec", 0.3), ("sus", 0.0), ("rel", 0.4), ("rvmix", 0.25), ("rvdec", 0.5), ("vol", 0.324)]),
    pre!("Cosmic Zipper", "Sci-Fi", 55, [
        ("wave", 0.67), ("ring", 0.6), ("ringr", 0.8), ("dive", 0.0), ("divet", 0.55), ("fmd", 0.2), ("atk", 0.0),
        ("dec", 0.5), ("sus", 0.0), ("rel", 0.4), ("ftype", 0.6667), ("fcut", 0.8), ("fres", 0.5), ("rvmix", 0.35),
        ("rvdec", 0.6), ("vol", 0.925)]),
    pre!("Elevator To Nowhere", "Pad", 52, [
        ("wave", 0.35), ("swarm", 0.6), ("drift", 0.5), ("wobble", 0.18), ("wobr", 0.3), ("wobs", 0.0), ("atk", 0.5),
        ("sus", 0.9), ("rel", 0.6), ("fcut", 0.55), ("fres", 0.2), ("crush", 0.1), ("rvmix", 0.4), ("spread", 1.0), ("vol", 0.401)]),
    pre!("Molten Glass Bass", "Bass", 33, [
        ("wave", 0.2), ("fb", 0.7), ("fold", 0.5), ("body", 0.5), ("mat", 0.75), ("btune", 0.25), ("bdecay", 0.5),
        ("strike", 0.5), ("atk", 0.0), ("dec", 0.5), ("sus", 0.7), ("rel", 0.3), ("fcut", 0.5), ("fres", 0.5),
        ("fdrive", 0.5), ("lo", 0.62), ("rvmix", 0.15), ("vol", 0.176)]),
    pre!("Cryo Chamber Door", "Industrial", 40, [
        ("wave", 0.67), ("noise", 0.5), ("body", 0.8), ("mat", 0.5), ("bdecay", 0.7), ("bow", 0.5), ("dive", 0.3),
        ("divet", 0.8), ("atk", 0.3), ("dec", 0.7), ("sus", 0.3), ("rel", 0.8), ("ftype", 1.0), ("fcut", 0.42),
        ("fres", 0.4), ("hi", 0.6), ("rvmix", 0.5), ("rvdec", 0.8), ("vol", 0.950)]),
    pre!("Siren From Another Dimension", "Sci-Fi", 69, [
        ("wave", 0.5), ("fb", 0.4), ("fmd", 0.3), ("fmr", 0.5), ("wobble", 0.6), ("wobr", 0.55), ("wobs", 0.0),
        ("formant", 0.4), ("vowel", 0.4), ("body", 0.4), ("mat", 0.75), ("bow", 0.3), ("glide", 0.7), ("atk", 0.4),
        ("sus", 1.0), ("rel", 0.7), ("fcut", 0.8), ("fres", 0.3), ("rvmix", 0.55), ("rvdec", 0.85), ("vol", 0.547)]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enough_unique_presets_with_valid_keys() {
        assert!(PRESETS.len() >= 50, "{}", PRESETS.len());
        let mut names: Vec<_> = PRESETS.iter().map(|p| p.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), PRESETS.len());
        for p in PRESETS {
            let v = p.to_params();
            assert!(v.iter().all(|x| (0.0..=1.0).contains(x)), "{}", p.name);
        }
    }
}
