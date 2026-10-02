use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    input_data: PathBuf,

    #[arg(long)]
    output: PathBuf,
}

pub fn main() {
    let args = Args::parse();

    let input_data = openfx_bindgen::input_data::load_input_data(args.input_data)
        .expect("Failed to load input data");

    std::fs::create_dir_all(&args.output).expect("Failed to create folder `c_bindings`");
    openfx_bindgen::bindgen::generate_bindings(&input_data, &args.output)
        .expect("Failed to generate bindings");
}
