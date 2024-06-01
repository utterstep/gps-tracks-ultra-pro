use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long)]
    pub sqlite_path: String,

    #[arg(short, long)]
    pub output_path: String,

    #[arg(short, long)]
    pub track_name: Option<String>,
}
