use iced::Task;
use iced::futures::StreamExt;
use iced::widget::{button, column, text};
use std::collections::BTreeMap;
use web_audio_api::node::OscillatorNode;
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
    osc_map: BTreeMap<String, OscillatorNode>,
}

#[derive(Debug, Clone)]
enum Message {
    StartPlayback,
    NotifyPlaybackStopped(f64),
    StopPlayback,
}

impl AudioApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartPlayback => {
                let (sender, mut reciever) = iced::futures::channel::mpsc::channel::<f64>(1);

                let time = self.audio_context.current_time();
                for i in 1..=9 {
                    let osc = self.create_oscillator(time + i as f64);
                    let mut sender_clone = sender.clone(); // clone the sender...
                    osc.set_onended(move |_| {
                        if let Err(e) = sender_clone.try_send(time + i as f64) {
                            println!("{}", e);
                        }
                    });
                    self.osc_map.insert(format!("{:0007.2}", time + i as f64), osc);
                }

                return Task::stream(reciever.map(|time| Message::NotifyPlaybackStopped(time)));
            }

            Message::NotifyPlaybackStopped(time) => {
                println!("Playback has stopped at {:0007.2}", time);
                self.osc_map.remove(&format!("{:0007.2}", time));
                dbg!(&self.osc_map.keys().cloned().collect::<Vec<String>>());
                Task::none()
            }

            Message::StopPlayback => {
                for osc in self.osc_map.values_mut() {
                    osc.stop();
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        column![
            text(self.status_info.as_str()),
            button("Play").on_press(Message::StartPlayback),
            button("Stop").on_press(Message::StopPlayback),
        ]
        .into()
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
                osc_map: BTreeMap::new(),
            },
            Task::none(),
        )
    }

    fn create_oscillator(&self, time: f64) -> OscillatorNode {
        let mut oscillator = self.audio_context.create_oscillator();
        oscillator.connect(&self.audio_context.destination());
        oscillator.frequency().set_value(440.0);
        oscillator.start_at(time);
        oscillator.stop_at(time + 0.2);
        oscillator
    }
}
