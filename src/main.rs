use std::collections::HashMap;

use iced::Task;
use iced::time::{self};

use iced::widget::{button, column, text};

// use std::sync::mpsc::{Receiver, Sender, channel};

use web_audio_api::node::{AudioDestinationNode, OscillatorNode};
use web_audio_api::{
    context::{AudioContext, AudioContextLatencyCategory, AudioContextOptions, BaseAudioContext},
    node::{AudioNode, AudioScheduledSourceNode},
};

fn main() -> iced::Result {
    iced::application(AudioApp::new, AudioApp::update, AudioApp::view).run()
}

#[derive(Default)]
struct AudioApp {
    audio_context: AudioContext,
    status_info: String,

    osc_map: HashMap<String, OscillatorNode>,
}

#[derive(Debug, Clone)]
enum Message {
    StartPlayback,
    NotifyPlaybackStopped(String),
}

impl AudioApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartPlayback => {
                let (sender, reciever) = std::sync::mpsc::channel::<String>();
                let time = self.audio_context.current_time();

                for i in 1..=9 {
                    let osc = self.create_oscillator(time + i as f64);
                    let sender_clone = sender.clone(); // clone the sender...
                    osc.set_onended(move |_| {
                        if let Err(_) = sender_clone.send(format!("{:.2}", time + i as f64)) {
                            println!("Failed to send playback stopped notification");
                        }
                    });
                }

                return Task::stream(tokio_stream::iter(
                    std::sync::mpsc::Receiver::into_iter(reciever).map(|time_string| Message::NotifyPlaybackStopped(time_string)),
                ));
            }
            Message::NotifyPlaybackStopped(time_string) => {
                println!("Playback has stopped at {}", time_string);
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        column![text(self.status_info.as_str()), button("Play").on_press(Message::StartPlayback),].into()
    }

    fn create_oscillator(&mut self, time: f64) -> OscillatorNode {
        let mut oscillator = self.audio_context.create_oscillator();
        oscillator.connect(&self.audio_context.destination());
        oscillator.frequency().set_value(440.0);
        oscillator.start_at(time);
        oscillator.stop_at(time + 0.5);

        oscillator
    }

    fn new() -> (Self, Task<Message>) {
        let latency_hint = match std::env::var("WEB_AUDIO_LATENCY").as_deref().map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Ok("interactive") => AudioContextLatencyCategory::Interactive,
            Ok("balanced") => AudioContextLatencyCategory::Balanced,
            Ok("playback") => AudioContextLatencyCategory::Playback,
            _ => AudioContextLatencyCategory::default(),
        };

        let context = AudioContext::new(AudioContextOptions {
            latency_hint,
            ..AudioContextOptions::default()
        });

        (
            Self {
                audio_context: context,
                status_info: "Hello!".to_string(),
                osc_map: HashMap::new(),
            },
            Task::none(),
        )
    }
}
