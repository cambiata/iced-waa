use std::collections::BTreeMap;
use web_audio_api::context::{AudioContext, AudioContextLatencyCategory, AudioContextOptions, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode, OscillatorNode};

fn main() {
    let context = get_context();
    let mut map: BTreeMap<String, OscillatorNode> = BTreeMap::new();
    let context_time = context.current_time();

    // Schedule 10 oscillators to blip during 10 seconds
    for scheduled_time in [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0] {
        println!("Creating oscillator for time: {}", scheduled_time);

        let mut osc = context.create_oscillator();
        osc.connect(&context.destination());
        osc.frequency().set_value(440.0);
        osc.start_at(context_time + scheduled_time);
        osc.stop_at(context_time + scheduled_time + 0.1);

        osc.set_onended(move |_| {
            println!("Oscillator ended");
        });

        map.insert(format!("{:.2}", scheduled_time), osc);
    }

    // wait for 5 seconds...
    std::thread::sleep(std::time::Duration::from_secs(5));
    println!("Close");

    map.values_mut().for_each(|osc| osc.stop());
    map.clear();

    std::thread::sleep(std::time::Duration::from_secs(5));
}

fn get_context() -> AudioContext {
    let latency_hint = match std::env::var("WEB_AUDIO_LATENCY").as_deref().map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Ok("interactive") => AudioContextLatencyCategory::Interactive,
        Ok("balanced") => AudioContextLatencyCategory::Balanced,
        Ok("playback") => AudioContextLatencyCategory::Playback,
        _ => AudioContextLatencyCategory::default(),
    };

    AudioContext::new(AudioContextOptions {
        latency_hint,
        ..AudioContextOptions::default()
    })
}
