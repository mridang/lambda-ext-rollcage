
use std::collections::HashMap;

pub struct KinesisRecord {

}

pub struct Person {
    name: String,
    age: u32,
    record_buffer: HashMap<String, Vec<u8>>,
}

impl Person {
    // Public constructor
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age, record_buffer: HashMap::new() }
    }

    // Public method to access the private name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    // Public method to mutate private age
    pub fn have_birthday(&mut self) {
        self.age += 1;
    }

    // Public method to access the private age
    pub fn get_age(&self) -> u32 {
        self.age
    }

    pub fn add_record(&mut self, key: String, data: Vec<u8>) {
        match self.record_buffer.get_mut(&key) {
            Some(vec) => {
                vec.push(1);
            }
            None => {
                self.record_buffer.insert(key, data);
            }
        };
    }
}
