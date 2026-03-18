use std::fs::File;
use std::time::Duration;

use iced::time::{self};
use iced::{
    Subscription, Task,
    widget::{button, column, text},
};

use web_audio_api::{
    context::{AudioContext, AudioContextLatencyCategory, AudioContextOptions, BaseAudioContext},
    node::{AudioNode, AudioScheduledSourceNode},
};

fn main() -> iced::Result {
    iced::application("AudioApp", AudioApp::update, AudioApp::view)
        .subscription(AudioApp::subscription)
        .run_with(AudioApp::new)
}

#[derive(Default)]
struct AudioApp {
    audio_context: AudioContext,
    status_info: String,
    timer_enabled: bool,
}

use iced::futures::channel::oneshot;

#[derive(Debug, Clone)]
enum Message {
    StartPlayback,
    NotifyPlaybackStopped,
    Tick,
}

impl AudioApp {
    fn subscription(&self) -> Subscription<Message> {
        if self.timer_enabled {
            time::every(Duration::from_millis(20)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartPlayback => {
                let afile = File::open("samples/piano/60.mp3").unwrap();
                let abuffer = self.audio_context.decode_audio_data_sync(afile).unwrap();
                let mut asource = self.audio_context.create_buffer_source();
                asource.connect(&self.audio_context.destination());
                asource.set_buffer(abuffer);

                // create a oneshot channel for passing info that playback ends
                let (sender, reciever) = oneshot::channel();

                // start playback immediately...
                asource.start_at(self.audio_context.current_time());
                self.timer_enabled = true;

                // ...and set up an callback that runs when playback ends
                asource.set_onended(|_| {
                    println!("Playback ended callback triggered");
                    if let Err(_) = sender.send(()) {
                        // handle the case where the receiver was dropped
                    }
                });

                // pass the receiver as an async task that fire the message
                return Task::perform(reciever, |_| Message::NotifyPlaybackStopped);
            }
            Message::NotifyPlaybackStopped => {
                println!("Playback has stopped");
                self.timer_enabled = false;
                self.status_info = "Playback has stopped".to_string();
            }
            Message::Tick => {
                self.status_info = format!("{:.2}", &self.audio_context.current_time());
            }
        }

        Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        column![text(self.status_info.as_str()), button("Play").on_press(Message::StartPlayback)].into()
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
                timer_enabled: false,
            },
            Task::none(),
        )
    }
}
