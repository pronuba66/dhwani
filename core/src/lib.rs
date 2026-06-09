//! # Dhwani — Node-Based Audio DAW Engine
//!
//!
//! **Dhwani** is an audio engine for Digital Audio Workstations (DAWs).
//! It focuses on safety, performance, and modular, extensible DSP building
//! blocks for audio applications. The engine is designed to power
//! synthesizers, mixers, effects chains, and complete DAW backends while
//! leveraging Rust's safety and concurrency guarantees.
//!
//! ## Core architecture
//!
//! Processing is built around four core concepts: [`Processor`],
//! [`track::Track`], [`node::Node`], and connections.
//!
//! A project may contain multiple tracks, and each track may contain multiple
//! nodes. A node inherits its active [`time::TimeRange`] from its parent
//! track.
//!
//! Every node exposes one or more [`port::Port`]s, which may be either inputs
//! or outputs and carry [`signal::Signals`] or [`event::Events`]. Ports can be
//! connected to compatible ports on other nodes. Each node also owns internal
//! buffers corresponding to its output ports.
//!
//! Nodes are processed in topological order. This guarantees that when a node
//! with input connections is executed, the output buffers of all upstream
//! nodes have already been produced.
//!
//! ### Nodes
//!
//! Every processing unit is represented by a `dyn` trait object implementing
//! [`node::NodeTrait`]. Nodes are created through the builder pattern via
//! [`node::NodeBuilderTrait`], allowing declarative configuration before being
//! inserted into the processing graph.
//!
//! Example nodes include oscillators, filters, piano rolls, mixers, and
//! effects.
//!
//! ### Ports & Connections
//!
//! Nodes communicate through [`port::Port`]s. Three kinds of ports are
//! available:
//!
//! - **[`signal::Signals`]** — continuous audio-rate or control-rate
//!   floating-point streams (e.g. audio buffers or LFO outputs) for each
//!   channel.
//! - **[`event::Events`]** — discrete, timestamped events within a processing
//!   frame (e.g. MIDI note on/off messages).
//! - **[`port::PortProxy`]** — proxy ports that forward another port,
//!   primarily used by parent nodes to expose child node interfaces.
//!
//! An output port may be connected to one or more compatible input ports.
//!
//! ### Processor
//!
//! The [`Processor`] owns the node graph and is driven by the audio thread.
//! On each callback it advances the graph by one frame, walking nodes in
//! topological order and propagating data through connected ports.
//!
//! The order of nodes are updated when a node is inserted or replaced. Replacing
//! may remove all previous connections to and from the nodes. If no connections
//! require removal or change, then the previous connections persists.
//!
//! ### Controller
//!
//! The optional `controller` feature exposes a [`controller::start_controller`] handle
//! that can be sent to a UI thread or any non-audio thread. It communicates with the
//! [`Processor`] via a lock-free SPSC ring buffer (`rb`) consumer and a MPSC queue.
//! allowing safe parameter automation, node graph mutations, and transport control
//! without blocking the audio thread.
//!
//! ## Feature Flags
//!
//! | Flag | Description |
//! |------|-------------|
//! | `controller` | Enables the [`controller::start_controller`] type for cross-thread communication |
//!
//! ## Example (minimal skeleton)
//!
//! see `examples/gui.rs` and `examples/simple.rs`
//! Can run the example using `./dhwani.sh gui` or `./dhwani.sh simple`

// Private
mod connection;
mod errors;
mod frame;
mod midi_consts;
mod processor;

// Public

pub mod buffer;
pub mod channel;
pub mod event;
pub mod midi_note;
pub mod node;
pub mod port;
pub mod signal;
pub mod time;
pub mod track;

/// Built-in node implementations (oscillators, filters, mixers, etc.).
/// Each sub-module exposes a builder that implements [`node::NodeBuilderTrait`].
pub mod nodes;

// ─── Re-exports ───────────────────────────────────────────────────────────────

/// Dhwani error type
pub use errors::Error;

/// Owns the node graph and drives per-frame processing on the audio thread.
/// Constructed once; thereafter only mutated via [`controller::start_controller`] messages.
pub use processor::Processor;

// ─── Optional: Controller (cross-thread communication) ────────────────────────

/// Cross-thread controller bridge.
///
/// The [`controller::start_controller`] is the only safe way to mutate the [`Processor`] from
/// outside the audio thread. Internally it uses a lock-free ring buffer (`rb`)
/// to queue commands (parameter changes, node graph edits, transport control)
/// that the audio thread drains at the start of each frame.
///
/// # Usage
///
/// ```text
/// #[cfg(feature = "controller")]
/// let (controller: Arc<controller>, rb: RingBufReceiver) = Controller::new(config);
/// ```
#[cfg(feature = "controller")]
pub mod controller;
