//
// Copyright (c) 2022 Nathan Fiedler
//
use simpledb::store::Database;

fn main() {
    // based on the first example from the wiki by nehaleadz
    let mut database = Database::new();
    database.begin();
    database.set("a", "10");
    println!("a = {}", database.get("a").unwrap()); // 10
    database.begin();
    database.set("a", "20");
    println!("a = {}", database.get("a").unwrap()); // 20
    database.rollback();
    println!("a = {}", database.get("a").unwrap()); // 10
    database.rollback();
    println!("a = {:?}", database.get("a")); // None
}
