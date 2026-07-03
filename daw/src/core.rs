use std::fmt::Debug;
use std::sync::Mutex;

use cpal::Stream;
use cpal::traits::StreamTrait;
use dhwani::channel::ChannelPositionsMask;
use dhwani::controller::CtrlSender;
use dhwani::midi::MidiEvent;
use dhwani::midi::MidiMsg;
use dhwani::midi::MidiNote;
use dhwani::node::NodeBuilderTrait;
use dhwani::node::NodeId;
use dhwani::nodes;
use dhwani::nodes::NodeInfo;
use dhwani::port::Port;
use dhwani::port::PortId;
use dhwani::time::TimeBaseType;
use dhwani::time::TimeFrom;
use dhwani::time::TimeRange;
use dhwani::time::TimeUnit;
use dhwani::track::Track;
use dhwani::track::TrackId;
use serde::Deserialize;
use serde::Serialize;
use tauri::State;

use crate::storage::Storage;

pub fn err_to_string(e: impl Debug) -> String {
    format!("{:?}", e)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Port2 {
    pub id: usize,
    pub node_id: usize,
    pub is_event: bool,
    pub is_input: bool,
    pub auto_connect: bool,
    pub name: String,
}

impl From<Port> for Port2 {
    fn from(value: Port) -> Self {
        Port2 {
            id: value.id().0,
            node_id: value.node_id().0,
            is_event: value.kind().is_voice(),
            is_input: value.kind().is_input(),
            auto_connect: value.auto_connect(),
            name: value.name().to_string(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo2 {
    pub id: usize,
    pub track_id: usize,
}

impl From<NodeInfo> for NodeInfo2 {
    fn from(value: NodeInfo) -> Self {
        NodeInfo2 {
            id: value.id.0,
            track_id: value.track_id.0,
        }
    }
}

#[tauri::command]
pub async fn clear<'a>(
    storage: State<'a, Mutex<Storage>>,
    ctrl_sender: State<'a, CtrlSender>,
) -> Result<(), String> {
    storage.lock().unwrap().clear();
    ctrl_sender.clear().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tracks<'a>(ctrl_sender: State<'a, CtrlSender>) -> Result<Vec<usize>, String> {
    let n = ctrl_sender.get_track_count().await.map_err(err_to_string)?;
    let tracks = Vec::<Track>::with_capacity(n);
    ctrl_sender
        .get_tracks(tracks)
        .await
        .map(|mut tracks| {
            tracks
                .drain(..)
                .map(|track| track.id().0)
                .collect::<Vec<usize>>()
        })
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn get_nodes<'a>(
    ctrl_sender: State<'a, CtrlSender>,
    track_id: usize,
) -> Result<Vec<NodeInfo2>, String> {
    let track_id: TrackId = track_id.into();
    let n = ctrl_sender
        .get_node_count(track_id)
        .await
        .map_err(err_to_string)?;
    let nodes = Vec::<NodeInfo>::with_capacity(n);
    ctrl_sender
        .get_nodes(track_id, nodes)
        .await
        .map(|mut nodes| {
            nodes
                .drain(..)
                .map(|node| NodeInfo2::from(node))
                .collect::<Vec<NodeInfo2>>()
        })
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn get_ports<'a>(
    ctrl_sender: State<'a, CtrlSender>,
    node_id: usize,
) -> Result<Vec<Port2>, String> {
    let node_id: NodeId = node_id.into();
    let n = ctrl_sender
        .get_port_count(node_id)
        .await
        .map_err(err_to_string)?;
    let ports = Vec::<Port>::with_capacity(n);
    ctrl_sender
        .get_ports(node_id, ports)
        .await
        .map(|mut ports| {
            ports
                .drain(..)
                .map(|port| Port2::from(port))
                .collect::<Vec<Port2>>()
        })
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn play<'a>(
    stream: State<'a, Mutex<Stream>>,
    ctrl_sender: State<'a, CtrlSender>,
    enable: bool,
) -> Result<(), String> {
    if enable {
        stream.lock().unwrap().play().map_err(err_to_string)?;
    }
    ctrl_sender.play(enable).await.map_err(err_to_string)?;
    if !enable {
        stream.lock().unwrap().pause().map_err(err_to_string)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn seek<'a>(
    ctrl_sender: State<'a, CtrlSender>,
    mode: String,
    time: TimeBaseType,
) -> Result<TimeBaseType, String> {
    match mode.as_str() {
        "start" => ctrl_sender
            .set_time(TimeFrom::Start(TimeUnit::Seconds(time)))
            .await
            .map_err(err_to_string),
        "end" => ctrl_sender
            .set_time(TimeFrom::End(TimeUnit::Seconds(time)))
            .await
            .map_err(err_to_string),
        "current" => ctrl_sender
            .set_time(TimeFrom::Current(TimeUnit::Seconds(time)))
            .await
            .map_err(err_to_string),
        _ => return Err("Invalid mode".into()),
    }
}

#[tauri::command]
pub async fn add_track<'a>(
    ctrl_sender: State<'a, CtrlSender>,
    start: TimeBaseType,
    end: Option<TimeBaseType>,
) -> Result<usize, String> {
    let start = TimeUnit::Seconds(start);
    let end = end.map(|end| TimeUnit::Seconds(end));
    ctrl_sender
        .add_track(TimeRange::new(start, end))
        .await
        .map(|id| id.0)
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn remove_track<'a>(ctrl_sender: State<'a, CtrlSender>, id: usize) -> Result<(), String> {
    ctrl_sender
        .remove_track(id.into())
        .await
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn set_track_time_range<'a>(
    ctrl_sender: State<'a, CtrlSender>,
    id: usize,
    start: TimeBaseType,
    end: Option<TimeBaseType>,
) -> Result<(), String> {
    let start = TimeUnit::Seconds(start);
    let end = end.map(|end| TimeUnit::Seconds(end));
    ctrl_sender
        .set_track_time_range(id.into(), TimeRange::new(start, end))
        .await
        .map_err(err_to_string)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventProps {
    id: usize,
    note: u8,
    vel: f32,
    start: TimeBaseType,
    end: TimeBaseType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "tag")]
pub enum NodeProps {
    #[serde(rename_all = "camelCase")]
    Osc {
        n_channels: Option<u16>,
        mode: String,
        // Duty cycle
        ds: Option<f32>,
        freq: f32,
        _phase: Option<f32>,
        mul: Option<f32>,
    },
    SimpleMixer {
        n_channels: Option<u16>,
        muls: Vec<f32>,
    },
    #[serde(rename_all = "camelCase")]
    SimpleFilter {
        n_channels: Option<u16>,
        mode: String,
        freq: f32,
    },
    Stereo,
    #[serde(rename_all = "camelCase")]
    Delay {
        n_channels: Option<u16>,
        delay: TimeBaseType,
        mul: f32,
    },
    #[serde(rename_all = "camelCase")]
    Sampler {
        storage_id: usize,
    },
    #[serde(rename_all = "camelCase")]
    PianoRoll {
        events: Vec<EventProps>,
    },
    #[serde(rename_all = "camelCase")]
    Adsr {
        a: f32,
        d: f32,
        s: f32,
        r: f32,
    },
}

fn get_channel_mask(n_channels: Option<u16>) -> Result<ChannelPositionsMask, String> {
    if let Some(n_channels) = n_channels {
        if n_channels == 1u16 {
            Ok(ChannelPositionsMask::FRONT_LEFT)
        } else if n_channels == 2u16 {
            Ok(ChannelPositionsMask::FRONT_LEFT | ChannelPositionsMask::FRONT_RIGHT)
        } else {
            Err("Only stero/mono supported".into())
        }
    } else {
        Ok(ChannelPositionsMask::FRONT_LEFT | ChannelPositionsMask::FRONT_RIGHT)
    }
}

fn new_builder<'a>(
    storage: State<'a, Mutex<Storage>>,
    props: NodeProps,
) -> Result<Box<dyn NodeBuilderTrait>, String> {
    match props {
        NodeProps::Osc {
            n_channels,
            mode,
            ds,
            freq,
            _phase,
            mul,
        } => match mode.as_str() {
            "Sine" => Ok(Box::new(
                nodes::OscProps::new_sin(
                    get_channel_mask(n_channels)?,
                    freq * std::f32::consts::TAU,
                    mul.unwrap_or_else(|| 1f32),
                )
                .map_err(err_to_string)?,
            )),
            "Square" => Ok(Box::new(
                nodes::OscProps::new_square(
                    get_channel_mask(n_channels)?,
                    ds.unwrap_or_else(|| 0.5f32),
                    freq * std::f32::consts::TAU,
                    mul.unwrap_or_else(|| 1f32),
                )
                .map_err(err_to_string)?,
            )),
            "Saw" => Ok(Box::new(
                nodes::OscProps::new_saw(
                    get_channel_mask(n_channels)?,
                    ds.unwrap_or_else(|| 0.5f32),
                    freq * std::f32::consts::TAU,
                    mul.unwrap_or_else(|| 1f32),
                )
                .map_err(err_to_string)?,
            )),
            _ => return Err("Invalid mode".into()),
        },
        NodeProps::SimpleMixer { n_channels, muls } => Ok(Box::new(nodes::SimpleMixerProps::new(
            get_channel_mask(n_channels)?,
            muls,
        ))),
        NodeProps::SimpleFilter {
            n_channels,
            mode,
            freq,
        } => match mode.as_str() {
            "LPF" => Ok(Box::new(
                nodes::SimpleFilterProps::new_lpf(get_channel_mask(n_channels)?, freq)
                    .map_err(err_to_string)?,
            )),
            "HPF" => Ok(Box::new(
                nodes::SimpleFilterProps::new_hpf(get_channel_mask(n_channels)?, freq)
                    .map_err(err_to_string)?,
            )),
            "BPF" => Ok(Box::new(
                nodes::SimpleFilterProps::new_bpf(get_channel_mask(n_channels)?, freq)
                    .map_err(err_to_string)?,
            )),
            "BSF" => Ok(Box::new(
                nodes::SimpleFilterProps::new_bsf(get_channel_mask(n_channels)?, freq)
                    .map_err(err_to_string)?,
            )),
            _ => Err("Invalid mode".into()),
        },
        NodeProps::Stereo => Ok(Box::new(nodes::StereoProps::default())),
        NodeProps::Delay {
            n_channels,
            delay,
            mul,
        } => Ok(Box::new(
            nodes::DelayProps::new(get_channel_mask(n_channels)?, TimeUnit::Seconds(delay), mul)
                .map_err(err_to_string)?,
        )),
        NodeProps::Sampler { storage_id } => {
            let info = {
                let guard = storage.lock().unwrap();
                guard
                    .get(storage_id)
                    .ok_or_else(|| "Invalid id".to_string())?
            };
            Ok(Box::new(
                nodes::SamplerProps::new(info).map_err(err_to_string)?,
            ))
        }
        NodeProps::PianoRoll {
            events: mut event_props,
        } => {
            let mut events = Vec::<MidiMsg>::with_capacity(event_props.len() * 2);
            for event_prop in event_props.drain(..) {
                let note = MidiNote::from_midi_num(event_prop.note).map_err(err_to_string)?;
                let data_on = MidiEvent::NoteOn {
                    note,
                    vel: event_prop.vel,
                };
                let data_off = MidiEvent::NoteOff {
                    note,
                    vel: event_prop.vel,
                };
                events.push(MidiMsg::new(
                    event_prop.id.into(),
                    TimeUnit::Seconds(event_prop.start),
                    data_on,
                ));
                events.push(MidiMsg::new(
                    event_prop.id.into(),
                    TimeUnit::Seconds(event_prop.end),
                    data_off,
                ));
            }
            events.sort_by(|a, b| a.time.to_samples(44100).cmp(&b.time.to_samples(44100)));
            Ok(Box::new(nodes::PianoRollProps::new(events)))
        }
        NodeProps::Adsr { a, d, s, r } => Ok(Box::new(nodes::AdsrProps::new(a, d, s, r))),
    }
}

#[tauri::command]
pub async fn add_node<'a>(
    storage: State<'a, Mutex<Storage>>,
    ctrl_sender: State<'a, CtrlSender>,
    track_id: usize,
    props: NodeProps,
) -> Result<usize, String> {
    ctrl_sender
        .add_node(track_id.into(), new_builder(storage, props)?)
        .await
        .map(|id| id.0)
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn replace_node<'a>(
    storage: State<'a, Mutex<Storage>>,
    ctrl_sender: State<'a, CtrlSender>,
    id: usize,
    props: NodeProps,
) -> Result<bool, String> {
    ctrl_sender
        .replace_node(id.into(), new_builder(storage, props)?)
        .await
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn remove_node<'a>(ctrl_sender: State<'a, CtrlSender>, id: usize) -> Result<(), String> {
    ctrl_sender
        .remove_node(id.into())
        .await
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn set_output_port<'a>(
    ctrl_sender: State<'a, CtrlSender>,
    port: Option<(usize, usize)>,
) -> Result<(), String> {
    let port: Option<(NodeId, PortId)> = port.map(|port| (port.0.into(), port.1.into()));
    ctrl_sender
        .set_output_port(port)
        .await
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn connect_ports<'a>(
    ctrl_sender: State<'a, CtrlSender>,
    source: (usize, usize),
    target: (usize, usize),
) -> Result<(), String> {
    ctrl_sender
        .connect_ports(
            (source.0.into(), source.1.into()),
            (target.0.into(), target.1.into()),
        )
        .await
        .map_err(err_to_string)
}

#[tauri::command]
pub async fn unlink_port<'a>(
    ctrl_sender: State<'a, CtrlSender>,
    target: (usize, usize),
) -> Result<(), String> {
    ctrl_sender
        .unlink_port((target.0.into(), target.1.into()))
        .await
        .map_err(err_to_string)
}
