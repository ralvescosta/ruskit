use futures_util::StreamExt;
use paho_mqtt as mqtt;
use std::time::Duration;
pub struct MqttImplV2 {}

impl MqttImplV2 {
    pub async fn test() -> Result<(), ()> {
        let opts = mqtt::CreateOptionsBuilder::new()
            .server_uri("blau:1883")
            .client_id("ai pai para")
            .finalize();

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(30))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(false)
            .user_name("blau")
            .password("blau")
            .finalize();

        let mut client = mqtt::AsyncClient::new(opts).map_err(|_| ())?;

        let mut stream = client.get_stream(1000);

        client.connect(conn_opts).await.map_err(|_| ())?;

        client.subscribe("blau", mqtt::QOS_2);

        while let Some(delivery) = stream.next().await {
            match delivery {
                Some(msg) => {
                    println!("{:?}", msg)
                }
                _ => {
                    println!("OMG ....")
                }
            }
        }

        Ok(())
    }
}
