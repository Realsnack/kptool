use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long)]
    pub database: String,

    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Get { path: String },
    GetUsername { path: String },
    GetPassword { path: String },
}
