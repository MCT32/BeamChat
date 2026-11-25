use cpal::{BufferSize, Sample, SampleFormat, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use hrtf::{HrirSphere, HrtfContext, HrtfProcessor, Vec3};
use log::{debug, error};

pub fn audio_thread() {
    let host = cpal::default_host();

    let output_device = match host.default_output_device() {
        Some(output_device) => output_device,
        None => {
            error!("Could not find a default output device");
            return;
        }
    };

    let mut supported_configs_range = match output_device.supported_output_configs() {
        Ok(supported_configs_range) => supported_configs_range,
        Err(err) => {
            error!("Error getting supported configs from output device: {err}");
            return;
        },
    };

    // Only supports F32
    // TODO: Add support for other formats
    // TODO: Allow sample rate to be configured
    let supported_config = match supported_configs_range.find(|&x|
            x.sample_format() == SampleFormat::F32 &&
            x.channels() == 2
        )
        .and_then(|c| c.try_with_sample_rate(48_000)) {
            Some(supported_config) => supported_config,
            None => {
                error!("Could not find compatible configuration for the selected output device");
                return;
            },
        };

    let sample_format = supported_config.sample_format();
    let sample_rate = supported_config.sample_rate();
    let channels = supported_config.channels();
    let buffer_size_range = supported_config.buffer_size();
    debug!("Found device config with sample format {sample_format} and sample rate {sample_rate} with {channels} channels and size {buffer_size_range:?}");

    let mut config: StreamConfig = supported_config.into();

    // TODO: Check buffer size is supported
    let desired_buffer_size = 512;
    config.buffer_size = BufferSize::Fixed(desired_buffer_size);

    // TODO: Print when file is loading
    // TODO: Configurable sphere
    let hrir_sphere = match HrirSphere::from_file("./IRC_1002_C.bin", sample_rate) {
        Ok(hrir_sphere) => hrir_sphere,
        Err(err) => {
            error!("Error loading HRIR sphere: {err:?}");
            return;
        },
    };

    // unwrap should be fine here
    let mut hrtf_processor = HrtfProcessor::new(hrir_sphere, desired_buffer_size.try_into().unwrap());

    let err_fn = |err| error!("an error occurred on the output audio stream: {err}");
    let stream = match sample_format {
        SampleFormat::F32 => output_device.build_output_stream(config, move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {process_audio(data, info, &mut hrtf_processor)}, err_fn, None),
        sample_format => {
            error!("Unsupported sample format {sample_format:?}");
            return;
        },
    };
    // TODO: Find a way to do this without shadowing
    let stream = match stream {
        Ok(stream) => stream,
        Err(err) => {
            error!("Error creating output audio stream: {err}");
            return;
        },
    };

    match stream.play() {
        Ok(_) => (),
        Err(err) => {
            error!("Error starting output audio stream: {err}");
            return;
        },
    };

    loop {}
}

fn process_audio(data: &mut [f32], _: &cpal::OutputCallbackInfo, processor: &mut HrtfProcessor) {
    let mut source = vec![0f32;data.len() / 2];
    write_noise(&mut source);

    let mut stereo = vec![(0f32, 0f32);source.len()];

    let mut history = vec![0f32;source.len()];
    let vector = Vec3::new(1.0, 0.0, 0.0);
    let context = HrtfContext {
        source: &source,
        output: &mut stereo,
        new_sample_vector: vector,
        prev_sample_vector: vector,
        new_gain: 1.0,
        prev_gain: 1.0,
        history: &mut history
    };

    processor.process_samples(context);

    stereo_frames_to_interleaved(&stereo, data);
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}

// TODO: Support other types
// FIXME: Assumes stereo
fn write_noise(data: &mut [f32]) {
    for frame in data.chunks_exact_mut(2) {
        let sample = rand::random_range(-1.0..1.0);
        frame[0] = sample;
        frame[1] = sample;
    }
}

// TODO: Support other types
// TODO: Error for mismatched size
// TODO: Consider returning output instead of writing to it
fn stereo_frames_to_interleaved(input: &[(f32, f32)], output: &mut [f32]) {
    for (dst, &(left, right)) in output
        .chunks_exact_mut(2)
        .zip(input.iter()) {
            dst[0] = left;
            dst[1] = right;
        }
}