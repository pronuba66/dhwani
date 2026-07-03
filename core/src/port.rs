//! [`Port`] type definitions and signal/event port variants.
//! Ports are the typed sockets through which nodes exchange data.

use std::fmt::Debug;

use crate::{channel::ChannelPositionsMask, node::NodeId};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct PortId(pub usize);

impl From<usize> for PortId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Debug for PortId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P[{}]", self.0)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct PortProxyId(pub(crate) usize);

impl From<usize> for PortProxyId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Debug for PortProxyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P[{}]", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PortProxy {
    // [`PortProxy::id`] along with [`PortProxy::is_event`] and
    // [`PortProxy::is_input`] is used for checking if port proxt is same
    // The node id and port id may change
    pub(crate) id: PortProxyId,
    pub(crate) node_id: NodeId,
    pub(crate) port_id: PortId,
    pub(crate) is_event: bool,
    pub(crate) is_input: bool,
}

impl PortProxy {
    #[must_use]
    pub const fn new(id: PortProxyId, port: Port) -> Self {
        Self {
            id,
            node_id: port.node_id,
            port_id: port.id,
            is_event: port.kind.is_voice(),
            is_input: port.kind.is_input(),
        }
    }
}

impl PartialEq for PortProxy {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.is_event == other.is_event && self.is_input == other.is_input
    }
}

impl Eq for PortProxy {}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PortType {
    VoicesIn,
    SignalIn,
    VoicesOut,
    SignalOut(ChannelPositionsMask),
    /// Port proxy is identified by [`PortProxyId`] so that when replacing node, the
    /// connection can persist if the [`PortProxyId`] is same.
    Proxy(PortProxy),
}

impl PortType {
    #[must_use]
    pub const fn is_input(&self) -> bool {
        match self {
            Self::VoicesIn | Self::SignalIn => true,
            Self::VoicesOut | Self::SignalOut(_) => false,
            Self::Proxy(port_proxy) => port_proxy.is_input,
        }
    }

    #[must_use]
    pub const fn is_output(&self) -> bool {
        !self.is_input()
    }
    #[must_use]
    pub const fn is_voice(&self) -> bool {
        match self {
            Self::VoicesIn | Self::VoicesOut => true,
            Self::SignalIn | Self::SignalOut(_) => false,
            Self::Proxy(port_proxy) => port_proxy.is_event,
        }
    }

    #[must_use]
    pub const fn is_signal(&self) -> bool {
        !self.is_voice()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PortProps {
    pub(crate) id: PortId,
    pub(crate) kind: PortType,
    pub(crate) auto_connect: bool,
    pub(crate) name: &'static str,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Port {
    pub(crate) node_id: NodeId,
    pub(crate) id: PortId,
    pub(crate) kind: PortType,
    pub(crate) auto_connect: bool,
    pub(crate) name: &'static str,
}

impl Port {
    pub(crate) const fn new(
        node_id: NodeId,
        id: PortId,
        kind: PortType,
        auto_connect: bool,
        name: &'static str,
    ) -> Self {
        Self {
            node_id,
            id,
            kind,
            auto_connect,
            name,
        }
    }

    #[must_use]
    pub const fn node_id(&self) -> NodeId {
        self.node_id
    }

    #[must_use]
    pub const fn id(&self) -> PortId {
        self.id
    }

    #[must_use]
    pub const fn kind(&self) -> PortType {
        self.kind
    }

    #[must_use]
    pub const fn auto_connect(&self) -> bool {
        self.auto_connect
    }

    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

impl Debug for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}::{:?}", self.name, self.node_id, self.id)?;
        write!(f, "[")?;
        match self.kind {
            PortType::VoicesIn => write!(f, "E[I]"),
            PortType::SignalIn => write!(f, "S[I]"),
            PortType::VoicesOut => write!(f, "E[O]"),
            PortType::SignalOut(_n_channels) => write!(f, "S[O]"),
            PortType::Proxy(_) => write!(f, "P[O]"),
        }?;
        write!(f, "]")
    }
}
