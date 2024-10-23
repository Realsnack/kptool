mod arguments;
mod models;

use keepass::{
    db::{Group, Node, NodeRef},
    error::DatabaseOpenError,
    Database, DatabaseKey,
};

use clap::Parser;

use arguments::Args;
use models::{kp_entry::KpEntry, kp_group::KpGroup, kp_tree::KpTree, kp_error::KpError};
use std::{collections::HashMap, fs::File, io};

fn main() -> Result<(), DatabaseOpenError> {
    let args = Args::parse();

    if args.debug {
        println!("Action chosen {:?}", args.command);
    }

    let checksum = calculate_checksum(&args.database);
    println!("Calculated cheksum {}", checksum);

    let mut file = File::open(args.database)?;
    let key = match &args.password {
        Some(password) => Some(DatabaseKey::new().with_password(password)),
        None => None,
    };

    let db = match key {
        Some(key) => Database::open(&mut file, key)?,
        None => panic!("No password supplied and key file not supported... yet :("),
    };

    let loaded_database = create_database_tree(&db.root).unwrap();
    let keepass_tree = KpTree::new(loaded_database, checksum);

    match &args.command {
        arguments::Commands::GetPassword { path } => {
            let password = find_password_by_path(&keepass_tree, &path);
            match password {
                Ok(pass) => println!("Password for entry '{}': '{}'", path, pass),
                Err(E) => println!("No entry found with the title '{}'", path),
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
        },
        arguments::Commands::GetPasswordNew { path } => {
            let password = find_password_by_path(&keepass_tree, path).unwrap();

            println!("{}", password);
        }
        _ => println!("Not implemented"),
    }

    Ok(())
}

// REVIEW: possibly not necessary
fn calculate_checksum(file_path: &String) -> String {
    sha256::try_digest(file_path).unwrap()
}

// REVIEW: move to kp_tree.rs
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

fn find_target_kp_group<'a>(kp_tree: &'a KpTree, splitted_search_path: &Vec<&str>) -> Result<&'a KpGroup, KpError> {
    let mut searched_group = &kp_tree.root_group;
    
    for part_path in splitted_search_path.iter().take(splitted_search_path.len() - 1) {
        if searched_group.groups.contains_key(*part_path) {
            searched_group = &searched_group.groups[*part_path];
        }
        else {
            return Err(KpError::GroupNotFound(splitted_search_path.join(".")));
        }
    }
    
    Ok(searched_group)
}

fn find_password_by_path(kp_tree: &KpTree, search_path: &str) -> Result<String, KpError> {
    let splitted_search_path: Vec<&str> = search_path.split(".").collect();
    let searched_group = find_target_kp_group(kp_tree, &splitted_search_path)?;
    
    let password_path = splitted_search_path.last().copied().unwrap();
    if searched_group.entries.contains_key(password_path) {
        // searched_group.entries.get(password_path).unwrap().password.as_ref().unwrap();
        match searched_group.entries.get(password_path) {
            None => {
                Err(KpError::EntryNotFound(search_path.to_string()))
            },
            Some(entry) => {
                match entry.password.as_ref() {
                    None => {
                        Err(KpError::PasswordNotFound(password_path.to_string()))
                    },
                    Some(password) => {
                        Ok(password.to_owned())
                    }
                }
            }
        }
    }
    else {
        Err(KpError::EntryNotFound(search_path.to_string()))
    }
}
