use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    input_data: PathBuf,

    #[arg(long)]
    output: PathBuf,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let input_data = openfx_bindgen::input_data::load_input_data(args.input_data)?;

    std::fs::create_dir_all(&args.output)?;
    openfx_bindgen::bindgen::generate_bindings(&input_data, &args.output)
        .map_err(|e| e as Box<dyn std::error::Error>)?;

    Ok(())
}
