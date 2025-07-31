use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleRate, StreamConfig, SupportedBufferSize};
use crossbeam_channel::{Receiver, Sender};
use std::thread;
use std::time::Duration;

pub const BUFFER_SIZE: u32 = 2048;
const MIN_SAMPLE_RATE: u32 = 44100;
const MIN_SAMPLE_SIZE: usize = 2;

pub fn run(audio_recv: Receiver<f32>, sample_rate_send: Sender<u32>) {
    thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no audio output device available");

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .find(|supported_config| {
                supported_config.channels() == 1
                    && supported_config.sample_format().sample_size() >= MIN_SAMPLE_SIZE
                    && match supported_config.buffer_size() {
                        SupportedBufferSize::Range { min, max } => {
                            min <= &BUFFER_SIZE && &BUFFER_SIZE <= max
                        }
                        SupportedBufferSize::Unknown => true,
                    }
            })
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

        sample_rate_send.send(selected_sample_rate.0).unwrap();

        let sample_callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                *sample = Sample::from_sample(audio_recv.try_recv().unwrap_or(0.0f32));
            }
        };

        let err_fn = |err| println!("ERROR: an error occurred on the output audio stream: {err}");

        let stream = device
            .build_output_stream(&config, sample_callback, err_fn, None)
            .unwrap();

        println!("Start audio...");
        stream.play().unwrap();

        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });
}
