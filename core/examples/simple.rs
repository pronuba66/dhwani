use dhwani::{
    Processor,
    channel::ChannelPositionsMask,
    nodes::{self},
    time::{TimeRange, TimeUnit},
};

const SAMPLE_RATE: u32 = 44100;
const N_CHANNELS: u16 = 2;
const CHANNEL_MASK: ChannelPositionsMask = ChannelPositionsMask::from_bits(
    ChannelPositionsMask::FRONT_LEFT.bits() | ChannelPositionsMask::FRONT_RIGHT.bits(),
)
.unwrap();
const BUFFER_SIZE: usize = 4096;

fn main() -> Result<(), anyhow::Error> {
    let mut processor = Processor::new(SAMPLE_RATE, N_CHANNELS, BUFFER_SIZE);
    // Create track 0
    let track_0_id = processor
        .add_track(TimeRange::new(TimeUnit::Seconds(0f64), None))
        .unwrap();
    // First sine
    let sine_0_node_id = processor
        .add_node(
            track_0_id,
            &nodes::OscProps::new_sin(CHANNEL_MASK, 220f32, 1f32),
        )
        .unwrap();
    // Create track 1
    let track_1_id = processor
        .add_track(TimeRange::new(TimeUnit::Seconds(0f64), None))
        .unwrap();
    // Second sine
    let sine_1_node_id = {
        let node_id = processor
            .add_node(
                track_1_id,
                &nodes::OscProps::new_sin(CHANNEL_MASK, 330f32, 1f32),
            )
            .unwrap();
        // Connect freq to output of sine 0
        processor
            .connect_ports(
                (sine_0_node_id, nodes::OscProps::PORT_ID_OUTPUT),
                (node_id, nodes::OscProps::PORT_ID_MUL_CTRL),
            )
            .unwrap();
        node_id
    };
    // Root track
    let root_track_id = processor
        .add_track(TimeRange::new(TimeUnit::Seconds(0f64), None))
        .unwrap();
    let mixer_node_id = {
        let builder = nodes::SimpleMixerProps::new(CHANNEL_MASK, vec![1f32; 2]);
        let node_id = processor.add_node(root_track_id, &builder).unwrap();
        processor
            .connect_ports(
                (sine_0_node_id, nodes::OscProps::PORT_ID_OUTPUT),
                (node_id, nodes::SimpleMixerProps::get_input_port_id(0)),
            )
            .unwrap();
        processor
            .connect_ports(
                (sine_1_node_id, nodes::OscProps::PORT_ID_OUTPUT),
                (node_id, nodes::SimpleMixerProps::get_input_port_id(1)),
            )
            .unwrap();
        node_id
    };
    processor
        .set_output_port(Some((
            mixer_node_id,
            nodes::SimpleMixerProps::PORT_ID_OUTPUT,
        )))
        .unwrap();
    let mut buffer = vec![0f32; usize::from(N_CHANNELS) * BUFFER_SIZE];
    processor.set_playing(true);
    processor
        .process(buffer.as_mut_slice())
        .map_err(|e| anyhow::Error::msg(format!("{e:?}")))?;
    #[cfg(feature = "debug")]
    processor.debug_render_graph("graph.html".into());
    dbg!(&buffer[0..10]);
    Ok(())
}
