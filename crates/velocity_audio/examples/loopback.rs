//! port diagnostics: record the default output device's loopback (what actually reaches the endpoint) and print
//! the rms per second. `cargo run -p velocity_audio --example loopback -- <seconds>`

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

fn main() {
    let secs: u64 = std::env::args().nth(1).and_then(|v| v.parse().ok()).unwrap_or(10);
    let host = cpal::default_host();
    let dev = host.default_output_device().expect("no output device");
    let cfg = dev.default_output_config().expect("no output config");
    println!("device {:?} config {:?}", dev.description().ok(), cfg);
    let acc = Arc::new(Mutex::new((0.0f64, 0u64, 0.0f32)));
    let a = acc.clone();
    let stream = dev
        .build_input_stream(
            &cfg.clone().into(),
            move |d: &[f32], _: &cpal::InputCallbackInfo| {
                let mut g = a.lock().unwrap();
                for &s in d {
                    g.0 += (s as f64) * (s as f64);
                    g.2 = g.2.max(s.abs());
                }
                g.1 += d.len() as u64;
            },
            |e| eprintln!("loopback error {e}"),
            None,
        )
        .expect("loopback stream (f32)");
    stream.play().unwrap();
    for s in 1..=secs {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut g = acc.lock().unwrap();
        let rms = if g.1 > 0 { (g.0 / g.1 as f64).sqrt() } else { 0.0 };
        let db = if rms > 0.0 { 20.0 * rms.log10() } else { -999.0 };
        println!("{s} samples {} rms_db {db:.1} peak {:.4}", g.1, g.2);
        *g = (0.0, 0, 0.0);
    }
}
