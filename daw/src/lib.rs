mod core;
mod storage;

use std::sync::Mutex;

use cpal::traits::{HostTrait, StreamTrait};
use cpal::{SupportedStreamConfig, traits::DeviceTrait};
use dhwani::controller::{CtrlRingBufReceiver, start_controller};

use crate::storage::Storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), anyhow::Error> {
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
    let stream = make_stream(&device, config.into(), ctrl_receiver)?;
    stream.pause()?;
    let storage = Storage::default();
    tauri::Builder::default()
        .manage(Mutex::new(stream))
        .manage(Mutex::new(storage))
        .manage(ctrl_sender)
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            core::clear,
            core::get_tracks,
            core::get_nodes,
            core::get_ports,
            core::play,
            core::seek,
            core::add_track,
            core::remove_track,
            core::set_track_time_range,
            core::add_node,
            core::replace_node,
            core::remove_node,
            core::set_output_port,
            core::connect_ports,
            core::unlink_port,
            storage::store_file,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_process::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}

pub fn make_stream(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    ctrl_receiver: CtrlRingBufReceiver,
) -> Result<cpal::Stream, anyhow::Error> {
    let stream: cpal::Stream = device.build_output_stream(
        config,
        move |output: &mut [f32], _output_callback_info: &cpal::OutputCallbackInfo| {
            match ctrl_receiver.get(output) {
                Ok(read) => {
                    if read != output.len() {
                        eprintln!("Buffer underrun, read = {}/{}!", read, output.len());
                    }
                }
                Err(e) => {
                    eprintln!("{e:?}");
                }
            }
        },
        |err| eprintln!("Error building output sound stream: {err}"),
        None,
    )?;
    Ok(stream)
}
