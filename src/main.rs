use cpal::traits::{ DeviceTrait, HostTrait, StreamTrait };
use std::{ sync::mpsc::channel, time::Duration };
use tokio::runtime::Runtime;
use async_openai::{ types::{ AudioInput, CreateTranscriptionRequestArgs, InputSource }, Client };
use std::error::Error;
use tokio::sync::mpsc;
use async_openai::types::InputSource::VecU8;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    capture_and_transcribe().await;

    Ok(())
}

async fn send_audio_for_transcription(voice_record: AudioInput) -> Result<String, Box<dyn Error>> {
    let client = Client::new(); // Assuming the client is instantiated without needing an API key; adjust as necessary.

    let request = CreateTranscriptionRequestArgs::default()
        .file(voice_record)
        .model("whisper-1") // Specify the model; adjust as needed.
        .build()?;

    let response = client.audio().transcribe(request).await?;

    Ok(response.text)
}

async fn capture_and_transcribe() {
    let (tx, mut rx) = mpsc::channel(32);

    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input device available");
        let config = device.default_input_config().unwrap();

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let audio_chunk = encode_audio_to_format(data); // You need to implement this
            tx.blocking_send(audio_chunk).unwrap();
        };

        let stream = device
            .build_input_stream(
                &config.into(),
                input_data_fn,
                err_fn,
                Some(Duration::from_secs(15))
            )
            .unwrap();
        stream.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(15)); // Capture for a limited time
    });

    while let Some(chunk) = rx.recv().await {
        if
            let Ok(transcription) = send_audio_for_transcription(AudioInput {
                source: InputSource::VecU8 {
                    filename: String::from("audio.mp3"),
                    vec: chunk.expect("REASON"),
                },
            }).await
        {
            println!("Transcription: {}", transcription);
        } // You need to implement this
    }
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("An error occurred on stream: {}", err);
}

use std::io::Cursor;
use hound::{ WavSpec, WavWriter, SampleFormat };

fn encode_audio_to_format(data: &[f32]) -> anyhow::Result<Vec<u8>> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut buffer, spec)?;
        for &sample in data.iter() {
            let amplitude = (sample * (i16::MAX as f32)) as i16;
            writer.write_sample(amplitude)?;
        }
        writer.finalize()?;
    }

    Ok(buffer.into_inner())
}
