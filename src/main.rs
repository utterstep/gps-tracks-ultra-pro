use clap::Parser;
use color_eyre::eyre::Result;

mod cli;
mod extract;

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = cli::Cli::parse();
    let sqlite = rusqlite::Connection::open(&args.sqlite_path)?;
    let gpx = extract::extract(&sqlite, args.track_name)?;

    let file = std::fs::File::create(&args.output_path)?;
    let writer = std::io::BufWriter::new(file);
    gpx::write(&gpx, writer)?;

    Ok(())
}
