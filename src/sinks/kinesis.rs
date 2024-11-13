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
    async fn sink(&mut self, data: Vec<u8>) {
        let output = self
            .client
            .put_record()
            .stream_arn("arn:aws:kinesis:us-east-1:188628773952:stream/mytest")
            .partition_key("moomoo")
            .data(Blob::new(data))
            .send()
            .await;
    }
}
