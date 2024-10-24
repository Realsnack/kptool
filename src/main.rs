mod arguments;
mod models;

use keepass::{
    db::{Group, Node},
    error::DatabaseOpenError,
    Database, DatabaseKey,
};

use clap::Parser;

use arguments::Args;
use models::{kp_entry::KpEntry, kp_group::KpGroup, kp_tree::KpTree, kp_error::KpError};
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

    let loaded_database = create_database_tree(&db.root).unwrap();
    let keepass_tree = KpTree::new(loaded_database);

    match &args.command {
        arguments::Commands::GetEntry { path } => {
            match find_entry_by_path(&keepass_tree, &path) {
                Ok(entry) => {
                    println!("Entry detail: {:?}", entry);
                },
                Err(e) => println!("ERROR: {}", e),
            }
        },
        arguments::Commands::GetUsername { path } => {
            match find_username_by_path(&keepass_tree, &path) {
                Ok(username) => println!("Found username: {}", username),
                Err(e) => println!("ERROR: {}", e),
            }
        },
        arguments::Commands::GetPassword { path } => {
            match find_password_by_path(&keepass_tree, &path) {
                Ok(pass) => println!("Password for entry '{}': '{}'", path, pass),
                Err(e) => println!("ERROR: {}", e),
            }
        }
        arguments::Commands::FillTemplate { file_path } => {
            todo!("Not implemented... yet!");
        },
    }

    Ok(())
}

// REVIEW: move to kp_tree.rs
fn create_database_tree(db_group: &Group) -> Option<KpGroup> {
    let mut root_group = KpGroup::new();
    create_group_node(&db_group, &mut root_group);

    if root_group.entries.is_empty() && root_group.groups.is_empty() {
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

fn find_entry_by_path<'a>(kp_tree: &'a KpTree, search_path: &str) -> Result<&'a KpEntry, KpError> {
    let splitted_search_path: Vec<&str> = search_path.split(".").collect();
    let searched_group = find_target_kp_group(kp_tree, &splitted_search_path)?;
    
    let entry_path = splitted_search_path.last().copied().unwrap();
    if searched_group.entries.contains_key(entry_path) {
        // searched_group.entries.get(entry_path).unwrap().password.as_ref().unwrap();
        match searched_group.entries.get(entry_path) {
            None => {
                Err(KpError::EntryNotFound(search_path.to_string()))
            },
            Some(entry) => {
                Ok(entry)
            }
        }
    }
    else {
        Err(KpError::EntryNotFound(search_path.to_string()))
    }
}

fn find_username_by_path(kp_tree: &KpTree, search_path: &str) -> Result<String, KpError> {
    match find_entry_by_path(kp_tree, search_path) {
        Ok(entry) => {
            match &entry.username {
                Some(username) => {
                    Ok(username.clone())
                },
                None => {
                    Err(KpError::UsernameNotFound(search_path.to_string()))
                }
            }
        },
        Err(e) => Err(e),
    }
}
