use bitflags::bitflags;

use crate::Error;

bitflags! {
    /// Taken from Symphonia channels
    /// ref: https://github.com/pdeljanov/Symphonia/blob/main/symphonia-core/src/audio/channels.rs#L19

    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct ChannelPositionsMask: u64 {
        /// Front-left (left) channel.
        const FRONT_LEFT          = 1 << 0;
        /// Front-right (right) channel.
        const FRONT_RIGHT         = 1 << 1;
        /// Front-center (center) or the Mono channel.
        const FRONT_CENTER        = 1 << 2;
        /// Low-frequency effects (LFE) channel 1.
        ///
        /// Apple calls this channel "lfe screen".
        const LFE1                = 1 << 3;
        /// Rear-left channel.
        ///
        /// Apple calls this channel "left surround". Dolby calls it "surround rear left".
        const REAR_LEFT           = 1 << 4;
        /// Rear-right channel.
        ///
        /// Apple calls this channel "right surround". Dolby calls it "surround rear right".
        const REAR_RIGHT          = 1 << 5;
        /// Front left-of-center channel.
        ///
        /// Apple and Dolby call this channel "left center".
        const FRONT_LEFT_CENTER   = 1 << 6;
        /// Front right-of-center channel.
        ///
        /// Apple and Dolby call this channel "right center".
        const FRONT_RIGHT_CENTER  = 1 << 7;
        /// Rear-center channel.
        ///
        /// Microsoft calls this channel "back center". Apple calls it "center surround". Dolby
        /// calls it "surround rear center".
        const REAR_CENTER         = 1 << 8;
        /// Side-left channel.
        ///
        /// Apple calls this channel "left surround direct". Dolby calls it "surround left".
        const SIDE_LEFT           = 1 << 9;
        /// Side-right channel.
        ///
        /// Apple calls this channel "right surround direct". Dolby calls it "surround right".
        const SIDE_RIGHT          = 1 << 10;
        /// Top-center channel.
        ///
        /// Apple calls this channel "top center surround".
        const TOP_CENTER          = 1 << 11;
        /// Top-front left channel.
        ///
        /// Apple calls this channel "vertical height left".
        const TOP_FRONT_LEFT      = 1 << 12;
        /// Top-center channel.
        ///
        /// Apple calls this channel "vertical height center".
        const TOP_FRONT_CENTER    = 1 << 13;
        /// Top-front right channel.
        ///
        /// Apple calls this channel "vertical height right".
        const TOP_FRONT_RIGHT     = 1 << 14;
        /// Top-rear left channel.
        ///
        /// Microsoft and Apple call this channel "top back left".
        const TOP_REAR_LEFT       = 1 << 15;
        /// Top-rear center channel.
        ///
        /// Microsoft and Apple call this channel "top back center".
        const TOP_REAR_CENTER     = 1 << 16;
        /// Top-rear right channel.
        ///
        /// Microsoft and Apple call this channel "top back right".
        const TOP_REAR_RIGHT      = 1 << 17;

        // End of standard WAVE channels.

        /// Low-frequency effects channel 2.
        const LFE2                = 1 << 18;
        /// Top-side left channel.
        const TOP_SIDE_LEFT       = 1 << 19;
        /// Top-rear right channel.
        const TOP_SIDE_RIGHT      = 1 << 20;
        /// Bottom-front center channel.
        const BOTTOM_FRONT_CENTER = 1 << 21;
        /// Bottom-front left channel.
        const BOTTOM_FRONT_LEFT   = 1 << 22;
        /// Bottom-front right channel.
        const BOTTOM_FRONT_RIGHT  = 1 << 23;
        /// Front-left wide channel.
        ///
        /// Apple calls this channel "left wide".
        const FRONT_LEFT_WIDE     = 1 << 24;
        /// Front-right wide channel.
        ///
        /// Apple calls this channel "right wide".
        const FRONT_RIGHT_WIDE    = 1 << 25;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelPosition {
    /// Front-left (left) channel.
    FrontLeft,
    /// Front-right (right) channel.
    FrontRight,
    /// Front-center (center) or the Mono channel.
    FrontCenter,
    /// Low-frequency effects (LFE) channel 1.
    ///
    /// Apple calls this channel "lfe screen".
    Lfe1,
    /// Rear-left channel.
    ///
    /// Apple calls this channel "left surround". Dolby calls it "surround rear left".
    RearLeft,
    /// Rear-right channel.
    ///
    /// Apple calls this channel "right surround". Dolby calls it "surround rear right".
    RearRight,
    /// Front left-of-center channel.
    ///
    /// Apple and Dolby call this channel "left center".
    FrontLeftCenter,
    /// Front right-of-center channel.
    ///
    /// Apple and Dolby call this channel "right center".
    FrontRightCenter,
    /// Rear-center channel.
    ///
    /// Microsoft calls this channel "back center". Apple calls it "center surround". Dolby
    /// calls it "surround rear center".
    RearCenter,
    /// Side-left channel.
    ///
    /// Apple calls this channel "left surround direct". Dolby calls it "surround left".
    SideLeft,
    /// Side-right channel.
    ///
    /// Apple calls this channel "right surround direct". Dolby calls it "surround right".
    SideRight,
    /// Top-center channel.
    ///
    /// Apple calls this channel "top center surround".
    TopCenter,
    /// Top-front left channel.
    ///
    /// Apple calls this channel "vertical height left".
    TopFrontLeft,
    /// Top-center channel.
    ///
    /// Apple calls this channel "vertical height center".
    TopFrontCenter,
    /// Top-front right channel.
    ///
    /// Apple calls this channel "vertical height right".
    TopFrontRight,
    /// Top-rear left channel.
    ///
    /// Microsoft and Apple call this channel "top back left".
    TopRearLeft,
    /// Top-rear center channel.
    ///
    /// Microsoft and Apple call this channel "top back center".
    TopRearCenter,
    /// Top-rear right channel.
    ///
    /// Microsoft and Apple call this channel "top back right".
    TopRearRight,

    // End of standard WAVE channels.
    /// Low-frequency effects channel 2.
    Lfe2,
    /// Top-side left channel.
    TopSideLeft,
    /// Top-rear right channel.
    TopSideRight,
    /// Bottom-front center channel.
    BottomFrontCenter,
    /// Bottom-front left channel.
    BottomFrontLeft,
    /// Bottom-front right channel.
    BottomFrontRight,
    /// Front-left wide channel.
    ///
    /// Apple calls this channel "left wide".
    FrontLeftWide,
    /// Front-right wide channel.
    ///
    /// Apple calls this channel "right wide".
    FrontRightWide,
}

impl TryFrom<ChannelPositionsMask> for ChannelPosition {
    type Error = Error;

    fn try_from(value: ChannelPositionsMask) -> Result<Self, Self::Error> {
        debug_assert!(
            value.bits().count_ones() == 1,
            "Channel position mask contains multiple channels"
        );
        match value {
            ChannelPositionsMask::FRONT_LEFT => Ok(ChannelPosition::FrontLeft),
            ChannelPositionsMask::FRONT_RIGHT => Ok(ChannelPosition::FrontRight),
            ChannelPositionsMask::FRONT_CENTER => Ok(ChannelPosition::FrontCenter),
            ChannelPositionsMask::LFE1 => Ok(ChannelPosition::Lfe1),
            ChannelPositionsMask::REAR_LEFT => Ok(ChannelPosition::RearLeft),
            ChannelPositionsMask::REAR_RIGHT => Ok(ChannelPosition::RearRight),
            ChannelPositionsMask::FRONT_LEFT_CENTER => Ok(ChannelPosition::FrontLeftCenter),
            ChannelPositionsMask::FRONT_RIGHT_CENTER => Ok(ChannelPosition::FrontRightCenter),
            ChannelPositionsMask::REAR_CENTER => Ok(ChannelPosition::RearCenter),
            ChannelPositionsMask::SIDE_LEFT => Ok(ChannelPosition::SideLeft),
            ChannelPositionsMask::SIDE_RIGHT => Ok(ChannelPosition::SideRight),
            ChannelPositionsMask::TOP_CENTER => Ok(ChannelPosition::TopCenter),
            ChannelPositionsMask::TOP_FRONT_LEFT => Ok(ChannelPosition::TopFrontLeft),
            ChannelPositionsMask::TOP_FRONT_CENTER => Ok(ChannelPosition::TopFrontCenter),
            ChannelPositionsMask::TOP_FRONT_RIGHT => Ok(ChannelPosition::TopFrontRight),
            ChannelPositionsMask::TOP_REAR_LEFT => Ok(ChannelPosition::TopRearLeft),
            ChannelPositionsMask::TOP_REAR_CENTER => Ok(ChannelPosition::TopRearCenter),
            ChannelPositionsMask::TOP_REAR_RIGHT => Ok(ChannelPosition::TopRearRight),
            ChannelPositionsMask::LFE2 => Ok(ChannelPosition::Lfe2),
            ChannelPositionsMask::TOP_SIDE_LEFT => Ok(ChannelPosition::TopSideLeft),
            ChannelPositionsMask::TOP_SIDE_RIGHT => Ok(ChannelPosition::TopSideRight),
            ChannelPositionsMask::BOTTOM_FRONT_CENTER => Ok(ChannelPosition::BottomFrontCenter),
            ChannelPositionsMask::BOTTOM_FRONT_LEFT => Ok(ChannelPosition::BottomFrontLeft),
            ChannelPositionsMask::BOTTOM_FRONT_RIGHT => Ok(ChannelPosition::BottomFrontRight),
            ChannelPositionsMask::FRONT_LEFT_WIDE => Ok(ChannelPosition::FrontLeftWide),
            ChannelPositionsMask::FRONT_RIGHT_WIDE => Ok(ChannelPosition::FrontRightWide),
            _ => Err(Error::msg("Invalid channel mask".into())),
        }
    }
}

impl From<ChannelPositionsMask> for Vec<ChannelPosition> {
    fn from(value: ChannelPositionsMask) -> Self {
        value
            .iter()
            .map(|ch| ChannelPosition::try_from(ch).unwrap())
            .collect()
    }
}
