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
    let mut writer: SimpleJournalWriter<String> = SimpleJournalWriter::new(file);
    writer.store_entry("Hello World!".into()).unwrap();
    writer.store_entry("Another Entry".into()).unwrap();
    writer.store_entry("Even more data".into()).unwrap();
    for i in 4..100 {
        writer.store_entry(format!("This is entry no {}.", i)).unwrap();
    }
}

pub fn read(file: &mut File) {
    let mut reader: SimpleIndexedJournal<String> = SimpleIndexedJournal::new(file)
        .unwrap(); // creating an IndexedJournal will scan the whole file which might fail, thus it returns a Result

    // You can use IndexedJournal just like JournalReader ...
    for value in &mut reader {
        let unwrapped = value.unwrap();
        println!("Entry: {}", unwrapped);
    }

    let values = reader
        .iter()
        .take(5) // just check the first five entries
        .collect::<Result<Vec<String>, _>>()
        .unwrap();

    assert_eq!(values, vec!["Hello World!", "Another Entry", "Even more data", "This is entry no 4.", "This is entry no 5."]);

    // But you can also do random access requests
    let fifth = reader.load_entry(4).unwrap();
    assert_eq!(fifth, "This is entry no 5.");
    let fourteenth = reader.load_entry(13).unwrap();
    assert_eq!(fourteenth, "This is entry no 14.");

    // If the provided index is out of bounds you'll get an Err
    if let JournalError::IndexOutOfBounds = reader.load_entry(200).unwrap_err() {
        // Handle Error
    } else {
        panic!();
    }

    // You can also start iterating at a specific position
    // This is faster than loading consecutive entries separately because a buffered reader is used
    let values = reader.iter_from(20)
        .unwrap() // This can fail if index is out of bounds
        .take(3)
        .collect::<Result<Vec<String>, _>>()
        .unwrap();

    assert_eq!(values, vec![
        "This is entry no 21.",
        "This is entry no 22.",
        "This is entry no 23."]);
}