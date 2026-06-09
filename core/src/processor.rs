//! Owns the node graph and drives per-frame processing on the audio thread.
//! Constructed once; thereafter only mutated via [`controller::start_controller`] messages.

use std::collections::{HashMap, HashSet};

use crate::{
    Error,
    channel::ChannelPosition,
    connection::Connection,
    event::Events,
    frame::Frame,
    node::{Node, NodeBuilderTrait, NodeId, NodeInputs, NodeOutputs},
    nodes::NodeInfo,
    port::{Port, PortId, PortType},
    signal::SingalsFrame,
    time::{ResolvedTimeRange, SampleBaseType, SampleRateBaseType, TimeFrom, TimeRange, TimeUnit},
    track::{Track, TrackId},
};

#[derive(Debug, PartialEq)]
enum ProcessorState {
    Paused,
    Playing,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
enum FadeMode {
    In,
    Out,
}

#[derive(Debug, Clone, Copy)]
struct Fader {
    mode: FadeMode,
    step: usize,
    duration: usize,
}

impl IntoIterator for Fader {
    type Item = f32;
    type IntoIter = FaderIter;

    fn into_iter(self) -> Self::IntoIter {
        FaderIter {
            mode: self.mode,
            step: self.step,
            duration: self.duration,
        }
    }
}

impl Fader {
    pub const fn new(mode: FadeMode, duration: usize) -> Self {
        Self {
            mode,
            // Already make the fader state = done
            step: duration,
            duration,
        }
    }

    const fn remaining(&self) -> usize {
        self.duration - self.step
    }

    const fn done(&self) -> bool {
        self.step >= self.duration
    }

    const fn fade_in(self) -> Self {
        Self {
            mode: FadeMode::In,
            step: 0,
            ..self
        }
    }

    /// New fader with old settings for starting fade out.
    const fn fade_out(self) -> Self {
        Self {
            mode: FadeMode::Out,
            step: 0,
            ..self
        }
    }

    pub const fn update(&mut self, step: usize) {
        self.step += step;
        if self.step > self.duration {
            self.step = self.duration;
        }
    }
}

struct FaderIter {
    mode: FadeMode,
    step: usize,
    duration: usize,
}

impl Iterator for FaderIter {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let mul = match self.mode {
            FadeMode::Out => {
                if self.step >= self.duration {
                    0f32
                } else {
                    let mul = (self.duration - self.step) as f32 / self.duration as f32;
                    self.step += 1;
                    mul
                }
            }
            FadeMode::In => {
                if self.step > self.duration {
                    1f32
                } else {
                    let mul = self.step as f32 / self.duration as f32;
                    self.step += 1;
                    mul
                }
            }
        };
        Some(mul)
    }
}

pub struct Processor {
    state: ProcessorState,
    sr: SampleRateBaseType,
    n_channels: u16,
    buffer_size: usize,
    pub step: SampleBaseType,
    id: usize,
    pub(crate) nodes_map: HashMap<NodeId, Node>,
    pub(crate) tracks_map: HashMap<TrackId, Track>,
    /// Unlike [`Processor::frames_maps`], this is flattened for optimal insert/access
    pub(crate) connections_map: HashMap<(NodeId, PortId), Connection>,
    pub(crate) frames_maps: HashMap<NodeId, HashMap<PortId, Frame>>,
    // Compiled connection maps. This would not contain proxy ports
    compiled_connections_map: HashMap<(NodeId, PortId), Connection>,
    // The pointer to nodes sorted in DFS order for processing
    nodes: Vec<NodeId>,
    // Final port
    output_port: Option<Port>,
    end_time: SampleBaseType,
    invalidate: bool,
    fader: Fader,
}

impl Processor {
    const TRACKS_MAP_CAPACITY: usize = 32;
    const NODES_MAP_CAPACITY: usize = 512;
    const CONNECTIONS_MAP_CAPACITY: usize = 1024;
    const FRAMES_MAPS_CAPACITY: usize = 1024;
    const FADE_DURATION: f64 = 0.020f64; // 20ms

    #[must_use]
    pub fn new(sr: SampleRateBaseType, n_channels: u16, buffer_size: usize) -> Self {
        let mut processor = Self {
            state: ProcessorState::Paused,
            sr,
            n_channels,
            buffer_size,
            step: 0 as SampleBaseType,
            id: 0,
            tracks_map: HashMap::with_capacity(Self::TRACKS_MAP_CAPACITY),
            nodes_map: HashMap::with_capacity(Self::NODES_MAP_CAPACITY),
            connections_map: HashMap::with_capacity(Self::CONNECTIONS_MAP_CAPACITY),
            frames_maps: HashMap::with_capacity(Self::FRAMES_MAPS_CAPACITY),
            compiled_connections_map: HashMap::with_capacity(Self::CONNECTIONS_MAP_CAPACITY),
            nodes: vec![],
            output_port: None,
            end_time: 0 as SampleBaseType,
            invalidate: true,
            fader: Fader::new(
                FadeMode::In,
                // 10ms fader
                TimeUnit::Seconds(Self::FADE_DURATION).to_samples(sr) as usize,
            ),
        };
        processor.reset();
        processor
    }

    fn process_fader(&mut self, chunk: usize) {
        self.fader.update(chunk);
        debug_assert!(!matches!(self.state, ProcessorState::Stopped));
        if self.fader.done() {
            match self.fader.mode {
                FadeMode::In => {
                    self.set_playing(true);
                }
                FadeMode::Out => {
                    self.set_playing(false);
                }
            }
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.set_playing(false);
        self.step = 0;
        self.tracks_map.clear();
        self.connections_map.clear();
        self.nodes_map.clear();
        self.id = 0;
        self.output_port = None;
    }

    #[inline]
    pub(crate) fn new_id(&mut self) -> usize {
        let id = self.id;
        assert!(self.id != usize::MAX, "ID exhausted!");
        self.id += 1;
        id
    }

    fn remove_node_connections(&mut self, id: NodeId) {
        // Remove exisiting connections
        self.connections_map.retain(|_, connection| {
            connection.source.node_id != id && connection.target.node_id != id
        });
    }

    pub(crate) fn remove_nodes(&mut self, root_id: NodeId, remove_connection: bool) {
        // Remove output node
        if remove_connection
            && self
                .output_port
                .as_ref()
                .is_some_and(|output_port| output_port.node_id == root_id)
        {
            self.output_port = None
        }
        let mut stack = Vec::<NodeId>::with_capacity(self.nodes_map.capacity());
        stack.push(root_id);
        while let Some(id) = stack.pop() {
            self.nodes_map.remove(&id);
            self.frames_maps.remove(&id);
            if remove_connection {
                self.remove_node_connections(id);
            }
            for (child_id, _) in self
                .nodes_map
                .extract_if(|_, node| node.parent_id() == Some(id))
            {
                stack.push(child_id);
            }
        }
    }

    pub(crate) fn build_frames(&self, node: &Node) -> HashMap<PortId, Frame> {
        let mut frames: HashMap<PortId, Frame> = HashMap::<PortId, Frame>::new();
        for port in node.ports() {
            match port.kind {
                PortType::EventsIn | PortType::SignalIn | PortType::Proxy(_) => {}
                PortType::EventsOut => {
                    // Lets keep minimum of 64 events per frame
                    let frame = Frame::Events(Events::new(self.min_events_per_frame()));
                    frames.insert(port.id, frame);
                }
                PortType::SignalOut(channel_mask) => {
                    let frame = Frame::Signals(SingalsFrame::new(channel_mask, self.buffer_size));
                    frames.insert(port.id, frame);
                }
            }
        }
        frames
    }

    fn clear_frames(&mut self, id: NodeId) {
        if let Some(frames) = self.frames_maps.get_mut(&id) {
            for frame in frames.values_mut() {
                match frame {
                    Frame::Events(events) => {
                        events.clear();
                    }
                    Frame::Signals(frame) => frame.clear(),
                }
            }
        }
    }

    pub(crate) fn is_child(&self, parent_id: NodeId, id: NodeId) -> bool {
        let mut node = self.nodes_map.get(&id);
        while let Some(id) = node.and_then(Node::parent_id) {
            if id == parent_id {
                return true;
            }
            node = self.nodes_map.get(&id);
        }
        false
    }

    pub(crate) fn reverse_port(&self, mut port: Port) -> Result<Port, Error> {
        while let PortType::Proxy(port_proxy) = port.kind {
            let node = self
                .nodes_map
                .get(&port_proxy.node_id)
                .ok_or_else(|| Error::msg("Failed to reverse port".into()))?;
            port = node
                .get_port(port_proxy.port_id)
                .ok_or_else(|| Error::msg("Failed to reverse port".into()))?;
        }
        Ok(port)
    }

    ///
    /// # Errors
    /// Return `Err` if buffer len is invalid.
    /// Buffer length must be a multiple of `self.n_channels` and must
    /// be <= `self.n_channels` * `self.buffer_size`
    ///
    pub fn process(&mut self, buffer: &mut [f32]) -> Result<usize, Error> {
        let n_channels = usize::from(self.n_channels);
        if !buffer.len().is_multiple_of(n_channels) {
            return Err(Error::msg(
                "Buffer must be mulutple of number of channels".to_string(),
            ));
        }
        let buffer_size = buffer.len() / n_channels;
        if buffer_size > self.buffer_size {
            return Err(Error::msg(format!(
                "Buffer size must be <= number of channels * {}",
                self.buffer_size
            )));
        }
        if !self.is_playing() {
            buffer.fill(0f32);
            return Ok(buffer.len());
        }
        // So that fading completes exactly and command can be processed
        let fader_rem = self.fader.remaining();
        let chunk = if matches!(self.fader.mode, FadeMode::Out) && fader_rem > 0 {
            buffer_size.min(fader_rem)
        } else {
            buffer_size
        };
        if self.invalidate {
            self.recompile_graph();
            self.invalidate = false;
        }
        let start_sample = self.step;
        let end_sample = start_sample + chunk as SampleBaseType;
        for i in 0..self.nodes.len() {
            let node_id = *self.nodes.get(i).unwrap();
            // Temporarly take the item from hashmap
            let mut node = self.nodes_map.remove(&node_id).unwrap();
            // Compute time related
            let (track_start_sample, track_end_sample) = {
                let time_range = self.tracks_map.get(&node.track_id()).unwrap().time_range();
                (
                    time_range.start().to_samples(self.sr),
                    time_range
                        .end()
                        .map(|t| t.to_samples(self.sr))
                        .unwrap_or_else(|| end_sample),
                )
            };
            let new_start_sample = start_sample.max(track_start_sample);
            let new_end_sample = end_sample.min(track_end_sample);
            if new_start_sample >= new_end_sample {
                // Clear frame if not invalidated.
                // Lets say a node A is from time [a,b] and  node B is from [c,d]
                // where [a,b] < [c,d] and A is connected to B. When processing B,
                // B will fetch values from A, but it might be a previous value
                // So need to clear it
                if !node.frames_invalidated {
                    node.frames_invalidated = true;
                    self.clear_frames(node_id);
                }
                // Insert back
                self.nodes_map.insert(node_id, node);
                continue;
            };
            let node_time_range = ResolvedTimeRange::try_from_time_range(
                self.sr,
                TimeRange::new(new_start_sample.into(), Some(new_end_sample.into())),
            )
            .unwrap();
            node.frames_invalidated = false;
            let range_start = (node_time_range.start() - start_sample) as usize;
            let range_end = range_start + node_time_range.duraiton();
            let range = range_start..range_end;
            // Temporarly take the item from hashmap
            let mut frames = self.frames_maps.remove(&node_id).unwrap();
            let node_inputs = NodeInputs::new(
                range.clone(),
                &self.frames_maps,
                node_id,
                &self.compiled_connections_map,
            );
            let mut node_outputs = NodeOutputs::new(range.clone(), &mut frames);
            node_outputs.clear_buffers();
            node.inner
                .process(node_time_range, &node_inputs, &mut node_outputs);
            // Insert back
            self.frames_maps.insert(node_id, frames);
            // Insert back
            self.nodes_map.insert(node_id, node);
        }
        if let Some(output_port) = self.output_port {
            let signals = self
                .frames_maps
                .get(&output_port.node_id)
                .unwrap()
                .get(&output_port.id)
                .unwrap()
                .get_signals(0..((end_sample - start_sample) as usize))
                .unwrap();
            if let Some(signal) = signals.get(ChannelPosition::FrontLeft) {
                let mut fader = self.fader.into_iter();
                for i in 0..chunk {
                    let mul = fader.next().unwrap();
                    buffer[n_channels * i] = signal[i] * mul;
                }
            }
            if let Some(signal) = signals.get(ChannelPosition::FrontRight) {
                let mut fader = self.fader.into_iter();
                for i in 0..chunk {
                    let mul = fader.next().unwrap();
                    buffer[(n_channels * i) + 1] = signal[i] * mul;
                }
            }
        }
        self.process_fader(chunk);
        self.step = end_sample;
        Ok(n_channels * chunk)
    }

    fn recompile_connections(&mut self) {
        let mut compiled_connections_map = self.connections_map.clone();
        for (&(node_id, port_id), connection) in &self.connections_map {
            let source = self.reverse_port(connection.source).unwrap();
            if source == connection.source {
                continue;
            }
            let connection = Connection::new(source, connection.target).unwrap();
            compiled_connections_map.insert((node_id, port_id), connection);
        }
        self.compiled_connections_map = compiled_connections_map;
    }

    fn recompile_graph_iter(
        &mut self,
        target_node_id: NodeId,
        node_status: &mut HashSet<NodeId>,
        connections_map: &mut HashMap<NodeId, Vec<PortId>>,
    ) {
        // Remove temporarily
        if let Some(connections) = connections_map.remove(&target_node_id) {
            for &target_port_id in &connections {
                let &connection = self
                    .compiled_connections_map
                    .get(&(target_node_id, target_port_id))
                    .unwrap();
                debug_assert!(
                    connection.target.id == target_port_id,
                    "Target ID not valid"
                );
                if node_status.insert(connection.source.node_id) {
                    self.recompile_graph_iter(
                        connection.source.node_id,
                        node_status,
                        connections_map,
                    );
                    self.nodes.push(connection.source.node_id);
                }
            }
            // Put it back
            connections_map.insert(target_node_id, connections);
        }
    }

    fn recompile_graph(&mut self) {
        self.nodes = Vec::<NodeId>::with_capacity(self.nodes_map.len());
        let mut node_status = HashSet::<NodeId>::with_capacity(self.nodes_map.len());
        self.end_time = 0;
        for track in self.tracks_map.values() {
            if let Some(end) = track.time_range().end() {
                let end_time = end.to_samples(self.sr);
                // Set end time the maximum
                self.end_time = end_time.max(self.end_time);
            }
        }
        self.recompile_connections();
        if let Some(output_port) = self.output_port {
            // All nodes put into self.nodes in processing order
            let mut connections_map = HashMap::<NodeId, Vec<PortId>>::new();
            for connection in self.compiled_connections_map.values() {
                connections_map
                    .entry(connection.target.node_id)
                    .or_default()
                    .push(connection.target.id);
            }
            self.recompile_graph_iter(output_port.node_id, &mut node_status, &mut connections_map);
            self.nodes.push(output_port.node_id);
        }
        // #[cfg(debug_assertions)]
        // {
        //     println!("Tracks");
        //     for track in self.tracks_map.values() {
        //         println!("{:?}", track);
        //     }
        //     println!("Nodes");
        //     for node in self.nodes_map.values() {
        //         println!("{node:?}");
        //     }
        //     println!("Nodes in process");
        //     for id in &self.nodes {
        //         let node = self.nodes_map.get(id).unwrap();
        //         println!("{node:?}");
        //     }
        //     println!("Connection map");
        //     println!("{{");
        //     for ((node_id, port_id), connection) in &self.connections_map {
        //         println!("    connection: {node_id:?}, {port_id:?}, {connection:?}");
        //     }
        //     println!("}}")
        // }
    }

    /// Adds a new track
    ///
    /// Returns the [`Result`] of [`TrackId`]
    ///
    /// # Arguments
    /// * `builder` - [`NodeBuilderTrait`] object
    ///
    /// # Errors
    /// Returns error if track already exists
    ///
    pub fn add_track(&mut self, time_range: TimeRange) -> Result<TrackId, Error> {
        let id: TrackId = self.new_id().into();
        self.tracks_map.insert(id, Track::new(id, time_range));
        self.invalidate = true;
        Ok(id)
    }

    /// Adds a new node
    ///
    /// Returns the [`Result`] to reference to node
    ///
    /// # Arguments
    /// * `builder` - [`NodeBuilderTrait`] object
    ///
    /// # Errors
    /// Returns error if track does not exits
    ///
    pub fn add_node(
        &mut self,
        track_id: TrackId,
        builder: &dyn NodeBuilderTrait,
    ) -> Result<NodeId, Error> {
        self.add_node_with_parent(track_id, None, builder)
    }

    pub(crate) fn add_node_with_parent(
        &mut self,
        track_id: TrackId,
        parent_id: Option<NodeId>,
        builder: &dyn NodeBuilderTrait,
    ) -> Result<NodeId, Error> {
        let id = Node::add(self, track_id, parent_id, builder)?;
        self.invalidate = true;
        Ok(id)
    }

    /// Replace node
    ///
    /// Returns the [`Result`] to reference to node
    ///
    /// If previous connection are not valid, connections are removed
    ///
    /// If node validation fails, new node is not updated and the previous node
    /// stays as it is.
    ///
    /// # Arguments
    /// * `builder` - [`NodeBuilderTrait`] object
    ///
    /// # Errors
    /// Returns error if node does not exits
    ///
    pub fn replace_node(
        &mut self,
        id: NodeId,
        builder: &dyn NodeBuilderTrait,
    ) -> Result<bool, Error> {
        let invalidate_connections = Node::replace(self, id, builder)?;
        if invalidate_connections
            && self
                .output_port
                .is_some_and(|output_port| output_port.node_id == id)
        {
            self.output_port = None;
        }
        self.invalidate = true;
        Ok(invalidate_connections)
    }

    /// Remove node for `id` including its child nodes, related connections and
    /// frames
    ///
    /// # Arguments
    /// * `id` - [`NodeId`]
    ///
    /// # Errors
    /// Returns error if node not found for `id`
    ///
    pub fn remove_node(&mut self, id: NodeId) -> Result<(), Error> {
        let node = self
            .nodes_map
            .get(&id)
            .ok_or_else(|| Error::msg("Node not found".into()))?;
        if node.parent_id().is_some() {
            return Err(Error::msg("Child nodes can't be removed".into()));
        }
        self.remove_nodes(id, true);
        self.invalidate = true;
        Ok(())
    }

    pub fn remove_track(&mut self, id: TrackId) -> Result<Track, Error> {
        let track = self
            .tracks_map
            .remove(&id)
            .ok_or_else(|| Error::msg("Track not found".into()))?;
        let mut ids: Vec<NodeId> = self
            .nodes_map
            .iter()
            .filter(|(_, node)| node.track_id() == id)
            .map(|(&id, _)| id)
            .collect();
        for id in ids.drain(..) {
            self.remove_nodes(id, true);
        }
        self.invalidate = true;
        Ok(track)
    }

    pub fn set_track_time_range(
        &mut self,
        id: TrackId,
        time_range: TimeRange,
    ) -> Result<(), Error> {
        let track = self
            .tracks_map
            .get_mut(&id)
            .ok_or_else(|| Error::msg("Track not found".into()))?;
        track.set_time_range(time_range);
        self.invalidate = true;
        Ok(())
    }

    pub fn set_output_port(&mut self, port: Option<(NodeId, PortId)>) -> Result<(), Error> {
        if let Some(port) = port {
            let node = self
                .nodes_map
                .get(&port.0)
                .ok_or_else(|| Error::msg("Node not found".into()))?;
            if node.parent_id().is_some() {
                return Err(Error::msg(
                    "Port of child nodes can't be set as output".into(),
                ));
            }
            self.output_port = Some(
                node.get_port(port.1)
                    .ok_or_else(|| Error::msg("Port not found".into()))?,
            );
        } else {
            self.output_port = None;
        }
        self.invalidate = true;
        Ok(())
    }

    #[must_use]
    pub fn get_output_port(&self) -> Option<Port> {
        self.output_port
    }

    ///
    /// # Errors
    /// Return `Err` if source/target ports invalid
    ///
    pub fn connect_ports(
        &mut self,
        (source_node_id, source_port_id): (NodeId, PortId),
        (target_node_id, target_port_id): (NodeId, PortId),
    ) -> Result<(), Error> {
        let source_node = self
            .nodes_map
            .get(&source_node_id)
            .ok_or_else(|| Error::msg("Source node not found".into()))?;
        let source = source_node
            .get_port(source_port_id)
            .ok_or_else(|| Error::msg("Source port not found".into()))?;
        let target_node = self
            .nodes_map
            .get(&target_node_id)
            .ok_or_else(|| Error::msg("Target node not found".into()))?;
        let target = target_node
            .get_port(target_port_id)
            .ok_or_else(|| Error::msg("Target port not found".into()))?;
        let connection = Connection::new(source, target)?;
        self.connections_map
            .insert((target.node_id, target.id), connection);
        self.invalidate = true;
        Ok(())
    }

    ///
    /// # Errors
    /// Return `Err` if source/target ports invalid
    ///
    pub fn connect_nodes(&mut self, source_id: NodeId, target_id: NodeId) -> Result<(), Error> {
        let source_node = self
            .nodes_map
            .get(&source_id)
            .ok_or_else(|| Error::msg("Invalid source node".into()))?;
        // Find the first auto_connect output port
        let ports = source_node.ports();
        let mut source: Option<Port> = None;
        for port in ports {
            if port.auto_connect && port.kind.is_output() {
                source = Some(*port);
                break;
            }
        }
        let source =
            source.ok_or_else(|| Error::msg("No auto connect port in source node".into()))?;
        let target_node = self
            .nodes_map
            .get(&target_id)
            .ok_or_else(|| Error::msg("Invalid target node".into()))?;
        // Find the first auto_connect input port that is not already connected
        let ports = target_node.ports();
        let mut target: Option<Port> = None;
        for port in ports {
            if port.auto_connect
                && port.kind.is_input()
                && !self.connections_map.contains_key(&(port.node_id, port.id))
            {
                target = Some(*port);
                break;
            }
        }
        let target =
            target.ok_or_else(|| Error::msg("No auto connect port in target node".into()))?;
        // Do not worry about port type, or directions. It will be checked in [`Connection::new`]
        let connection = Connection::new(source, target)?;
        self.connections_map
            .insert((target.node_id, target.id), connection);
        self.invalidate = true;
        Ok(())
    }

    ///
    /// # Errors
    /// Return `Err` target port invalid
    ///
    pub fn unlink_port(
        &mut self,
        (target_node_id, target_port_id): (NodeId, PortId),
    ) -> Result<(), Error> {
        self.connections_map
            .remove(&(target_node_id, target_port_id))
            .ok_or_else(|| Error::msg("Invalid port".into()))?;
        self.invalidate = true;
        Ok(())
    }

    #[must_use]
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes_map.get(&id)
    }

    #[must_use]
    pub fn min_events_per_frame(&self) -> usize {
        (self.buffer_size() / 2).max(64)
    }

    pub fn clear(&mut self) {
        self.reset();
        self.invalidate = true;
    }

    pub fn get_track_count(&self) -> usize {
        self.tracks_map.len()
    }

    pub fn get_tracks(&self, tracks: &mut Vec<Track>) {
        for track in self.tracks_map.values() {
            tracks.push(track.clone());
        }
    }

    pub fn get_node_count(&self, track_id: TrackId) -> usize {
        self.nodes_map.iter().fold(0usize, |acc, (_, node)| {
            if node.track_id() == track_id && node.parent_id().is_none() {
                acc + 1
            } else {
                acc
            }
        })
    }

    pub fn get_node_infos(&self, track_id: TrackId, nodes: &mut Vec<NodeInfo>) {
        for node in self.nodes_map.values() {
            if node.track_id() == track_id && node.parent_id().is_none() {
                // Add only user created nodes
                nodes.push(NodeInfo {
                    id: node.id(),
                    track_id: node.track_id(),
                });
            }
        }
    }

    pub const fn stop(&mut self) {
        // Do not add to command queue, instead stop immidiately
        self.state = ProcessorState::Stopped;
    }

    pub const fn set_playing(&mut self, enable: bool) {
        self.state = if enable {
            ProcessorState::Playing
        } else {
            ProcessorState::Paused
        }
    }

    pub fn set_playing_with_fader(&mut self, enable: bool) {
        if enable && self.is_playing() {
            self.set_playing(true);
        } else if !enable && !self.is_playing() {
            self.set_playing(false);
        } else {
            if enable {
                self.set_playing(true);
                self.fader = self.fader.fade_in();
            } else {
                self.fader = self.fader.fade_out();
            }
        }
    }

    pub fn is_fader_done(&self) -> bool {
        self.fader.done()
    }

    pub fn seek(&mut self, time: TimeFrom) -> f64 {
        let last_step = self.step;
        match time {
            TimeFrom::Start(time) => {
                self.step = time.to_samples(self.sr);
            }
            TimeFrom::Current(time) => {
                self.step += time.to_samples(self.sr);
            }
            TimeFrom::End(time) => {
                self.step = self.end_time + time.to_samples(self.sr);
            }
        }
        if self.step != last_step {
            let time_unit = TimeUnit::Samples(self.step);
            for node in self.nodes_map.values_mut() {
                node.inner.reset(time_unit);
            }
        }
        TimeUnit::Samples(self.step).to_seconds(self.sr)
    }

    #[must_use]
    pub const fn sample_rate(&self) -> SampleRateBaseType {
        self.sr
    }

    #[must_use]
    pub const fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    #[must_use]
    pub const fn n_channels(&self) -> u16 {
        self.n_channels
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        self.state != ProcessorState::Stopped
    }

    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.state == ProcessorState::Playing
    }

    #[cfg(feature = "debug")]
    pub fn debug_render_graph(&self, path: String) {
        const NODE_DISTANCE: f64 = 3f64;
        const NODE_SIZE: f64 = 32f64;
        const PORT_DISTANCE: f64 = 1f64;
        const PORT_SIZE: f64 = 16f64;
        use charming::{
            Chart, HtmlRenderer,
            component::Legend,
            element::{Label, LabelPosition, LineStyle, ScaleLimit, Tooltip},
            series::{Graph, GraphCategory, GraphData, GraphLink, GraphNode},
        };
        let mut nodes = Vec::<GraphNode>::new();
        let mut links = Vec::<GraphLink>::new();
        let categories = vec![
            GraphCategory {
                name: "Nodes".into(),
            },
            GraphCategory {
                name: "Input Events".into(),
            },
            GraphCategory {
                name: "Input Signal".into(),
            },
            GraphCategory {
                name: "Output Events".into(),
            },
            GraphCategory {
                name: "Output Signal".into(),
            },
            GraphCategory {
                name: "Proxy".into(),
            },
        ];
        let mut flattened = Vec::<NodeId>::with_capacity(self.nodes_map.capacity());
        flattened.extend_from_slice(&self.nodes);
        for id in self.nodes_map.keys() {
            if !flattened.contains(id) {
                flattened.push(*id);
            }
        }
        // (orbital, index, n_per_orbital)
        let mut state = (0usize, 0usize, 1usize);
        // Distance over perimeter for 5 nodes at `NODE_DISTANCE` radius
        const DISTANCE: f64 = NODE_DISTANCE * std::f64::consts::TAU / 5f64;
        for id in &flattened {
            let node = self.nodes_map.get(id).unwrap();
            let r = NODE_DISTANCE * state.0 as f64;
            let t = state.1 as f64 / state.2 as f64;
            let x = r * (t * std::f64::consts::TAU).cos();
            let y = r * (t * std::f64::consts::TAU).sin();
            state.1 += 1;
            if state.1 >= state.2 {
                state.0 += 1;
                let p = NODE_DISTANCE * state.0 as f64 * std::f64::consts::TAU;
                state.1 = 0;
                state.2 = (p / DISTANCE).round() as usize;
            }
            let graph_node = GraphNode {
                id: format!("N{}", node.id().val()),
                name: format!("N{} {}", node.id().val(), node.inner.name()),
                x,
                y,
                value: node.id().val() as f64,
                category: 0,
                symbol_size: NODE_SIZE,
                label: None,
            };
            nodes.push(graph_node);
            if let Some(parent_node_id) = node.parent_id() {
                let link = GraphLink {
                    source: format!("N{}", parent_node_id.val()),
                    target: format!("N{}", node.id().val()),
                    value: None,
                };
                links.push(link);
            }
            let n_ports = node.ports().len();
            for (j, port) in node.ports().iter().enumerate() {
                let category = match port.kind {
                    PortType::EventsIn => 1,
                    PortType::SignalIn => 2,
                    PortType::EventsOut => 3,
                    PortType::SignalOut(_) => 4,
                    PortType::Proxy(_) => 5,
                };
                let t = j as f64 / n_ports as f64;
                let x = x + (PORT_DISTANCE * (t * std::f64::consts::TAU).cos());
                let y = y + (PORT_DISTANCE * (t * std::f64::consts::TAU).sin());
                let postfix = match port.kind {
                    PortType::SignalOut(channel_mask) => format!("[{:?}]", channel_mask),
                    _ => "".to_string(),
                };
                let graph_node = GraphNode {
                    id: format!("N{}P{}", node.id().val(), port.id.val()),
                    name: format!(
                        "N{}P{} {}{}",
                        node.id().val(),
                        port.id.val(),
                        port.name,
                        postfix
                    ),
                    x,
                    y,
                    value: node.id().val() as f64,
                    category,
                    symbol_size: PORT_SIZE,
                    label: None,
                };
                nodes.push(graph_node);
                let link = if port.kind.is_input() {
                    GraphLink {
                        source: format!("N{}P{}", port.node_id.val(), port.id.val()),
                        target: format!("N{}", node.id().val()),
                        value: None,
                    }
                } else {
                    GraphLink {
                        source: format!("N{}", node.id().val()),
                        target: format!("N{}P{}", port.node_id.val(), port.id.val()),
                        value: None,
                    }
                };
                links.push(link);
            }
        }
        for connection in self.connections_map.values() {
            let link = GraphLink {
                source: format!(
                    "N{}P{}",
                    connection.source.node_id.val(),
                    connection.source.id.val()
                ),
                target: format!(
                    "N{}P{}",
                    connection.target.node_id.val(),
                    connection.target.id.val()
                ),
                value: None,
            };
            links.push(link);
        }
        for connection in self.connections_map.values() {
            let mut source = connection.source;
            let mut last_source = source;
            while let PortType::Proxy(port_proxy) = source.kind {
                let link = GraphLink {
                    source: format!("N{}P{}", port_proxy.node_id.val(), port_proxy.port_id.val()),
                    target: format!("N{}P{}", source.node_id.val(), source.id.val()),
                    value: None,
                };
                links.push(link);
                last_source = source;
                source = self
                    .nodes_map
                    .get(&port_proxy.node_id)
                    .unwrap()
                    .get_port(port_proxy.port_id)
                    .unwrap();
            }
            if source != connection.source {
                let link = GraphLink {
                    source: format!("N{}P{}", source.node_id.val(), source.id.val()),
                    target: format!("N{}P{}", last_source.node_id.val(), last_source.id.val()),
                    value: None,
                };
                links.push(link);
            }
        }
        let data = GraphData {
            nodes,
            links,
            categories,
        };
        let chart = Chart::new()
            .tooltip(Tooltip::new())
            .legend(Legend::new().data(data.categories.iter().map(|c| c.name.clone()).collect()))
            .series(
                Graph::new()
                    .name("Dhwani")
                    .roam(true)
                    .label(
                        Label::new()
                            .show(true)
                            .position(LabelPosition::Bottom)
                            .formatter("{b}"),
                    )
                    .edge_symbol(Some(("none".to_string(), "arrow".to_string())))
                    .scale_limit(ScaleLimit::new().min(0.01).max(16))
                    .line_style(LineStyle::new().width(1.0).color("source").curveness(0.25))
                    .data(data),
            );
        let mut renderer = HtmlRenderer::new("Dhwani - Graph", 1920, 1080);
        renderer.save(&chart, path).unwrap();
    }
}
