use anyhow::Result;
use clap::Parser;
use std::path::Path;

#[derive(Debug, Parser)]
struct Cli {
    /// The file path
    path: Box<Path>,

    /// Run the cowlang vizualizer
    #[arg(short, long)]
    vizualize: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let text = tokio::fs::read_to_string(&cli.path).await?;
    let program = cowlang::Program::parse(&text);

    if cli.vizualize {
        cowlang_viz::vizualize(cowlang_viz::Options { program }).await?;
    } else {
        todo!()
    }

    Ok(())
}
