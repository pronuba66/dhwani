use std::{cell::Cell, fmt::Debug, marker::PhantomData, sync::Arc};

use crate::{
    Error, Processor,
    controller::{
        ctrl_buffer::{CtrlBuffer, CtrlReqParam, CtrlRspParam},
        ctrl_sender::CtrlReq,
    },
    node::NodeId,
    nodes::NodeInfo,
    port::Port,
    time::TimeBaseType,
    track::{Track, TrackId},
};

#[derive(Debug)]
pub(crate) enum CtrlRsp {
    Quit,
    Play(bool),
    Time(TimeBaseType),
    Clear,
    GetTrackCount(usize),
    GetTracks(Vec<Track>),
    GetNodeCount(usize),
    GetNodes(Vec<NodeInfo>),
    AddTrack(TrackId),
    RemoveTrack,
    SetTrackTimeRange,
    AddNode(NodeId),
    ReplaceNode(bool),
    RemoveNode,
    GetPortCount(usize),
    GetPorts(Vec<Port>),
    GetOutputPort(Option<Port>),
    SetOutputPort,
    ConnectPorts,
    UnlinkPort,
    ConnectNodes,
    #[cfg(feature = "debug")]
    DebugVisualize,
}

pub(crate) struct CtrlPendingReq {
    pub(crate) rsp: CtrlRsp,
    rsp_param: CtrlRspParam,
}

impl Debug for CtrlPendingReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{rsp: {:?}}}", self.rsp)
    }
}

impl CtrlPendingReq {
    pub(crate) fn send(self) {
        self.rsp_param.send(Ok(self.rsp));
    }

    pub(crate) fn send_err(self, e: Error) {
        self.rsp_param.send(Err(e));
    }
}

/// Unbounded consumer for controller buffer
pub struct CtrlReceiver {
    buffer: Arc<CtrlBuffer>,
    _not_send_and_sync: PhantomData<Cell<()>>,
}

impl CtrlReceiver {
    pub(crate) fn new(buffer: Arc<CtrlBuffer>) -> Self {
        Self {
            buffer,
            _not_send_and_sync: PhantomData,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Receives controller req. If not avaialble, the method waits till data available.
    ///
    #[allow(dead_code)]
    pub(crate) fn recv(&self) -> CtrlReqParam {
        self.buffer.recv()
    }

    /// Receives controller req if data is avaialble.
    ///
    pub(crate) fn try_recv(&self) -> Option<CtrlReqParam> {
        self.buffer.try_recv()
    }

    pub(crate) fn process_request(
        processor: &mut Processor,
        ctrl_params: CtrlReqParam,
    ) -> Option<CtrlPendingReq> {
        let (ctrl, rsp_param) = ctrl_params.split();
        let rsp = match ctrl {
            CtrlReq::Quit => {
                processor.stop();
                Ok(CtrlRsp::Quit)
            }
            CtrlReq::Play(enable) => {
                processor.set_playing_with_fader(enable);
                Ok(CtrlRsp::Play(enable))
            }
            CtrlReq::Time(time) => Ok(CtrlRsp::Time(processor.seek(time))),
            CtrlReq::Clear => {
                processor.clear();
                Ok(CtrlRsp::Clear)
            }
            CtrlReq::GetTrackCount => Ok(CtrlRsp::GetTrackCount(processor.get_track_count())),
            CtrlReq::GetTracks(mut tracks) => {
                processor.get_tracks(&mut tracks);
                Ok(CtrlRsp::GetTracks(tracks))
            }
            CtrlReq::GetNodeCount(track_id) => {
                Ok(CtrlRsp::GetNodeCount(processor.get_node_count(track_id)))
            }
            CtrlReq::GetNodes(track_id, mut nodes) => {
                processor.get_node_infos(track_id, &mut nodes);
                Ok(CtrlRsp::GetNodes(nodes))
            }
            CtrlReq::AddTrack(time_range) => processor
                .add_track(time_range)
                .map(|id| CtrlRsp::AddTrack(id)),
            CtrlReq::RemoveTrack(id) => processor.remove_track(id).map(|_| CtrlRsp::RemoveTrack),
            CtrlReq::SetTrackTimeRange(id, time_range) => processor
                .set_track_time_range(id, time_range)
                .map(|_| CtrlRsp::SetTrackTimeRange),
            CtrlReq::AddNode(track_id, builder) => processor
                .add_node(track_id, builder.as_ref())
                .map(|id| CtrlRsp::AddNode(id)),
            CtrlReq::ReplaceNode(id, builder) => processor
                .replace_node(id, builder.as_ref())
                .map(|invalidate_connections| CtrlRsp::ReplaceNode(invalidate_connections)),
            CtrlReq::RemoveNode(id) => processor.remove_node(id).map(|_| CtrlRsp::RemoveNode),
            CtrlReq::GetPortCount(node_id) => processor
                .get_node(node_id)
                .ok_or_else(|| Error::msg("Node not found".into()))
                .map(|node| CtrlRsp::GetPortCount(node.ports().len())),
            CtrlReq::GetPorts(node_id, mut ports) => processor
                .get_node(node_id)
                .ok_or_else(|| Error::msg("Node not found".into()))
                .map(|node| {
                    ports.extend_from_slice(node.ports());
                    CtrlRsp::GetPorts(ports)
                }),
            CtrlReq::GetOutputPort => Ok(CtrlRsp::GetOutputPort(processor.get_output_port())),
            CtrlReq::SetOutputPort(port) => processor
                .set_output_port(port)
                .map(|_| CtrlRsp::SetOutputPort),
            CtrlReq::ConnectPorts { source, target } => processor
                .connect_ports(source, target)
                .map(|_| CtrlRsp::ConnectPorts),
            CtrlReq::UnlinkPort { target } => {
                processor.unlink_port(target).map(|_| CtrlRsp::UnlinkPort)
            }
            CtrlReq::ConnectNodes { source, target } => processor
                .connect_nodes(source, target)
                .map(|_| CtrlRsp::ConnectNodes),
            #[cfg(feature = "debug")]
            CtrlReq::DebugVisualize(path) => {
                processor.debug_render_graph(path);
                Ok(CtrlRsp::DebugVisualize)
            }
        };
        match rsp {
            Ok(rsp) => Some(CtrlPendingReq { rsp, rsp_param }),
            Err(e) => {
                // Send immediately if error
                rsp_param.send(Err(e));
                None
            }
        }
    }
}
