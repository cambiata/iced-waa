use std::fs::File;

use iced::{Task, widget::button};
use web_audio_api::{
    context::{AudioContext, AudioContextLatencyCategory, AudioContextOptions, BaseAudioContext},
    node::{AudioNode, AudioScheduledSourceNode},
};

fn main() -> iced::Result {
    iced::application("My App", AudioApp::update, AudioApp::view).run_with(AudioApp::new)
}

#[derive(Default)]
struct AudioApp {
    audio_context: AudioContext,
}

use iced::futures::channel::oneshot::{self, Receiver, Sender};

#[derive(Debug, Clone)]
enum Message {
    StartPlayback,
    NotifyPlaybackStopped,
}

impl AudioApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NotifyPlaybackStopped => {
                println!("Playback has stopped");
            }
            Message::StartPlayback => {
                let afile = File::open("samples/piano/60.mp3").unwrap();
                let abuffer = self.audio_context.decode_audio_data_sync(afile).unwrap();
                let mut asource = self.audio_context.create_buffer_source();
                asource.connect(&self.audio_context.destination());
                asource.set_buffer(abuffer);

                // create a oneshot channel for passing info that playback ends
                let (sender, reciever): (Sender<f32>, Receiver<f32>) = oneshot::channel();

                // start playback immediately...
                asource.start_at(self.audio_context.current_time());

                // ...and set up an callback that runs when playback ends
                asource.set_onended(|_| {
                    println!("Playback ended callback triggered");
                    if let Err(_) = sender.send(0.1) {
                        // handle the case where the receiver was dropped
                    }
                });

                // pass the receiver as an async task that fire the message
                return Task::perform(reciever, |_| Message::NotifyPlaybackStopped);
            }
        }
        Task::none()
    }

    fn view(&self) -> iced::Element<Message> {
        button("Play").on_press(Message::StartPlayback).into()
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

        (Self { audio_context: context }, Task::none())
    }
}
