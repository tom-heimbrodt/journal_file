use std::fs::File;
use std::path::Path;
use io::*;

fn main() {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(Path::new("test"))
        .unwrap();

    write(&mut file);
    read(&mut file);
}

pub fn write(file: &mut File) {
    let mut writer: SimpleJournalWriter<&str> = SimpleJournalWriter::new(file);
    writer.store_entry("Hello World!").unwrap();

    let other_entries = vec!["Another Entry", "Even more data"];
    writer.store_entries(other_entries.into_iter()).unwrap();
}

pub fn read(file: &mut File) {
    let mut reader: SimpleJournalReader<String> = SimpleJournalReader::new(file);

    for value in &mut reader {
        let unwrapped = value
            .unwrap(); // We will iterate over Results since deserialization can fail
        println!("Entry: {}", unwrapped);
    }

    let values = reader
        .iter()
        .collect::<Result<Vec<String>, _>>()
        .unwrap();

    assert_eq!(values, vec!["Hello World!", "Another Entry", "Even more data"]);
}