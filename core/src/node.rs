//! [`NodeTrait`] and [`NodeBuilderTrait`] definitions, plus the type-erased
//! [`Node`] wrapper. Implement these traits to create custom processing nodes.

use std::{collections::HashMap, fmt::Debug, ops::Range};

use crate::{
    Error, Processor,
    connection::Connection,
    event::{Events, EventsMut},
    frame::Frame,
    port::{Port, PortId, PortProps, PortType},
    signal::{Signals, SignalsMut},
    time::{SampleBaseType, SampleRateBaseType, TimeUnit},
    track::TrackId,
};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct NodeId(pub usize);

impl From<usize> for NodeId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "N[{}]", self.0)
    }
}

/// Context for node builders mostly as a wrapper to [`Processor`]
pub struct NodeCtx<'a> {
    processor: &'a mut Processor,
    id: NodeId,
    track_id: TrackId,
}

impl<'a> NodeCtx<'a> {
    pub(crate) const fn new(processor: &'a mut Processor, id: NodeId, track_id: TrackId) -> Self {
        Self {
            processor,
            id,
            track_id,
        }
    }

    #[must_use]
    pub const fn id(&self) -> NodeId {
        self.id
    }

    #[must_use]
    pub const fn sample_rate(&self) -> u32 {
        self.processor.sample_rate()
    }

    #[must_use]
    pub const fn buffer_size(&self) -> usize {
        self.processor.buffer_size()
    }

    /// Add node as a child node
    ///
    /// # Errors
    /// [`Processor::add_node_with_parent`] returns error if track id is not null
    /// Since track is never null, this should not throw any error
    /// But kept as [`Result`] for future
    pub fn add_node(&mut self, builder: &dyn NodeBuilderTrait) -> Result<NodeId, Error> {
        self.processor
            .add_node_with_parent(self.track_id, Some(self.id), builder)
    }

    /// Connect node
    ///
    /// # Errors
    /// Returns error if soure or target are not child nodes
    pub fn connect_nodes(&mut self, source_id: NodeId, target_id: NodeId) -> Result<(), Error> {
        // Source and Target nodes need to be child of current node
        if !self.processor.is_child(self.id, source_id) {
            return Err(Error::msg("Invalid source node".into()));
        }
        if !self.processor.is_child(self.id, target_id) {
            return Err(Error::msg("Invalid target node".into()));
        }
        self.processor.connect_nodes(source_id, target_id)
    }

    /// Connect ports
    ///
    /// # Errors
    /// Returns error if soure or target are not child nodes
    pub fn connect_ports(
        &mut self,
        (source_node_id, source_port_id): (NodeId, PortId),
        (target_node_id, target_port_id): (NodeId, PortId),
    ) -> Result<(), Error> {
        // Source and Target nodes need to be child of current node
        if !self.processor.is_child(self.id, source_node_id) {
            return Err(Error::msg("Invalid source node".into()));
        }
        if !self.processor.is_child(self.id, target_node_id) {
            return Err(Error::msg("Invalid target node".into()));
        }
        // Additional check if port proxy is self or child nnode
        // No need to check if port is valid right now, that will be checked
        // by [`Processor::connect_ports`]
        if let Some(source) = self
            .processor
            .get_node(source_node_id)
            .and_then(|node| node.get_port(source_port_id))
            && let PortType::Proxy(_) = source.kind
        {
            let source = self.processor.reverse_port(source)?;
            if !self.processor.is_child(self.id, source.node_id) {
                return Err(Error::msg("Invalid source node".into()));
            }
        }
        self.processor.connect_ports(
            (source_node_id, source_port_id),
            (target_node_id, target_port_id),
        )
    }

    #[must_use]
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        // Dissallow node that is not the child
        if !self.processor.is_child(self.id, id) {
            return None;
        }
        self.processor.get_node(id)
    }

    pub fn reverse_port(&self, port: Port) -> Result<Port, Error> {
        self.processor.reverse_port(port)
    }
}

pub struct NodeResetCtx {
    pub sample_rate: SampleRateBaseType,
    pub step: SampleBaseType,
}

/// Mutable output frames provided to [`NodeTrait::process`]
pub struct NodeOutputs<'a> {
    sr: SampleRateBaseType,
    range: Range<usize>,
    frames: &'a mut HashMap<PortId, Frame>,
}

impl<'a> NodeOutputs<'a> {
    #[must_use]
    pub(crate) const fn new(
        sr: SampleRateBaseType,
        range: Range<usize>,
        frames: &'a mut HashMap<PortId, Frame>,
    ) -> Self {
        Self { sr, range, frames }
    }

    pub fn clear(&mut self) {
        for frame in self.frames.values_mut() {
            // Do not clear Events as current state depends on previous states
            if let Frame::Signals(frame) = frame {
                frame.reset();
            }
        }
    }

    /// Attempts to get the mutable events for `port_id`
    ///
    /// Returns None if port not available or port is not a [`Events`]
    ///
    pub fn get_events_mut(&mut self, port_id: PortId) -> Option<EventsMut<'_>> {
        self.frames
            .get_mut(&port_id)
            .and_then(|frame| frame.get_events_mut(self.sr))
    }

    /// Attempts to get the mutable buffer for `port_id`
    ///
    /// Returns None if port not available or port is not a [`Signals`]
    ///
    pub fn get_signals_mut(&mut self, port_id: PortId) -> Option<SignalsMut<'_>> {
        self.frames
            .get_mut(&port_id)
            .and_then(|frame| frame.get_signals_mut(self.range.clone()))
    }
}

/// Input frames provided to [`NodeTrait::process`]
pub struct NodeInputs<'a> {
    sr: SampleRateBaseType,
    range: Range<usize>,
    frames_maps: &'a HashMap<NodeId, HashMap<PortId, Frame>>,
    id: NodeId,
    connections_map: &'a HashMap<(NodeId, PortId), Connection>,
}

impl<'a> NodeInputs<'a> {
    #[must_use]
    pub(crate) const fn new(
        sr: SampleRateBaseType,
        range: Range<usize>,
        frames_maps: &'a HashMap<NodeId, HashMap<PortId, Frame>>,
        id: NodeId,
        connections_map: &'a HashMap<(NodeId, PortId), Connection>,
    ) -> Self {
        Self {
            sr,
            range,
            frames_maps,
            id,
            connections_map,
        }
    }

    /// Attempts to get the buffer for `port_id`
    ///
    /// Returns None if port not available or port is not a [`Events`]
    ///
    #[must_use]
    pub fn get_events(&self, port_id: PortId) -> Option<Events<'_>> {
        self.connections_map
            .get(&(self.id, port_id))
            .and_then(|connection| {
                self.frames_maps
                    .get(&connection.source.node_id)
                    .and_then(|frames| frames.get(&connection.source.id))
            })
            .and_then(|frame| frame.get_events(self.sr))
    }

    /// Attempts to get the buffer for `port_id`
    ///
    /// Returns None if port not available or port is not a [`Signals`]
    ///
    #[must_use]
    pub fn get_signals(&self, port_id: PortId) -> Option<Signals<'_>> {
        self.connections_map
            .get(&(self.id, port_id))
            .and_then(|connection| {
                self.frames_maps
                    .get(&connection.source.node_id)
                    .and_then(|frames| frames.get(&connection.source.id))
            })
            .and_then(|frame| frame.get_signals(self.range.clone()))
    }

    /// Attempts to get the mono buffer
    #[must_use]
    pub fn get_mono(&self, port_id: PortId) -> Option<&[f32]> {
        self.connections_map
            .get(&(self.id, port_id))
            .and_then(|connection| {
                self.frames_maps
                    .get(&connection.source.node_id)
                    .and_then(|frames| frames.get(&connection.source.id))
            })
            .and_then(|frame| frame.get_mono(self.range.clone()))
    }
}

/// Trait for building a object of [`NodeTrait`]
pub trait NodeBuilderTrait: Sync + Send {
    fn build(&self, ctx: &mut NodeCtx) -> Box<dyn NodeTrait>;
}

/// Nodes are defined by [`NodeTrait`]
pub trait NodeTrait {
    fn process(&mut self, step_range: Range<usize>, inputs: &NodeInputs, outputs: &mut NodeOutputs);
    #[allow(unused_variables)]
    /// Reset happened when there is discontinuity so that node can adjust cumulative
    /// parameters
    /// Always called when node is built
    fn reset(&mut self, ctx: &NodeResetCtx) {}
    /// This is for keeping the node relevant event after the track end time
    /// Example delay node need extra
    /// None mean end is infinite
    fn duration_extension(&self) -> Option<TimeUnit> {
        Some(TimeUnit::Samples(0))
    }
    fn port_props(&self) -> &[PortProps];
    fn name(&self) -> &str;
}

/// Wrapper to [`NodeTrait`] used by [`Processor`]
pub struct Node {
    id: NodeId,
    track_id: TrackId,
    parent_id: Option<NodeId>,
    ports: Vec<Port>,
    pub(crate) inner: Box<dyn NodeTrait>,
    pub(crate) frames_invalidated: bool,
}

impl Node {
    /// Create [`Node`] from [`NodeBuilderTrait`] and add it to [`Processor`].
    ///
    /// [`Node`] is created and added to [`Processor`]. The [`Node`] can create
    /// other [`Node`]s and [`Connection`]s and thus does not make sense to have
    /// the root [`Node`] created seperately and returned to the caller.
    ///
    /// # Errors
    /// Returns [`Error`] if [`crate::track::Track`] for [`TrackId`] not found.
    ///
    pub(crate) fn add(
        processor: &mut Processor,
        track_id: TrackId,
        parent_id: Option<NodeId>,
        builder: &dyn NodeBuilderTrait,
    ) -> Result<NodeId, Error> {
        if !processor.tracks_map.contains_key(&track_id) {
            return Err(Error::msg("Track not found".into()));
        }
        let id: NodeId = processor.new_id().into();
        let mut ctx = NodeCtx::new(processor, id, track_id);
        let mut inner = builder.build(&mut ctx);
        let ctx = NodeResetCtx {
            sample_rate: processor.sample_rate(),
            step: processor.step
                - processor
                    .tracks_map
                    .get(&track_id)
                    .unwrap()
                    .time_range()
                    .start()
                    .to_samples(processor.sample_rate()),
        };
        inner.reset(&ctx);
        let mut ports = Vec::<Port>::with_capacity(inner.port_props().len());
        for port_props in inner.port_props() {
            ports.push(Port::new(
                id,
                port_props.id,
                port_props.kind,
                port_props.auto_connect,
                port_props.name,
            ));
        }
        let node = Self {
            id,
            track_id,
            parent_id,
            ports,
            inner,
            frames_invalidated: false,
        };
        processor
            .frames_maps
            .insert(id, processor.build_frames(&node));
        processor.nodes_map.insert(id, node);
        Ok(id)
    }

    /// Replace [`Node`] by rebuilding [`NodeBuilderTrait`] keeping, [`NodeId`]
    /// [`TrackId`], and parent [`NodeId`] intact. If incoming and outgoing
    /// connections does not match the new ports, all relevent connections are
    /// removed.
    ///
    /// See comments for [`Node::add`] to know the reason why [`Node`] is directly
    /// added to [`Processor`] and not returned to the caller
    ///
    /// # Errors
    /// Returns [`Error`] if [`crate::track::Track`] for [`TrackId`] not found.
    ///
    pub(crate) fn replace(
        processor: &mut Processor,
        id: NodeId,
        builder: &dyn NodeBuilderTrait,
    ) -> Result<bool, Error> {
        let old_node = processor
            .nodes_map
            .get(&id)
            .ok_or_else(|| Error::msg("Node not found".into()))?;
        let track_id = old_node.track_id;
        let parent_id = old_node.parent_id;
        let mut ctx = NodeCtx::new(processor, id, track_id);
        let mut inner = builder.build(&mut ctx);
        let ctx = NodeResetCtx {
            sample_rate: processor.sample_rate(),
            step: processor.step
                - processor
                    .tracks_map
                    .get(&track_id)
                    .unwrap()
                    .time_range()
                    .start()
                    .to_samples(processor.sample_rate()),
        };
        inner.reset(&ctx);
        let mut ports = Vec::<Port>::with_capacity(inner.port_props().len());
        for port_props in inner.port_props() {
            ports.push(Port::new(
                id,
                port_props.id,
                port_props.kind,
                port_props.auto_connect,
                port_props.name,
            ));
        }
        let node = Self {
            id,
            track_id,
            parent_id,
            ports,
            inner,
            frames_invalidated: false,
        };
        // Try to persist connection on replace
        let mut invalidate_connections = false;
        for connection in processor.connections_map.values() {
            // Outgoing connections
            if connection.source.node_id == id
                && Some(connection.source) != node.get_port(connection.source.id)
            {
                invalidate_connections = true;
                break;
            }
            // Incoming connections
            if connection.target.node_id == id
                && Some(connection.target) != node.get_port(connection.target.id)
            {
                invalidate_connections = true;
                break;
            }
        }
        // Remove node before adding new
        processor.remove_nodes(id, invalidate_connections);
        processor
            .frames_maps
            .insert(id, processor.build_frames(&node));
        processor.nodes_map.insert(id, node);
        Ok(invalidate_connections)
    }

    /// Returns the [`NodeId`]
    #[must_use]
    pub const fn id(&self) -> NodeId {
        self.id
    }

    /// Returns the [`TrackId`]
    #[must_use]
    pub const fn track_id(&self) -> TrackId {
        self.track_id
    }

    /// Returns the parent [`NodeId`] if available
    #[must_use]
    pub const fn parent_id(&self) -> Option<NodeId> {
        self.parent_id
    }

    /// Returns available [`Port`]s
    #[must_use]
    pub fn ports(&self) -> &[Port] {
        &self.ports
    }

    ///
    /// # Errors
    /// Return `Err` if port not found
    ///
    #[must_use]
    pub fn get_port(&self, id: PortId) -> Option<Port> {
        for port in &self.ports {
            if port.id == id {
                return Some(*port);
            }
        }
        None
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{name: \"{}\", id: {:?}}}", self.inner.name(), self.id)
    }
}
