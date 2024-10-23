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

    #[arg(long)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Get { path: String },
    GetUsername { path: String },
    GetPassword { path: String },
    FillTemplate { file_path: String },
    GetPasswordNew { path: String }
}
