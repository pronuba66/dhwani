mod osc;
mod piano_roll;
mod sampler;
mod simple_delay;
mod simple_filter;
mod simple_mixer;
mod simple_piano;
mod stereo;

pub use osc::OscMode;
pub use osc::OscProps;
pub use piano_roll::PianoRollProps;
pub use sampler::SampleInfo;
pub use sampler::SamplerProps;
pub use simple_delay::SimpleDelayProps;
pub use simple_filter::SimpleFilterProps;
pub use simple_filter::SimpleFilterType;
pub use simple_mixer::SimpleMixerProps;
pub use simple_piano::SimplePianoProps;
pub use stereo::StereoProps;

use crate::node::NodeId;
use crate::track::TrackId;

#[derive(Debug)]
pub struct NodeInfo {
    pub id: NodeId,
    pub track_id: TrackId,
}
