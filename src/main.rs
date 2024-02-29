use async_openai::{ types::{ AudioInput, CreateTranscriptionRequestArgs, InputSource }, Client };
use cpal::traits::{ DeviceTrait, HostTrait, StreamTrait };
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let res = capture_and_transcribe().await.expect("REASON");
    println!("{:?}", res);

    Ok(())
}

async fn send_audio_for_transcription(voice_record: AudioInput) -> anyhow::Result<String> {
    let client = Client::new(); // Assuming the client is instantiated without needing an API key; adjust as necessary.

    let request = CreateTranscriptionRequestArgs::default()
        .file(voice_record)
        .model("whisper-1") // Specify the model; adjust as needed.
        .build()?;

    let response = client.audio().transcribe(request).await?;

    Ok(response.text)
}

async fn capture_and_transcribe() -> anyhow::Result<String> {
    let (tx, mut rx) = mpsc::channel(32);

    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input device available");
        let config = device.default_input_config().unwrap();

        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let audio_chunk = encode_audio_to_format(data).unwrap(); // Ensure this handles errors properly
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
        std::thread::sleep(Duration::from_secs(10));
        drop(stream); // Dropping the stream stops the capture.
    });
    if let Some(chunk_result) = rx.recv().await {
        let audio_data = chunk_result.to_vec(); 
        save_audio_to_file(&audio_data, "audio.wav")?; // Correctly call save_audio_to_file

        return send_audio_for_transcription(AudioInput {
            source: InputSource::VecU8 {
                filename: String::from("audio.wav"),
                vec: audio_data, // Now correctly a Vec<u8>
            },
        }).await;
    }
    /*     while let Some(chunk) = rx.recv().await {
        if let Ok(transcription) = send_audio_for_transcription(AudioInput {
            source: InputSource::VecU8 {
                filename: String::from("audio.mp3"),
                vec: chunk.into(),
            },
        })
        .await
        {
            return Ok(transcription); // Return as soon as a transcription is successfully received
        }
    } */

    Err(anyhow::anyhow!("No transcription received"))
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("An error occurred on stream: {}", err);
}

use hound::{ SampleFormat, WavSpec, WavWriter };
use std::io::Cursor;

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

fn save_audio_to_file(audio_data: &[u8], file_name: &str) -> anyhow::Result<()> {
    let mut file = File::create(file_name)?;
    file.write_all(audio_data)?;
    Ok(())
}