use std::sync::mpsc::Receiver;

use cpal::{BufferSize, Sample, SampleFormat, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use log::{debug, error};

use crate::packet::{Camera, Packet};

#[derive(Debug, Clone)]
struct VehicleData {
    angle: f32,
    gain: f32,
}

struct AudioState {
    position_data: Option<Vec<VehicleData>>,
    stereo: [(f32, f32); 1025]
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            position_data: None,
            stereo: [(0f32, 0f32); 1025]
        }
    }
}

pub fn audio_thread(game_data_receiver: Receiver<Packet>) {
    let mut audio_reader = hound::WavReader::open("sample.wav").unwrap();

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
    let desired_buffer_size = 1025;
    config.buffer_size = BufferSize::Fixed(desired_buffer_size);

    let err_fn = |err| error!("an error occurred on the output audio stream: {err}");

    let mut audio_state: AudioState = Default::default();

    let (position_data_sender, position_data_receiver) = std::sync::mpsc::channel::<Vec<VehicleData>>();
    let stream = match sample_format {
        SampleFormat::F32 => output_device.build_output_stream(config, move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
            process_audio(data, info, &position_data_receiver, &mut audio_state)
        }, err_fn, None),
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

    for packet in game_data_receiver.iter() {
        let position_data = packet.vehicles.iter().map(|p| {
            let camera_space_position = position_to_camera_space(&packet.camera, p.position);
            VehicleData {
                angle: camera_space_position.angle_to(glam::vec3(0.0, 1.0, 0.0), glam::vec3(0.0, 0.0, 1.0)),
                gain: 1.0 / camera_space_position.length(),
            }
        }).collect();

        position_data_sender.send(position_data).unwrap();
    }
}

fn process_audio(
    data: &mut [f32],
    _: &cpal::OutputCallbackInfo,
    position_data_receiver: &Receiver<Vec<VehicleData>>,
    state: &mut AudioState
) {
    while let Ok(new_positions) = position_data_receiver.try_recv() {
        state.position_data = Some(new_positions)
    }

    let Some(positions) = state.position_data.as_deref() else {
        data.fill(0.0);
        return;
    };

    state.stereo.fill((0.0, 0.0));

    // TODO: Interpolate between frames
    // TODO: Interpolate between updates
    for vehicle in positions {
        let mut samples = vec![0.0; state.stereo.len()];
        write_noise(&mut samples);

        for ((left, right), source) in state.stereo.iter_mut().zip(samples) {
            let pan = vehicle.angle.sin();
            let left_gain = ((1.0 - pan) * 0.5).sqrt();
            let right_gain = ((1.0 + pan) * 0.5).sqrt();    

            *left += source * vehicle.gain * left_gain;
            *right += source * vehicle.gain * right_gain;
        }
    }

    stereo_frames_to_interleaved(&state.stereo, data);
}

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::EQUILIBRIUM;
    }
}

// TODO: Support other types
// FIXME: Assumes stereo
fn write_noise(data: &mut Vec<f32>) {
    for sample in data.iter_mut() {
        *sample = rand::random_range(-1.0..1.0);
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

fn position_to_camera_space(camera: &Camera, position: glam::Vec3) -> glam::Vec3 {
    camera.rotation * (position - camera.position)
}