mod arguments;
mod models;

use keepass::{
    db::{Group, Node},
    error::DatabaseOpenError,
    Database, DatabaseKey,
};

use clap::Parser;

use arguments::Args;
use models::{kp_entry::KpEntry, kp_error::KpError, kp_group::KpGroup, kp_tree::KpTree};
use regex::{self, Regex};
use std::{
    fs::{read_to_string, File}, io::{self, Write},
};

fn main() -> Result<(), DatabaseOpenError> {
    let args = Args::parse();

    if args.debug {
        println!("Action chosen {:?}", args.command);
    }

    let mut file = File::open(args.database)?;
    let key = args.password.as_ref().map(|password| DatabaseKey::new().with_password(password));

    let db = match key {
        Some(key) => Database::open(&mut file, key)?,
        None => panic!("No password supplied and key file not supported... yet :("),
    };

    let loaded_database = create_database_tree(&db.root).unwrap();
    let keepass_tree = KpTree::new(loaded_database);

    match &args.command {
        arguments::Commands::GetEntry { path } => match find_entry_by_path(&keepass_tree, path) {
            Ok(entry) => {
                println!("Entry detail: {:?}", entry);
            }
            Err(e) => println!("ERROR: {}", e),
        },
        arguments::Commands::GetUsername { path } => {
            match find_username_by_path(&keepass_tree, path) {
                Ok(username) => println!("Found username: {}", username),
                Err(e) => println!("ERROR: {}", e),
            }
        }
        arguments::Commands::GetPassword { path } => {
            match find_password_by_path(&keepass_tree, path) {
                Ok(pass) => println!("Password for entry '{}': '{}'", path, pass),
                Err(e) => println!("ERROR: {}", e),
            }
        }
        arguments::Commands::FillTemplate { file_path } => {
            match fill_template(&keepass_tree, file_path) {
                Ok(_) => std::process::exit(0),
                Err(e) => {
                    println!("ERROR: Following errors found: \n{}", e);
                    std::process::exit(1);
                }
            }
            // Open file
            // Find all regex matches
            // Find all variables in keepass_tree
            // - If any variable is missing, add it to ERROR list that will be printed out at the end
            // If no errors were found, replace all regex matches with the variables and save the file
        }
    }

    Ok(())
}

fn fill_template(kp_tree: &KpTree, file_path: &String) -> Result<bool, KpError> {
    let mut variables: Vec<(String, String)> = Vec::new();
    let mut errors: Vec<(String, KpError)> = Vec::new();

    if let Ok(matches) = get_regex_matches(file_path) {
        for match_tuple in matches {
            match find_entry_by_path(kp_tree, match_tuple.1.as_str()) {
                Ok(entry) => {
                    // DEBUG: println!("Found entry for: {}", match_tuple.0);
                    variables.push((match_tuple.0, entry.password.as_ref().unwrap().to_owned()));
                }
                Err(e) => {
                    errors.push((match_tuple.0, e));
                }
            }
        }
    }

    if variables.is_empty() {
        return Err(KpError::NoVariablesInSourceFile(file_path.to_owned()));
    }

    if !errors.is_empty() {
        println!("Errors: {:?}", errors);
        return Err(KpError::TemplateVariablesNotFound(errors));
    }

    let _ = write_filled_template(file_path, variables);

    Ok(true)
}


fn write_filled_template(file_path: &String, variables: Vec<(String, String)>) -> io::Result<()> {
    let file_contents = prepare_template_export(file_path, variables);
    let mut file = File::create("output.yaml")?;
    
    for line in file_contents {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

fn prepare_template_export(file_path: &String, variables: Vec<(String, String)>) -> Vec<String> {
    let re = Regex::new(
        r"\{\{\s*(?<search_path>[a-zA-Z\._0-9]{1,}):(?<entry_type>USERNAME|PASSWORD)\s*\}\}",
    )
    .unwrap();

    let mut file_vec: Vec<String> = Vec::new();

    {
        for line in read_to_string(file_path).unwrap().lines() {
            if let Some(matches) = re.captures(line) {
                if let Some(full_match) = matches.get(0) {
                    // find correct tuple
                    let secret_tuple = variables
                        .iter()
                        .find(|tuple| tuple.0 == full_match.as_str());

                    file_vec.push(line.replace(full_match.as_str(), secret_tuple.unwrap().1.as_str()));
                }
            }
            else {
                file_vec.push(line.to_owned());
            }
        }
    }

    // println!("DEBUG: {:?}", file_vec);
    file_vec
}

fn get_regex_matches(file_path: &String) -> Result<Vec<(String, String, String)>, KpError> {
    let re = Regex::new(
        r"\{\{\s*(?<search_path>[a-zA-Z\._0-9]{1,}):(?<entry_type>USERNAME|PASSWORD)\s*\}\}",
    )
    .unwrap();

    // the tuple is <matched_string, search_path, entry_type>
    let mut search_vec: Vec<(String, String, String)> = Vec::new();
    {
        for line in read_to_string(file_path).unwrap().lines() {
            // DEBUG: ? println!("Line read: {}", line);
            if let Some(matches) = re.captures(line) {
                let mut regex_match: String = String::from("value");
                if let Some(full_match) = matches.get(0) {
                    // DEBUG: println!("Full match: {}", full_match.as_str());
                    regex_match = full_match.as_str().to_owned();
                }

                // DEBUG: ? println!("Found match - Group:{}, Type: {}", &matches["search_path"], &matches["entry_type"]);
                search_vec.push((
                    regex_match,
                    matches["search_path"].to_owned(),
                    matches["entry_type"].to_owned(),
                ));
            }
        }
    }

    // DEBUG: println!("Vec result: {:?}", search_vec);
    Ok(search_vec)
}

// REVIEW: move to kp_tree.rs
fn create_database_tree(db_group: &Group) -> Option<KpGroup> {
    let mut root_group = KpGroup::new();
    create_group_node(db_group, &mut root_group);

    if root_group.entries.is_empty() && root_group.groups.is_empty() {
        return None;
    }

    Some(root_group)
}

fn create_group_node(group_ref: &Group, parent_group: &mut KpGroup) {
    for node in &group_ref.children {
        match node {
            Node::Group(g) => {
                // DEBUG: println!("Group '{}' added under: {}", g.name, group_ref.name);
                let mut added_group = KpGroup::new();
                create_group_node(g, &mut added_group);
                parent_group.groups.insert(g.clone().name, added_group);
            }
            Node::Entry(e) => {
                let user = e.get_username().unwrap_or("(no username)");
                let pass = e.get_password().unwrap_or("(no password)");
                // DEBUG: println!("Adding '{}' to tree, under: {}", title, group_ref.name);
                parent_group.entries.insert(
                    e.get_title().unwrap_or("(no title)").to_string(),
                    KpEntry::new(Some(user.to_string()), Some(pass.to_string())),
                );
            }
        }
    }
}

fn find_target_kp_group<'a>(
    kp_tree: &'a KpTree,
    splitted_search_path: &[&str],
) -> Result<&'a KpGroup, KpError> {
    let mut searched_group = &kp_tree.root_group;

    for part_path in splitted_search_path
        .iter()
        .take(splitted_search_path.len() - 1)
    {
        if searched_group.groups.contains_key(*part_path) {
            searched_group = &searched_group.groups[*part_path];
        } else {
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
            None => Err(KpError::EntryNotFound(search_path.to_string())),
            Some(entry) => match entry.password.as_ref() {
                None => Err(KpError::PasswordNotFound(password_path.to_string())),
                Some(password) => Ok(password.to_owned()),
            },
        }
    } else {
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
            None => Err(KpError::EntryNotFound(search_path.to_string())),
            Some(entry) => Ok(entry),
        }
    } else {
        Err(KpError::EntryNotFound(search_path.to_string()))
    }
}

fn find_username_by_path(kp_tree: &KpTree, search_path: &str) -> Result<String, KpError> {
    match find_entry_by_path(kp_tree, search_path) {
        Ok(entry) => match &entry.username {
            Some(username) => Ok(username.clone()),
            None => Err(KpError::UsernameNotFound(search_path.to_string())),
        },
        Err(e) => Err(e),
    }
}
