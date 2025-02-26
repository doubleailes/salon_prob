use chrono::Utc;
use futures::prelude::*;
use influxdb2::Client;
use influxdb2_derive::WriteDataPoint;
use serde::{Deserialize, Serialize};
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

#[derive(WriteDataPoint, Debug)]
#[measurement = "cpu_load_short"]
struct Temperature {
    #[influxdb(tag)]
    location: Option<String>,
    #[influxdb(field)]
    value: f64,
    #[influxdb(timestamp)]
    time: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: ConfyConfig = confy::load("salon_prob", None)?;
    let file = confy::get_configuration_file_path("salon_prob", None)?;
    println!("The configuration file path is: {:#?}", file);
    let client = Client::new(cfg.influx_host, cfg.influx_org, cfg.influx_token);
    let t = read_from_file(&cfg.bus_file).unwrap();
    let t: Temperature = Temperature {
        location: Some("Salon".to_string()),
        value: convert_to_metric(t).into(),
        time: Utc::now().timestamp_nanos_opt().expect("Can get timestamp"),
    };
    let points: Vec<Temperature> = vec![t];
    client
        .write(&cfg.influx_bucket, stream::iter(points))
        .await?;

    Ok(())
}
