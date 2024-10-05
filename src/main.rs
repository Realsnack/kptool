mod arguments;
mod models;

use keepass::{
    db::{Group, Node, NodeRef},
    error::DatabaseOpenError,
    Database, DatabaseKey,
};

use clap::Parser;

use arguments::Args;
use models::{kp_entry::KpEntry, kp_group::KpGroup};
use std::{collections::HashMap, fs::File};

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
        }
        arguments::Commands::FillTemplate { file_path } => {
            let _ = file_path;
            match create_database_tree(&db.root) {
                None => {
                    println!("Unfortunatelly, no entries have been found");
                    std::process::exit(1);
                }
                Some(root_group) => {
                    println!("Haha");
                }
            }
        }
        _ => println!("Not implemented"),
    }

    Ok(())
}

fn create_database_tree(db_group: &Group) -> Option<KpGroup> {
    let mut root_group = KpGroup::new();
    create_group_node(&db_group, &mut root_group);

    if root_group.entries.len() == 0 && root_group.groups.len() == 0 {
        return None;
    }

    Some(root_group)
}

fn create_group_node(group_ref: &Group, parent_group: &mut KpGroup) {
    for node in &group_ref.children {
        match node {
            Node::Group(g) => {
                println!("Group '{}' added under: {}", g.name, group_ref.name);
                let mut added_group = KpGroup::new();
                create_group_node(&g, &mut added_group);
                parent_group.groups.insert(g.clone().name, added_group);
            }
            Node::Entry(e) => {
                let title = e.get_title().unwrap_or("(no title)");
                let user = e.get_username().unwrap_or("(no username)");
                let pass = e.get_password().unwrap_or("(no password)");
                println!("Adding '{}' to tree, under: {}", title, group_ref.name);
                parent_group.entries.insert(
                    e.get_title().unwrap_or("(no title)").to_string(),
                    KpEntry::new(Some(user.to_string()), Some(pass.to_string())),
                );
            }
        }
    }
}

fn find_password_by_path(db: &Database, full_path: &str) -> Option<String> {
    let path_parts: Vec<&str> = full_path.split('/').collect();
    find_entry_in_group(&db.root, &path_parts)
}

fn find_entry_in_group(group: &Group, path_parts: &[&str]) -> Option<String> {
    if path_parts.is_empty() {
        return None;
    }

    let current_group_or_entry = path_parts[0]; // The current part of the path

    for node in &group.children {
        match node {
            Node::Group(g) => {
                if g.name == current_group_or_entry {
                    return find_entry_in_group(g, &path_parts[1..]);
                }
            }
            Node::Entry(e) => {
                if e.get_title()
                    .map_or(false, |title| title == current_group_or_entry)
                    && path_parts.len() == 1
                {
                    return e.get_password().map(|s| s.to_string());
                }
            }
        }
    }

    None // No matching group or entry found
}
