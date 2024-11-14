use std::collections::HashMap;

pub trait RecordSink {
    async fn sink(&mut self, stream_arn: String, encoded_data: Vec<u8>);
}

pub struct ConsoleSink;

impl RecordSink for ConsoleSink {
    async fn sink(&mut self, stream_arn: String, encoded_data: Vec<u8>) {
        println!("Dumped records: {:?} to {}", encoded_data, stream_arn);
    }
}

pub struct MockSink {
    pub captured_output: HashMap<String, Vec<u8>>,
}

impl MockSink {
    pub(crate) fn new() -> Self {
        MockSink {
            captured_output: HashMap::new(),
        }
    }
}

impl RecordSink for MockSink {
    async fn sink(&mut self, stream_arn: String, encoded_data: Vec<u8>) {
        self.captured_output
            .entry(stream_arn)
            .or_insert_with(Vec::new)
            .extend(encoded_data);
    }
}
