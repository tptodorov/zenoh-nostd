//! Declares an `rmw_zenoh` node `/rovi` with a `/cmd_vel` TwistStamped subscription, using
//! liveliness tokens, so that `ros2 node list` and `ros2 topic info -v /cmd_vel` show it.
use zenoh_examples::*;
use zenoh_nostd::session::*;

const DOMAIN: u32 = 0;
const CMD_VEL_KE: &str =
    "0/cmd_vel/geometry_msgs::msg::dds_::TwistStamped_/RIHS01_5f0fcd4f81d5d06ad9b4c4c63e3ea51b82d6ae4d0558f1d475229b1121db6f64";
const CMD_VEL_TOPIC: &str =
    "%cmd_vel/geometry_msgs::msg::dds_::TwistStamped_/RIHS01_5f0fcd4f81d5d06ad9b4c4c63e3ea51b82d6ae4d0558f1d475229b1121db6f64/2::,1:,:,:,,";

async fn entry(spawner: embassy_executor::Spawner) -> zenoh::ZResult<()> {
    env_logger::init();

    use static_cell::StaticCell;
    static CHANNEL: StaticCell<
        embassy_sync::channel::Channel<
            embassy_sync::blocking_mutex::raw::NoopRawMutex,
            FixedCapacitySample<128, 128>,
            8,
        >,
    > = StaticCell::new();
    let channel: &'static _ = CHANNEL.init(embassy_sync::channel::Channel::new());

    let config = init_session_example(&spawner).await;
    let zid = format!("{:?}", config.transports().zid());
    let session = zenoh::connect!(ExampleConfig: config, Endpoint::try_from(ENDPOINT)?);

    let node = format!("@ros2_lv/{DOMAIN}/{zid}/0/0/NN/%/%/rovi");
    let sub = format!("@ros2_lv/{DOMAIN}/{zid}/0/1/MS/%/%/rovi/{CMD_VEL_TOPIC}");
    let _node_token = session.declare_token(zenoh::keyexpr::new(&node)?).await?;
    let _sub_token = session.declare_token(zenoh::keyexpr::new(&sub)?).await?;

    let subscriber = session
        .declare_subscriber(zenoh::keyexpr::new(CMD_VEL_KE)?)
        .channel(channel.dyn_sender(), channel.dyn_receiver())
        .finish()
        .await?;

    zenoh::info!("zid {zid}: node and subscription declared");

    embassy_futures::select::select(session.run(), async {
        while let Some(sample) = subscriber.recv().await {
            zenoh::info!("[cmd_vel] {} bytes", sample.payload().len());
        }
        Ok::<(), Error>(())
    })
    .await;

    Ok(())
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    if let Err(e) = entry(spawner).await {
        zenoh::error!("Error in main: {}", e);
    }
}
