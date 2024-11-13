use crate::sink::RecordSink;
use prost::Message;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct PutRecord {
    pub stream_name: String,
    pub partition_key: String,
    pub explicit_hash_key: Option<String>,
    pub data: Vec<u8>,
}

#[derive(Message, PartialEq)]
struct AggregatedRecord {
    #[prost(string, repeated, tag = "1")]
    partition_key_table: Vec<String>,
    #[prost(string, repeated, tag = "2")]
    explicit_hash_key_table: Vec<String>,
    #[prost(message, repeated, tag = "3")]
    records: Vec<Record>,
}

#[derive(Message, PartialEq, Clone)]
struct Record {
    #[prost(uint64, tag = "1")]
    partition_key_index: u64,
    #[prost(uint64, optional, tag = "2")]
    explicit_hash_key_index: Option<u64>,
    #[prost(bytes, tag = "3")]
    data: Vec<u8>,
    #[prost(message, repeated, tag = "4")]
    tags: Vec<Tag>,
}

#[derive(Message, PartialEq, Clone)]
struct Tag {
    #[prost(string, tag = "1")]
    key: String,
    #[prost(string, optional, tag = "2")]
    value: Option<String>,
}

#[derive(Default)]
pub struct AggregatedData {
    pub partition_key_table: Vec<String>,
    pub explicit_hash_key_table: Vec<String>,
    pub records: Vec<Record>,
    pub partition_key_map: HashMap<String, u64>,
    pub explicit_hash_key_map: HashMap<String, u64>,
    pub current_size: u32,
}

pub struct StreamAggregator<S: RecordSink> {
    pub aggregated_data: HashMap<String, AggregatedData>,
    max_size: u32,
    pub record_sink: S,
}

impl<S: RecordSink> StreamAggregator<S> {
    fn new(max_size: u32, record_sink: S) -> Self {
        Self {
            max_size,
            record_sink,
            aggregated_data: HashMap::new(),
        }
    }

    async fn insert(&mut self, stream_name: String, put_record: PutRecord, record_size: u32) {
        {
            let aggregated_data = self
                .aggregated_data
                .entry(stream_name.clone())
                .or_insert_with(AggregatedData::default);

            if aggregated_data.current_size + record_size > self.max_size {
                self.flush(stream_name.clone()).await
            }
        }

        let aggregated_data = self
            .aggregated_data
            .entry(stream_name.clone())
            .or_insert_with(AggregatedData::default);

        let partition_key_index = *aggregated_data
            .partition_key_map
            .entry(put_record.partition_key.clone())
            .or_insert_with(|| {
                let index = aggregated_data.partition_key_table.len() as u64;
                aggregated_data
                    .partition_key_table
                    .push(put_record.partition_key.clone());
                index
            });

        let explicit_hash_key_index = put_record.explicit_hash_key.as_ref().map(|key| {
            *aggregated_data
                .explicit_hash_key_map
                .entry(key.clone())
                .or_insert_with(|| {
                    let index = aggregated_data.explicit_hash_key_table.len() as u64;
                    aggregated_data.explicit_hash_key_table.push(key.clone());
                    index
                })
        });

        aggregated_data.current_size += record_size;
        aggregated_data.records.push(Record {
            partition_key_index,
            explicit_hash_key_index,
            data: put_record.data,
            tags: vec![],
        });

        if aggregated_data.current_size == self.max_size {
            self.flush(stream_name.clone()).await
        }
    }

    async fn flush(&mut self, stream_name: String) {
        if let Some(aggregated_data) = self.aggregated_data.remove(&stream_name) {
            let aggregated_record = AggregatedRecord {
                partition_key_table: aggregated_data.partition_key_table.clone(),
                explicit_hash_key_table: aggregated_data.explicit_hash_key_table.clone(),
                records: aggregated_data.records.clone(),
            };

            let mut buf = Vec::new();
            aggregated_record
                .encode(&mut buf)
                .expect("Failed to encode record");
            println!("Dumped AggregatedRecord: {:?}", buf);

            self.record_sink.sink(buf).await;
        }
    }

    async fn flush_all(&mut self) {
        let keys: Vec<String> = self.aggregated_data.keys().cloned().collect();
        for key in keys {
            self.flush(key).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockSink;

    #[tokio::test]
    async fn test_insert_and_flush() {
        let mock_sink = MockSink::new();
        let mut aggregator = StreamAggregator::new(100, mock_sink);

        aggregator
            .insert(
                "stream1".to_string(),
                make_sample_record("key1", "hash1"),
                50,
            )
            .await;
        aggregator
            .insert(
                "stream1".to_string(),
                make_sample_record("key2", "hash2"),
                60,
            )
            .await; // Triggers flush

        println!("{}", aggregator.record_sink.captured_output.len());
        assert!(!aggregator.record_sink.captured_output.is_empty());

        let aggregated_record = AggregatedRecord::decode(&*aggregator.record_sink.captured_output)
            .expect("Failed to decode protobuf bytes");

        assert_eq!(
            aggregated_record,
            AggregatedRecord {
                partition_key_table: vec!["key1".to_string()],
                explicit_hash_key_table: vec!["hash1".to_string()],
                records: vec![Record {
                    partition_key_index: 0,
                    explicit_hash_key_index: Some(0),
                    data: vec![1, 2, 3],
                    tags: vec![],
                }],
            }
        );
    }

    #[tokio::test]
    async fn test_that_the_aggregator_flushes_when_threshold_matches() {
        let mock_sink = MockSink::new();
        let mut aggregator = StreamAggregator::new(100, mock_sink);

        aggregator
            .insert(
                "stream1".to_string(),
                make_sample_record("key1", "hash1"),
                50,
            )
            .await;
        aggregator
            .insert(
                "stream1".to_string(),
                make_sample_record("key2", "hash2"),
                50,
            )
            .await;

        println!("{}", aggregator.record_sink.captured_output.len());
        assert!(!aggregator.record_sink.captured_output.is_empty());

        let aggregated_record = AggregatedRecord::decode(&*aggregator.record_sink.captured_output)
            .expect("Failed to decode protobuf bytes");

        assert_eq!(
            aggregated_record,
            AggregatedRecord {
                partition_key_table: vec!["key1".to_string(), "key2".to_string()],
                explicit_hash_key_table: vec!["hash1".to_string(), "hash2".to_string()],
                records: vec![
                    Record {
                        partition_key_index: 0,
                        explicit_hash_key_index: Some(0),
                        data: vec![1, 2, 3],
                        tags: vec![],
                    },
                    Record {
                        partition_key_index: 1,
                        explicit_hash_key_index: Some(1),
                        data: vec![1, 2, 3],
                        tags: vec![],
                    }
                ],
            }
        );
    }

    #[tokio::test]
    async fn test_that_the_aggregator_correctly_flushes_all() {
        let mock_sink = MockSink::new();
        let mut aggregator = StreamAggregator::new(1000, mock_sink);

        aggregator
            .insert(
                "stream1".to_string(),
                make_sample_record("key1", "hash1"),
                50,
            )
            .await;
        aggregator
            .insert(
                "stream1".to_string(),
                make_sample_record("key2", "hash2"),
                50,
            )
            .await;

        assert!(aggregator.record_sink.captured_output.is_empty());

        aggregator.flush_all().await;
        assert!(!aggregator.record_sink.captured_output.is_empty());
        let aggregated_record = AggregatedRecord::decode(&*aggregator.record_sink.captured_output)
            .expect("Failed to decode protobuf bytes");

        assert_eq!(
            aggregated_record,
            AggregatedRecord {
                partition_key_table: vec!["key1".to_string(), "key2".to_string()],
                explicit_hash_key_table: vec!["hash1".to_string(), "hash2".to_string()],
                records: vec![
                    Record {
                        partition_key_index: 0,
                        explicit_hash_key_index: Some(0),
                        data: vec![1, 2, 3],
                        tags: vec![],
                    },
                    Record {
                        partition_key_index: 1,
                        explicit_hash_key_index: Some(1),
                        data: vec![1, 2, 3],
                        tags: vec![],
                    }
                ],
            }
        );
    }

    fn make_sample_record(partition_key: &str, explicit_hash_key: &str) -> PutRecord {
        PutRecord {
            stream_name: "test_stream".to_string(),
            partition_key: partition_key.to_string(),
            explicit_hash_key: Some(explicit_hash_key.to_string()),
            data: vec![1, 2, 3], // example data
        }
    }
}
