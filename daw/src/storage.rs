use std::{
    fs::File,
    sync::{Arc, Mutex},
};

use dhwani::{nodes::SampleInfo, time::SampleRateBaseType};
use serde::Serialize;
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
use tauri::State;

use crate::core::err_to_string;

#[derive(Default)]
pub struct Storage {
    id: usize,
    infos: Vec<SampleInfo>,
}

impl Storage {
    pub fn clear(&mut self) {
        self.id = 0;
        self.infos.clear();
    }

    pub fn new_id(&mut self) -> usize {
        let id = self.id;
        self.id = id + 1;
        return id;
    }

    pub fn add(&mut self, file_info: SampleInfo) {
        self.infos.push(file_info);
    }

    pub fn get(&self, id: usize) -> Option<SampleInfo> {
        return self
            .infos
            .iter()
            .find(|&info| info.id() == id)
            .map(|info| info.clone());
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfo {
    pub id: usize,
    pub n_channels: u16,
    pub sample_rate: SampleRateBaseType,
    pub n_samples_per_ch: usize,
}

#[tauri::command]
pub async fn store_file<'a>(
    storage: State<'a, Mutex<Storage>>,
    path: String,
) -> Result<AudioInfo, String> {
    let id = {
        let mut guard = storage.lock().unwrap();
        guard.new_id()
    };
    let handle = tokio::task::spawn_blocking(move || -> Result<(AudioInfo, Vec<f32>), String> {
        // ref: https://github.com/pdeljanov/Symphonia/blob/main/symphonia/examples/getting-started.rs
        let src = File::open(path).map_err(err_to_string)?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());
        let hint = Hint::new();
        // Use the default options for metadata and format readers.
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();
        // Probe the media source.
        let mut format = get_probe()
            .probe(&hint, mss, fmt_opts, meta_opts)
            .map_err(err_to_string)?;
        // Find the first audio track with a known (decodeable) codec.
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| "No audio track".to_string())?;
        // Use the default options for the decoder.
        let dec_opts = AudioDecoderOptions::default();
        // Create a decoder for the track.
        let mut decoder = get_codecs()
            .make_audio_decoder(
                track
                    .codec_params
                    .as_ref()
                    .ok_or_else(|| "Codec parameters missing".to_string())?
                    .audio()
                    .unwrap(),
                &dec_opts,
            )
            .map_err(err_to_string)?;
        // Store the track identifier, it will be used to filter packets.
        let track_id = track.id;
        let mut samples = Vec::<f32>::new();
        let mut sample_rate: SampleRateBaseType = 0;
        let mut n_channels: u16 = 0;
        let mut n_samples_per_ch: usize = 0;
        // The decode loop.
        loop {
            // Get the next packet from the media format.
            let packet = match format.next_packet().map_err(err_to_string)? {
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
            let audio_buf = decoder.decode(&packet).map_err(err_to_string)?;
            if n_channels == 0 {
                n_channels = audio_buf.spec().channels().count() as u16;
            }
            if sample_rate == 0 {
                sample_rate = audio_buf.spec().rate();
            }
            n_samples_per_ch += audio_buf.samples_planar();
            let len = samples.len();
            samples.resize(samples.len() + audio_buf.samples_interleaved(), f32::MID);
            audio_buf.copy_to_slice_interleaved(&mut samples[len..]);
        }
        let info = AudioInfo {
            id,
            n_channels,
            sample_rate,
            n_samples_per_ch,
        };
        Ok((info, samples))
    });
    let (info, samples) = handle.await.map_err(err_to_string)??;
    {
        let mut guard = storage.lock().unwrap();
        guard.add(SampleInfo::new(
            info.id,
            info.n_channels,
            info.n_samples_per_ch,
            Arc::new(samples),
        ));
    }
    Ok(info)
}
