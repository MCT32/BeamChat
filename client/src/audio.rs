use cpal::{Sample, SampleFormat, traits::{DeviceTrait, HostTrait, StreamTrait}};
use log::{debug, error};

pub fn audio_thread() {
    let host = cpal::default_host();

    let output = host.default_output_device().unwrap();

    let mut supported_configs_range = output.supported_output_configs().unwrap();
    // Only supports F32
    // TODO: Add support for other formats
    let supported_config = supported_configs_range.find(|&x| x.sample_format() == SampleFormat::F32)
        .unwrap().with_max_sample_rate();

    let sample_format = supported_config.sample_format();
    let sample_rate = supported_config.sample_rate();
    debug!("Found device config with sample format {sample_format} and sample rate {sample_rate}");
    let config = supported_config.into();
    let err_fn = |err| error!("an error occurred on the output audio stream: {err}");
    let stream = match sample_format {
        SampleFormat::F32 => output.build_output_stream(config, write_noise, err_fn, None),
        sample_format => panic!("Unsupported sample format {sample_format:?}"),
    }.unwrap();

    stream.play().unwrap();

    loop {}
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}

// TODO: Support other types
fn write_noise(data: &mut [f32], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = rand::random_range(-1.0..1.0);
    }
}