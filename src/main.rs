//
// Copyright (c) 2022 Nathan Fiedler
//
use simpledb::store::Database;
use std::io::{self, Write};

fn eval_and_print(database: &mut Database, line: &str) {
    // The names and values do _not_ contain spaces, which makes the process of
    // parsing the commands nothing more than splitting on whitespace.
    let mut iter = line.split_whitespace();
    if let Some(cmd) = iter.next() {
        if cmd == "END" {
            std::process::exit(0);
        } else if cmd == "SET" {
            if let Some(name) = iter.next() {
                if let Some(value) = iter.next() {
                    database.set(name, value);
                } else {
                    println!("missing value for SET");
                }
            } else {
                println!("missing name for SET");
            }
        } else if cmd == "GET" {
            if let Some(name) = iter.next() {
                if let Some(value) = database.get(name) {
                    println!("{}", value);
                } else {
                    println!("NULL")
                }
            } else {
                println!("missing name for GET");
            }
        } else if cmd == "UNSET" {
            if let Some(name) = iter.next() {
                database.delete(name);
            } else {
                println!("missing name for UNSET");
            }
        } else if cmd == "NUMEQUALTO" {
            if let Some(value) = iter.next() {
                let count = database.count(value);
                println!("{}", count);
            } else {
                println!("missing value for NUMEQUALTO");
            }
        } else if cmd == "BEGIN" {
            database.begin();
        } else if cmd == "ROLLBACK" {
            if !database.rollback() {
                println!("NO TRANSACTION");
            }
        } else if cmd == "COMMIT" {
            if !database.commit() {
                println!("NO TRANSACTION");
            }
        } else {
            println!("unknown command: {}", cmd);
        }
    }
}

fn main() {
    let mut database = Database::new();
    // the read-eval-print-loop
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => eval_and_print(&mut database, &input),
            Err(err) => println!("error: {:?}", err),
        }
    }
}
