mod arguments;

use keepass::{
    error::DatabaseOpenError,
    DatabaseKey,
    Database,
    db::NodeRef
};

use clap::Parser;

use std::fs::File;
use arguments::Args;

fn main() -> Result<(), DatabaseOpenError> {
    let args = Args::parse();
    
    println!("Will be doing: {:?}", args.command);

    let mut file = File::open(args.database)?;
    let key = match &args.password {
        Some(password) => Some(DatabaseKey::new().with_password(password)),
        None => None,
    };

    let db = match key {
        Some(key) => Database::open(&mut file, key)?,
        None => panic!("No password supplied and key file not supported... yet :(")
    };

    for node in &db.root {
        match node {
            NodeRef::Group(g) => {
                println!("Saw group '{0}'", g.name);
            },
            NodeRef::Entry(e) => {
                let title = e.get_title().unwrap_or("(no title)");
                let user = e.get_username().unwrap_or("(no username)");
                let pass = e.get_password().unwrap_or("(no password)");
                println!("Entry '{0}': '{1}' : '{2}'", title, user, pass);
            }
        }
    }

    Ok(())
}
