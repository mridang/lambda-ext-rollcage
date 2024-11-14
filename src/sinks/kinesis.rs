use crate::sink::RecordSink;
use aws_sdk_kinesis::primitives::Blob;
use aws_sdk_kinesis::Client;

pub struct KinesisSink {
    client: Client,
}

impl KinesisSink {
    pub async fn new(client: Option<Client>) -> Self {
        let client = match client {
            Some(existing_client) => existing_client,
            None => {
                let config = aws_config::load_from_env().await;
                Client::new(&config)
            }
        };
        KinesisSink { client }
    }
}

impl RecordSink for KinesisSink {
    async fn sink(&mut self, stream_arn: String, encoded_data: Vec<u8>) {
        let output = self
            .client
            .put_record()
            .stream_arn(stream_arn)
            .partition_key("moomoo")
            .data(Blob::new(encoded_data))
            .send()
            .await;
    }
}
