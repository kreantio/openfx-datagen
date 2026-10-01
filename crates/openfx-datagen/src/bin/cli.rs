use std::{collections::BTreeMap, path::PathBuf};

use clap::{Parser, Subcommand};
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use openfx_datagen::{
    parsing::{BindingsUnprocessed, parse},
    processing::{Bindings, process},
};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    GenData(CommandGenData),
    GenSchemata(CommandGenSchemata),
}

#[derive(Debug, Parser)]
struct CommandGenData {
    /// the path to the input C headers directory
    #[arg(long)]
    input_c_headers: PathBuf,

    /// the path to the output directory for generated data
    #[arg(long)]
    output_data: PathBuf,
}

#[derive(Debug, Parser)]
struct CommandGenSchemata {
    /// the path to the output directory for generated schemata
    #[arg(long)]
    output_schemata: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        Commands::GenData(cmd) => gen_data(cmd),
        Commands::GenSchemata(cmd) => gen_schemata(cmd),
    }
}

fn gen_data(cmd: CommandGenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut input_entries: Vec<(std::fs::DirEntry, String)> = vec![];
    for entry in std::fs::read_dir(&cmd.input_c_headers)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() || path.extension().is_none_or(|ext| ext != "h") {
            continue;
        }
        let name = path
            .file_name()
            .ok_or_else(|| format!("Failed to get file name for path: {:?}", path))?
            .to_string_lossy()
            .to_string();
        if !name.starts_with("ofx") {
            continue;
        }
        input_entries.push((entry, name));
    }

    let parsed_headers: BTreeMap<_, _> =
        input_entries
            .par_iter()
            .map(
                |(entry, name)| -> Result<
                    (String, BindingsUnprocessed),
                    Box<dyn std::error::Error + Send + Sync>,
                > {
                    let code = std::fs::read_to_string(entry.path())?;
                    Ok((name.to_owned(), parse(&code)?))
                },
            )
            .collect::<Result<BTreeMap<_, _>, _>>()?;

    let processed_bindings = process(parsed_headers)?;

    let output_bindings_path = cmd.output_data.join("bindings");
    std::fs::create_dir_all(&output_bindings_path)?;

    processed_bindings.par_iter().try_for_each(
        |(name, bindings)| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let output_path = output_bindings_path.join(format!("{}.json", name));
            let file = std::fs::File::create(&output_path)?;
            bindings.write_json_pretty(file)?;

            Ok(())
        },
    )?;

    Ok(())
}

fn gen_schemata(cmd: CommandGenSchemata) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output_schemata_path = cmd.output_schemata;
    std::fs::create_dir_all(&output_schemata_path)?;
    let output_bindings_schema_path = output_schemata_path.join("bindings.current.schema.json");
    let file = std::fs::File::create(&output_bindings_schema_path)?;
    Bindings::write_schema_json_pretty(file)?;

    Ok(())
}
