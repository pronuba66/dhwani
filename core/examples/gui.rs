extern crate dhwani;

use std::{fs::File, sync::Arc};

use anyhow::anyhow;
use cpal::{
    SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam_channel::{Receiver, Sender, bounded};
use dhwani::{
    channel::ChannelPositionsMask,
    controller::{CtrlRingBufReceiver, CtrlSender, start_controller},
    event, events,
    midi_note::MidiNote,
    nodes::{self, SampleInfo},
    time::{SampleRateBaseType, TimeFrom, TimeRange, TimeUnit},
};
use eframe::{
    App, Frame,
    egui::{self},
};
use egui_plot::{Line, Plot, PlotPoints};
use symphonia::{
    core::{
        audio::sample::Sample,
        codecs::audio::AudioDecoderOptions,
        formats::{FormatOptions, TrackType, probe::Hint},
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
    default::{get_codecs, get_probe},
};

const BUFFER_SIZE: usize = 44100 / 10;
const STRIDE: usize = BUFFER_SIZE / 5;
const CHANNEL_MASK: ChannelPositionsMask = ChannelPositionsMask::from_bits(
    ChannelPositionsMask::FRONT_LEFT.bits() | ChannelPositionsMask::FRONT_RIGHT.bits(),
)
.unwrap();

pub struct PlotData {
    i: usize,
    data: Vec<f32>,
    sender: Sender<Vec<f32>>,
}

impl PlotData {
    pub fn new(sender: Sender<Vec<f32>>) -> Self {
        Self {
            i: 0usize,
            data: vec![0f32; BUFFER_SIZE],
            sender,
        }
    }

    pub fn push(&mut self, buffer: &[f32]) {
        for value in buffer {
            self.data[BUFFER_SIZE - STRIDE + self.i] = *value;
            self.i += 1;
            if self.i >= STRIDE {
                self.i = 0;
                let _ = self.sender.try_send(self.data.clone());
                for i in 0..BUFFER_SIZE - STRIDE {
                    self.data[i] = self.data[STRIDE + i];
                }
            }
        }
    }
}

pub struct Plotter {
    is_playing: bool,
    data: [f32; BUFFER_SIZE / 2],
    receiver: Receiver<Vec<f32>>,
    ctrl_sender: CtrlSender,
}

impl Plotter {
    pub fn new(receiver: Receiver<Vec<f32>>, ctrl_sender: CtrlSender) -> Self {
        Self {
            is_playing: true,
            data: [0f32; BUFFER_SIZE / 2],
            receiver,
            ctrl_sender,
        }
    }

    pub fn play(&mut self, is_playing: bool) {
        self.is_playing = is_playing;
        println!("Play: {is_playing}");
        let ctrl_sender = self.ctrl_sender.clone();
        tokio::spawn(async move {
            let _ = ctrl_sender.play(is_playing).await;
        });
    }

    pub fn demo(&mut self) {
        println!("Demo");
        let ctrl_sender = self.ctrl_sender.clone();
        tokio::spawn(async move {
            if let Err(e) = ctrl_sender.clear().await {
                eprintln!("Failed to clear, {e:?}");
                return;
            }
            // Piano -> Output(L)
            // Piano -> Output(R)
            let track_id = match ctrl_sender
                .add_track(TimeRange::new(TimeUnit::Seconds(0f32), None))
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to add track, {e:?}");
                    return;
                }
            };
            // Simple piano
            let simple_piano_node_id = match ctrl_sender
                .add_node(
                    track_id,
                    Box::new(nodes::SimplePianoProps::new(
                        CHANNEL_MASK,
                        events![
                            (
                                0,
                                TimeUnit::Seconds(0f32),
                                event::EventData::NoteOn {
                                    note: MidiNote::from_midi_str("C4").unwrap(), // C4
                                    vel: 1f32,
                                }
                            ),
                            (
                                0,
                                TimeUnit::Seconds(1f32),
                                event::EventData::NoteOff {
                                    note: MidiNote::from_midi_str("C4").unwrap(), // C4
                                    vel: 1f32,
                                }
                            ),
                            (
                                1, // Do Not reuse ID, they will cause the phase to continue
                                TimeUnit::Seconds(2f32),
                                event::EventData::NoteOn {
                                    note: MidiNote::from_midi_str("D4").unwrap(), // D4
                                    vel: 1f32,
                                }
                            ),
                            (
                                1,
                                TimeUnit::Seconds(4f32),
                                event::EventData::NoteOff {
                                    note: MidiNote::from_midi_str("D4").unwrap(), // D4
                                    vel: 1f32,
                                }
                            ),
                            (
                                2, // Do Not reuse ID, they will cause the phase to continue
                                TimeUnit::Seconds(5f32),
                                event::EventData::NoteOn {
                                    note: MidiNote::from_midi_str("E4").unwrap(), // E4
                                    vel: 1f32,
                                }
                            ),
                            (
                                2,
                                TimeUnit::Seconds(20f32),
                                event::EventData::NoteOff {
                                    note: MidiNote::from_midi_str("E4").unwrap(), // E4
                                    vel: 1f32,
                                }
                            ),
                        ],
                    )),
                )
                .await
            {
                Ok(node_id) => node_id,
                Err(e) => {
                    eprintln!("Failed to add Node, {e:?}");
                    return;
                }
            };
            // Sampler
            let info = match get_sample_info(
                "core/examples/giomilko-c-major-9-bossa-nova-guitar.wav".into(),
            ) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to get sample info, {e:?}");
                    return;
                }
            };
            let sampler_node_id = match ctrl_sender
                .add_node(track_id, Box::new(nodes::SamplerProps::new(info).unwrap()))
                .await
            {
                Ok(node_id) => node_id,
                Err(e) => {
                    eprintln!("Failed to add Node, {e:?}");
                    return;
                }
            };
            // Mixer
            let simple_mixer_node_id = match ctrl_sender
                .add_node(
                    track_id,
                    Box::new(nodes::SimpleMixerProps::new(
                        CHANNEL_MASK,
                        vec![0.1f32, 0.5f32],
                    )),
                )
                .await
            {
                Ok(node_id) => node_id,
                Err(e) => {
                    eprintln!("Failed to add Node, {e:?}");
                    return;
                }
            };
            // Connect simple piano to mixer
            if let Err(e) = ctrl_sender
                .connect_nodes(simple_piano_node_id, simple_mixer_node_id)
                .await
            {
                eprintln!("Failed to connect, {e:?}");
            }
            // Connect sampler to mixer
            if let Err(e) = ctrl_sender
                .connect_nodes(sampler_node_id, simple_mixer_node_id)
                .await
            {
                eprintln!("Failed to connect, {e:?}");
            }
            // Set output port
            if let Err(e) = ctrl_sender
                .set_output_port(Some((
                    simple_mixer_node_id,
                    nodes::SimpleMixerProps::PORT_ID_OUTPUT,
                )))
                .await
            {
                eprintln!("Failed to set output port, {e:?}");
                return;
            }
            let _ = ctrl_sender.play(true).await;
        });
    }
}

impl App for Plotter {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        let mut should_close = false;
        let ctx = ui.ctx().clone();
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                    ..
                } = event
                    && *pressed
                {
                    match *key {
                        egui::Key::R => {
                            self.demo();
                        }
                        egui::Key::Q => {
                            should_close = true;
                        }
                        egui::Key::Space => {
                            self.play(!self.is_playing);
                        }
                        egui::Key::Comma => {
                            if !modifiers.command {
                                let time = TimeUnit::Seconds(-4f32);
                                println!("Set Time to {time:?} from current");
                                let ctrl_sender = self.ctrl_sender.clone();
                                tokio::spawn(async move {
                                    let _ = ctrl_sender.set_time(TimeFrom::Current(time)).await;
                                });
                            } else {
                                println!("Set Time to start");
                                let ctrl_sender = self.ctrl_sender.clone();
                                tokio::spawn(async move {
                                    let _ = ctrl_sender
                                        .set_time(TimeFrom::Start(TimeUnit::default()))
                                        .await;
                                });
                            }
                        }
                        egui::Key::Period => {
                            if !modifiers.command {
                                let time = TimeUnit::Seconds(4f32);
                                println!("Set Time to {time:?} from current");
                                let ctrl_sender = self.ctrl_sender.clone();
                                tokio::spawn(async move {
                                    let _ = ctrl_sender.set_time(TimeFrom::Current(time)).await;
                                });
                            } else {
                                println!("Set Time to end");
                                let ctrl_sender = self.ctrl_sender.clone();
                                tokio::spawn(async move {
                                    let _ = ctrl_sender
                                        .set_time(TimeFrom::End(TimeUnit::default()))
                                        .await;
                                });
                            }
                        }
                        #[cfg(feature = "debug")]
                        egui::Key::P => {
                            println!("Render graph");
                            let ctrl_sender = self.ctrl_sender.clone();
                            tokio::spawn(async move {
                                let _ = ctrl_sender.debug_render_graph("graph.html".into()).await;
                            });
                        }
                        _ => {}
                    }
                }
            }
        });
        if should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        // update your data here
        let points: PlotPoints<'_> = PlotPoints::from_ys_f32(&self.data);
        let line = Line::new("left", points).color(egui::Color32::from_rgb(100, 200, 100));
        egui::CentralPanel::default().show_inside(ui, |ui| {
            Plot::new("live")
                .show_axes(true)
                .default_y_bounds(-1.5f64, 1.5f64)
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
        });
        if !self.is_playing {
            return;
        }
        if let Ok(data) = self.receiver.try_recv() {
            for i in 0..data.len() / 2 {
                self.data[i] = data[i * 2];
            }
        }
        ctx.request_repaint(); // keep redrawing for live update
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let host: cpal::Host = cpal::default_host();
    let device: cpal::Device = host
        .default_output_device()
        .ok_or_else(|| anyhow::Error::msg("Default output device is not available"))?;
    println!("Output device : {}", device.description()?);
    let default_config = device.default_output_config()?;
    let sample_rate = default_config.sample_rate();
    let config: SupportedStreamConfig = SupportedStreamConfig::new(
        default_config.channels(),
        sample_rate,
        *default_config.buffer_size(),
        default_config.sample_format(),
    );
    println!("Output config : {config:?}");
    let (ctrl_sender, ctrl_receiver) =
        start_controller(sample_rate, default_config.channels(), 1024);
    let (sender, receiver) = bounded::<Vec<f32>>(1);
    let stream = make_stream(&device, config.into(), sender, ctrl_receiver)?;
    println!("Stream playing...");
    let _ = stream.play();
    let options = eframe::NativeOptions::default();
    {
        eframe::run_native(
            "dhwani",
            options,
            Box::new(move |_cc| {
                let mut plotter = Plotter::new(receiver, ctrl_sender);
                plotter.demo();
                Ok(Box::new(plotter))
            }),
        )
        .map_err(|_| anyhow::Error::msg("Run native failed"))?;
    }
    println!("Exiting main");
    Ok(())
}

pub fn make_stream(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    sender: Sender<Vec<f32>>,
    ctrl_receiver: CtrlRingBufReceiver,
) -> Result<cpal::Stream, anyhow::Error> {
    let mut plot_data = PlotData::new(sender);
    let stream: cpal::Stream = device.build_output_stream(
        config,
        move |output: &mut [f32], _output_callback_info: &cpal::OutputCallbackInfo| {
            match ctrl_receiver.get(output) {
                Ok(read) => {
                    if read != output.len() {
                        // eprintln!("Buffer underrun, read = {}/{}!", read, output.len());
                    }
                }
                Err(e) => {
                    eprintln!("{e:?}");
                }
            }
            plot_data.push(output);
        },
        |err| eprintln!("Error building output sound stream: {err}"),
        None,
    )?;
    Ok(stream)
}

fn get_sample_info(path: String) -> Result<SampleInfo, anyhow::Error> {
    // ref: https://github.com/pdeljanov/Symphonia/blob/main/symphonia/examples/getting-started.rs
    let src = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let hint = Hint::new();
    // Use the default options for metadata and format readers.
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();
    // Probe the media source.
    let mut format = get_probe().probe(&hint, mss, fmt_opts, meta_opts)?;
    // Find the first audio track with a known (decodeable) codec.
    let track = format
        .default_track(TrackType::Audio)
        .ok_or_else(|| anyhow!("No audio track"))?;
    // Use the default options for the decoder.
    let dec_opts: AudioDecoderOptions = Default::default();
    // Create a decoder for the track.
    let mut decoder = get_codecs().make_audio_decoder(
        track
            .codec_params
            .as_ref()
            .ok_or_else(|| anyhow!("Codec parameters missing"))?
            .audio()
            .unwrap(),
        &dec_opts,
    )?;
    // Store the track identifier, it will be used to filter packets.
    let track_id = track.id;
    let mut samples = Vec::<f32>::new();
    let mut sample_rate: SampleRateBaseType = 0;
    let mut n_channels: u16 = 0;
    let mut n_samples_per_ch: usize = 0;
    // The decode loop.
    loop {
        // Get the next packet from the media format.
        let packet = match format.next_packet()? {
            Some(packet) => packet,
            None => {
                // Reached the end of the stream.
                break;
            }
        };
        // Consume any new metadata that has been read since the last packet.
        while !format.metadata().is_latest() {
            // Pop the old head of the metadata queue.
            format.metadata().pop();
            // Consume the new metadata at the head of the metadata queue.
        }
        // If the packet does not belong to the selected track, skip over it.
        if packet.track_id != track_id {
            continue;
        }
        // Decode the packet into audio samples.
        let audio_buf = decoder.decode(&packet)?;
        if n_channels == 0 {
            n_channels = audio_buf.spec().channels().count() as u16;
        }
        if sample_rate == 0 {
            sample_rate = audio_buf.spec().rate();
        }
        n_samples_per_ch += audio_buf.samples_planar();
        let len = samples.len();
        samples.resize(len + audio_buf.samples_interleaved(), f32::MID);
        audio_buf.copy_to_slice_interleaved(&mut samples[len..]);
    }
    Ok(SampleInfo::new(
        0,
        n_channels,
        n_samples_per_ch,
        Arc::new(samples),
    ))
}
