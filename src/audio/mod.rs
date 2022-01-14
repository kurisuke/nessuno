use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SampleRate, StreamConfig, SupportedBufferSize};
use crossbeam_channel::Receiver;
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: u32 = 4096;
const MIN_SAMPLE_RATE: u32 = 44100;

pub fn run(audio_recv: Receiver<f32>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no audio output device available");

        let supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .filter(|supported_config| supported_config.channels() == 1)
            .filter(|supported_config| match supported_config.buffer_size() {
                SupportedBufferSize::Range { min, max } => {
                    min <= &BUFFER_SIZE && &BUFFER_SIZE <= max
                }
                SupportedBufferSize::Unknown => true,
            })
            .next()
            .expect("no supported config?!");
        let min_sample_rate = supported_config.min_sample_rate();
        let set_buffer_size = match supported_config.buffer_size() {
            SupportedBufferSize::Range { min: _, max: _ } => cpal::BufferSize::Fixed(BUFFER_SIZE),
            SupportedBufferSize::Unknown => cpal::BufferSize::Default,
        };
        let selected_sample_rate = if min_sample_rate.0 <= MIN_SAMPLE_RATE {
            SampleRate(MIN_SAMPLE_RATE)
        } else {
            min_sample_rate
        };

        let supported_config = supported_config.with_sample_rate(selected_sample_rate);
        let sample_format = supported_config.sample_format();
        let num_channels = supported_config.channels();
        let mut config: StreamConfig = supported_config.into();
        config.buffer_size = set_buffer_size;

        println!(
            "samplerate: {}, format: {:?}, channels: {}",
            selected_sample_rate.0, sample_format, num_channels
        );

        let sample_callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                *sample = Sample::from(&audio_recv.try_recv().unwrap_or(0.0f32));
            }
        };

        let err_fn = |err| {
            println!(
                "ERROR: an error occurred on the output audio stream: {}",
                err
            )
        };

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(&config, sample_callback, err_fn),
            SampleFormat::I16 => device.build_output_stream(&config, sample_callback, err_fn),
            SampleFormat::U16 => device.build_output_stream(&config, sample_callback, err_fn),
        }
        .unwrap();

        println!("Start audio...");
        stream.play().unwrap();

        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });
}
