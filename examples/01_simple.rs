use std::fs::File;
use std::path::Path;
use journal_file::*;

fn main() {
    // Open a file handle used for reading and writing
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
    // Construct a writer for writing &str
    // This won't perform any IO operations
    let mut writer: SimpleJournalWriter<&str> = SimpleJournalWriter::new(file);

    // Write a single entry
    writer.store_entry("Hello World!").unwrap();

    // Write multiple entries at once
    let other_entries = vec!["Another Entry", "Even more data"];
    writer.store_entries(other_entries.into_iter()).unwrap();
}

pub fn read(file: &mut File) {
    // Construct a reader for reading Strings
    // This won't perform any IO operations
    // Note: We cannot use &str here, since the deserialized value
    //       musst be owned
    let mut reader: SimpleJournalReader<String> =
        SimpleJournalReader::new(file);

    // You can use this reader in a simple for loop
    for value in &mut reader {
        // We will iterate over Results since deserialization can fail
        let unwrapped = value.unwrap();
        println!("Entry: {}", unwrapped);
    }

    // You might also want to collect the entries into a Vec
    let values = reader
        .iter()
        .collect::<Result<Vec<String>, _>>()
        .unwrap();

    assert_eq!(values, vec!["Hello World!", "Another Entry", "Even more data"]);
}