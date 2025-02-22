use serde::{Deserialize, Serialize};
use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use w1_therm_reader::{convert_to_metric, read_from_file};

#[derive(Debug, Serialize, Deserialize)]
struct ConfyConfig {
    bus_file: String,
    influx_host: String,
    influx_org: String,
    influx_token: String,
    influx_bucket: String,
}

impl Default for ConfyConfig {
    fn default() -> Self {
        ConfyConfig {
            bus_file: "/sys/bus/w1/devices/prout/w1_slave".into(),
            influx_host: "http://influxdb:8086".into(),
            influx_org: "my-org".into(),
            influx_token: "my-token".into(),
            influx_bucket: "my-bucket".into(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: ConfyConfig = confy::load("salon_prob", None)?;
    let file = confy::get_configuration_file_path("salon_prob", None)?;
    println!("The configuration file path is: {:#?}", file);
    let t = read_from_file(&cfg.bus_file).unwrap();
    let t_metric: f64 = convert_to_metric(t).into();
    let client = Client::new(cfg.influx_host, cfg.influx_org, cfg.influx_token);

    let points = vec![DataPoint::builder("salon")
        .field("Temperature", t_metric)
        .build()?];

    client
        .write(&cfg.influx_bucket, stream::iter(points))
        .await?;

    Ok(())
}
