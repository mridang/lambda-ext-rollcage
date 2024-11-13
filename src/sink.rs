pub trait RecordSink {
    fn sink(&mut self, data: Vec<u8>);
}

pub struct ConsoleSink;

impl RecordSink for ConsoleSink {
    fn sink(&mut self, data: Vec<u8>) {
        println!("Dumped AggregatedRecord: {:?}", data);
    }
}

pub struct MockSink {
    pub captured_output: Vec<u8>,
}

impl MockSink {
    pub(crate) fn new() -> Self {
        MockSink {
            captured_output: Vec::new(),
        }
    }
}

impl RecordSink for MockSink {
    fn sink(&mut self, bytes: Vec<u8>) {
        self.captured_output.extend(bytes);
    }
}
