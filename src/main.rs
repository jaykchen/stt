use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::channel;
use tokio::runtime::Runtime;

fn main() {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    println!("Input device: {}", device.name().unwrap());

    let config = device.default_input_config().unwrap();
    println!("Default input config: {:?}", config);

    let (sender, receiver) = channel();

    let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &_| {
            let _ = sender.send(data.to_owned());
        },
        err_fn,
    ).unwrap();

    stream.play().unwrap();
    println!("Recording... Press Ctrl+C to stop");

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        loop {
            match receiver.recv() {
                Ok(audio_data) => {
                    // Here you would typically convert your audio data to a suitable format
                    // and save or send it for transcription.
                    // This example does not include audio format conversion or async sending logic.
                    // You would use reqwest and possibly the tokio::fs module to handle file operations.
                    println!("Received audio data: {:?}", audio_data);
                },
                Err(_) => break,
            }
        }
    });
}
