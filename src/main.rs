mod arguments;

use keepass::{
    db::{Group, Node, NodeRef},
    error::DatabaseOpenError,
    Database, DatabaseKey,
};

use clap::Parser;

use arguments::Args;
use std::fs::File;

fn main() -> Result<(), DatabaseOpenError> {
    let args = Args::parse();

    if args.debug {
        println!("Action chosen {:?}", args.command);
    }

    let mut file = File::open(args.database)?;
    let key = match &args.password {
        Some(password) => Some(DatabaseKey::new().with_password(password)),
        None => None,
    };

    let db = match key {
        Some(key) => Database::open(&mut file, key)?,
        None => panic!("No password supplied and key file not supported... yet :("),
    };

    match &args.command {
        arguments::Commands::GetPassword { path } => {
            let password = find_password_by_path(&db, &path);
            match password {
                Some(pass) => println!("Password for entry '{}': '{}'", path, pass),
                None => println!("No entry found with the title '{}'", path),
            }
        },
        _ => println!("Not implemented")
    }

    // for node in &db.root {
    //     match node {
    //         NodeRef::Group(g) => {
    //             println!("Saw group '{0}'", g.name);
    //         }
    //         NodeRef::Entry(e) => {
    //             let title = e.get_title().unwrap_or("(no title)");
    //             let user = e.get_username().unwrap_or("(no username)");
    //             let pass = e.get_password().unwrap_or("(no password)");
    //             println!("Entry '{0}': '{1}' : '{2}'", title, user, pass);
    //         }
    //     }
    // }

    Ok(())
}

fn find_password_by_path(db: &Database, full_path: &str) -> Option<String> {
    let path_parts: Vec<&str> = full_path.split('/').collect();
    find_entry_in_group(&db.root, &path_parts)
}

fn find_entry_in_group(group: &Group, path_parts: &[&str]) -> Option<String> {
    if path_parts.is_empty() {
        return None; // No path to search
    }

    let current_group_or_entry = path_parts[0]; // The current part of the path

    // Traverse groups and entries in the current group
    for node in &group.children {
        match node {
            Node::Group(g) => {
                if g.name == current_group_or_entry {
                    // Recursively search in this group
                    return find_entry_in_group(g, &path_parts[1..]);
                }
            }
            Node::Entry(e) => {
                if e.get_title()
                    .map_or(false, |title| title == current_group_or_entry)
                    && path_parts.len() == 1
                {
                    // Found the entry; return its password
                    return e.get_password().map(|s| s.to_string());
                }
            }
        }
    }

    None // No matching group or entry found
}
