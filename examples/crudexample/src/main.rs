use kvstore::{KeyValueDb, KeyValueDbDumpPolicy, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Define an example struct which represents a rectangle.
/// Next we'll show how to write and read it into the DB.
#[derive(Serialize, Deserialize)]
struct Rectangle {
    width: i32,
    length: i32,
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Rectangle: length={}, width={}", self.length, self.width)
    }
}

fn main() {

    // create
    // create a new DB with AutoDum, meaning every change is written to the file,
    // and with Json serialization
    let mut db = KeyValueDb::new(
        "example.db",
        KeyValueDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    );


    // set the value 100 to the key 'key1'
    db.set("key1", &2023).unwrap();

    // set the value 1.1 to the key 'key2'
    db.set("key2", &1.1).unwrap();

    // set the value 'hello world' to the key 'key3'
    db.set("key3", &String::from("hello world")).unwrap();

    // set a vector value to the key 'key4'
    db.set("key4", &vec![1, 2, 3]).unwrap();

    // set a Rectangle value to the key 'key5'
    db.set(
        "key5",
        &Rectangle {
            width: 4,
            length: 10,
        },
    )
    .unwrap();

    // print the value of key1
    println!("The value of key1 is: {}", db.get::<i32>("key1").unwrap());

    // print the value of key2
    println!("The value of key2 is: {}", db.get::<f32>("key2").unwrap());

    // print the value of key3
    println!("The value of key3 is: {}",db.get::<String>("key3").unwrap());

    // print the value of key4
    println!("The value of key4 is: {:?}",db.get::<Vec<i32>>("key4").unwrap());

    // print the value of key5
    println!(
        "The value of key5 is: {}",
        db.get::<Rectangle>("key5").unwrap()
    );


    // update
    // override the value of key1. Please note the new value is of a different type the former one
    db.set("key1", &String::from("updated")).unwrap();

    // print the value of key1
    println!("The value of key1 is: {}",db.get::<String>("key1").unwrap());

    // remove key2
    db.rem("key2").unwrap();

    // was key2 removed?
    println!("key2 was removed. Is it still in the db? {}",db.get::<f32>("key2").is_some());


    // load an existing DB from a file (the same file in this case)
    let db2 = KeyValueDb::load(
        "example.db",
        KeyValueDbDumpPolicy::DumpUponRequest,
        SerializationMethod::Json,
    )
    .unwrap();


    // verify whether the remove operation is completed
    // print the value of key1
    println!("Value of key1 as loaded from file is: {}",db2.get::<String>("key1").unwrap());


    // read
    // iterate over all keys and values in the db
    for kv in db.iter() {
        match kv.get_key() {
            "key1" => println!("Value of {} is: {}",kv.get_key(),kv.get_value::<String>().unwrap()),
            "key3" => println!("Value of {} is: {}",kv.get_key(),kv.get_value::<String>().unwrap()),
            "key4" => println!("Value of {} is: {:?}",kv.get_key(),kv.get_value::<Vec<i32>>().unwrap()),
            "key5" => println!("Value of {} is: {}",kv.get_key(),kv.get_value::<Rectangle>().unwrap()),_ => (),}
    }

    
}
