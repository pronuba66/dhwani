use std::sync::Arc;

use crate::{
    Error,
    controller::{ctrl_buffer::CtrlBuffer, ctrl_receiver::CtrlRsp},
    node::{NodeBuilderTrait, NodeId},
    nodes::NodeInfo,
    port::{Port, PortId},
    time::{TimeFrom, TimeRange},
    track::{Track, TrackId},
};

pub(crate) enum CtrlReq {
    Quit,
    Play(bool),
    Time(TimeFrom),
    Clear,
    GetTrackCount,
    GetTracks(Vec<Track>),
    GetNodeCount(TrackId),
    GetNodes(TrackId, Vec<NodeInfo>),
    AddTrack(TimeRange),
    RemoveTrack(TrackId),
    SetTrackTimeRange(TrackId, TimeRange),
    AddNode(TrackId, Box<dyn NodeBuilderTrait>),
    ReplaceNode(NodeId, Box<dyn NodeBuilderTrait>),
    RemoveNode(NodeId),
    GetPortCount(NodeId),
    GetPorts(NodeId, Vec<Port>),
    GetOutputPort,
    SetOutputPort(Option<(NodeId, PortId)>),
    ConnectPorts {
        source: (NodeId, PortId),
        target: (NodeId, PortId),
    },
    UnlinkPort {
        target: (NodeId, PortId),
    },
    ConnectNodes {
        source: NodeId,
        target: NodeId,
    },
    #[cfg(feature = "debug")]
    DebugVisualize(String),
}

/// Unbounded producer for controller buffer
#[derive(Clone)]
pub struct CtrlSender {
    buffer: Arc<CtrlBuffer>,
}

impl CtrlSender {
    pub(crate) fn new(buffer: Arc<CtrlBuffer>) -> Self {
        Self { buffer }
    }

    pub(crate) fn quit(&self) -> Result<(), Error> {
        match self.buffer.send_blocking(CtrlReq::Quit)? {
            CtrlRsp::Quit => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Play/Pause
    pub async fn play(&self, enable: bool) -> Result<(), Error> {
        match self.buffer.send(CtrlReq::Play(enable)).await? {
            CtrlRsp::Play(_) => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Set current time
    pub async fn set_time(&self, time: TimeFrom) -> Result<f64, Error> {
        match self.buffer.send(CtrlReq::Time(time)).await? {
            CtrlRsp::Time(seconds) => Ok(seconds),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Clear all tracks, nodes, connections and reset the all states to default
    pub async fn clear(&self) -> Result<(), Error> {
        self.play(false).await?;
        match self.buffer.send(CtrlReq::Clear).await? {
            CtrlRsp::Clear => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn get_track_count(&self) -> Result<usize, Error> {
        match self.buffer.send(CtrlReq::GetTrackCount).await? {
            CtrlRsp::GetTrackCount(count) => Ok(count),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn get_tracks(&self, tracks: Vec<Track>) -> Result<Vec<Track>, Error> {
        match self.buffer.send(CtrlReq::GetTracks(tracks)).await? {
            CtrlRsp::GetTracks(tracks) => Ok(tracks),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn get_node_count(&self, track_id: TrackId) -> Result<usize, Error> {
        match self.buffer.send(CtrlReq::GetNodeCount(track_id)).await? {
            CtrlRsp::GetNodeCount(count) => Ok(count),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn get_nodes(
        &self,
        track_id: TrackId,
        nodes: Vec<NodeInfo>,
    ) -> Result<Vec<NodeInfo>, Error> {
        match self.buffer.send(CtrlReq::GetNodes(track_id, nodes)).await? {
            CtrlRsp::GetNodes(nodes) => Ok(nodes),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Add track for time range [`TimeRange`]
    pub async fn add_track(&self, time_range: TimeRange) -> Result<TrackId, Error> {
        match self.buffer.send(CtrlReq::AddTrack(time_range)).await? {
            CtrlRsp::AddTrack(id) => Ok(id),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Rmove track for [`TrackId`], its child nodes and relevent connections
    pub async fn remove_track(&self, id: TrackId) -> Result<(), Error> {
        match self.buffer.send(CtrlReq::RemoveTrack(id)).await? {
            CtrlRsp::RemoveTrack => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Rmove track for [`TrackId`], its child nodes and relevent connections
    pub async fn set_track_time_range(
        &self,
        id: TrackId,
        time_range: TimeRange,
    ) -> Result<(), Error> {
        match self
            .buffer
            .send(CtrlReq::SetTrackTimeRange(id, time_range))
            .await?
        {
            CtrlRsp::SetTrackTimeRange => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Add node inside track for [`TrackId`] with [`NodeBuilderTrait`]
    pub async fn add_node(
        &self,
        track_id: TrackId,
        builder: Box<dyn NodeBuilderTrait>,
    ) -> Result<NodeId, Error> {
        match self
            .buffer
            .send(CtrlReq::AddNode(track_id, builder))
            .await?
        {
            CtrlRsp::AddNode(id) => Ok(id),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Replace node with [`NodeBuilderTrait`]
    pub async fn replace_node(
        &self,
        id: NodeId,
        builder: Box<dyn NodeBuilderTrait>,
    ) -> Result<bool, Error> {
        match self.buffer.send(CtrlReq::ReplaceNode(id, builder)).await? {
            CtrlRsp::ReplaceNode(invalidate_connections) => Ok(invalidate_connections),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    /// Rmove node for [`NodeId`], its child nodes and relevent connections
    pub async fn remove_node(&self, id: NodeId) -> Result<(), Error> {
        match self.buffer.send(CtrlReq::RemoveNode(id)).await? {
            CtrlRsp::RemoveNode => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn get_port_count(&self, node_id: NodeId) -> Result<usize, Error> {
        match self.buffer.send(CtrlReq::GetPortCount(node_id)).await? {
            CtrlRsp::GetPortCount(count) => Ok(count),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn get_ports(&self, node_id: NodeId, ports: Vec<Port>) -> Result<Vec<Port>, Error> {
        match self.buffer.send(CtrlReq::GetPorts(node_id, ports)).await? {
            CtrlRsp::GetPorts(ports) => Ok(ports),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn get_output_port(&self) -> Result<Option<Port>, Error> {
        match self.buffer.send(CtrlReq::GetOutputPort).await? {
            CtrlRsp::GetOutputPort(port) => Ok(port),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn set_output_port(&self, port: Option<(NodeId, PortId)>) -> Result<(), Error> {
        match self.buffer.send(CtrlReq::SetOutputPort(port)).await? {
            CtrlRsp::SetOutputPort => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn connect_ports(
        &self,
        source: (NodeId, PortId),
        target: (NodeId, PortId),
    ) -> Result<(), Error> {
        match self
            .buffer
            .send(CtrlReq::ConnectPorts { source, target })
            .await?
        {
            CtrlRsp::ConnectPorts => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn unlink_port(&self, target: (NodeId, PortId)) -> Result<(), Error> {
        match self.buffer.send(CtrlReq::UnlinkPort { target }).await? {
            CtrlRsp::UnlinkPort => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    pub async fn connect_nodes(&self, source: NodeId, target: NodeId) -> Result<(), Error> {
        match self
            .buffer
            .send(CtrlReq::ConnectNodes { source, target })
            .await?
        {
            CtrlRsp::ConnectNodes => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }

    #[cfg(feature = "debug")]
    pub async fn debug_render_graph(&self, path: String) -> Result<(), Error> {
        match self.buffer.send(CtrlReq::DebugVisualize(path)).await? {
            CtrlRsp::DebugVisualize => Ok(()),
            _ => Err(Error::msg("Invalid response".into())),
        }
    }
}
