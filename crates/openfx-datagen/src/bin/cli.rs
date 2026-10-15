use std::{collections::BTreeMap, path::PathBuf};

use clap::{Parser, Subcommand};
use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator as _};

use openfx_datagen::{
    metadata_extracting::{Metadata, extract_metadata},
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
    #[clap(long, value_delimiter = ',', num_args(1..))]
    dry_run: Vec<CommandGenDataDryRunValue>,

    /// the path to the input C headers directory
    #[arg(long)]
    input_c_headers: PathBuf,

    /// the path to the output directory for generated data for bindings
    #[arg(long)]
    output_bindings_data: Option<PathBuf>,

    /// the path to the output directory for generated metadata
    #[arg(long)]
    output_metadata: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, clap::ValueEnum)]
enum CommandGenDataDryRunValue {
    BindingsData,
    Metadata,
}

#[derive(Debug, Parser)]
struct CommandGenSchemata {
    #[clap(long)]
    dry_run: bool,

    /// the path to the output directory for generated schemata
    #[arg(long, required_unless_present = "dry_run")]
    output_schemata: Option<PathBuf>,
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
    if cmd.dry_run.is_empty() {
        tracing::info!("Generating data…");
    } else {
        tracing::info!("Generating data… This is a dry run.");
        if cmd.output_bindings_data.is_some() || cmd.output_metadata.is_some() {
            return Err("Output paths should not be specified for a dry run.".into());
        }
    }

    let parsed_headers = {
        tracing::info!("Parsing input C headers…");

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

        let parsed_headers: BTreeMap<_, _> = input_entries
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

        parsed_headers
    };

    let metadata = if cmd.output_metadata.is_some()
        || cmd.dry_run.contains(&CommandGenDataDryRunValue::Metadata)
    {
        tracing::info!("Generating metadata from parsed headers…");
        Some(extract_metadata(&parsed_headers)?)
    } else {
        None
    };

    let processed_bindings = if cmd.output_bindings_data.is_some()
        || cmd
            .dry_run
            .contains(&CommandGenDataDryRunValue::BindingsData)
    {
        tracing::info!("Generating bindings data from parsed headers…");
        Some(process(parsed_headers)?)
    } else {
        None
    };

    if let Some(processed_bindings) = processed_bindings
        && let Some(output_bindings_path) = cmd.output_bindings_data
    {
        tracing::info!(
            "Writing the generated bindings data to: {:?}",
            output_bindings_path
        );

        std::fs::create_dir_all(&output_bindings_path)?;

        processed_bindings.par_iter().try_for_each(
            |(name, bindings)| -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let output_path = output_bindings_path.join(format!("{}.json", name));
                let file = std::fs::File::create(&output_path)?;
                bindings.write_json_pretty(file)?;

                Ok(())
            },
        )?;
    }

    if let Some(metadata) = metadata
        && let Some(output_metadata_path) = cmd.output_metadata
    {
        tracing::info!(
            "Writing the generated metadata to: {:?}",
            output_metadata_path
        );

        std::fs::create_dir_all(&output_metadata_path)?;

        let output_path = output_metadata_path.join("raw.json");
        let file = std::fs::File::create(&output_path)?;
        serde_json::to_writer_pretty(file, &metadata)?;
    }

    Ok(())
}

fn gen_schemata(cmd: CommandGenSchemata) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if cmd.dry_run {
        tracing::info!("Generating schemata… This is a dry run.");
        if cmd.output_schemata.is_some() {
            return Err("Output paths should not be specified for a dry run.".into());
        }
    } else {
        tracing::info!("Generating schemata…");
    }

    let output_schemata_path = cmd.output_schemata;
    if let Some(output_schemata_path) = &output_schemata_path {
        std::fs::create_dir_all(output_schemata_path)?;
    }

    {
        let (writer, writer_desc): (Box<dyn std::io::Write>, String) = match &output_schemata_path {
            Some(path) => {
                let file_path = path.join("bindings.current.schema.json");
                (
                    Box::new(std::fs::File::create(&file_path)?),
                    format!("a file at {:?}", file_path),
                )
            }
            None => (Box::new(std::io::sink()), "a sink".to_string()),
        };
        tracing::info!("Writing the current bindings schema to: {writer_desc}");
        Bindings::write_schema_json_pretty(writer)?;
    }

    {
        let (writer, writer_desc): (Box<dyn std::io::Write>, String) = match &output_schemata_path {
            Some(path) => {
                let file_path = path.join("metadata-raw.current.schema.json");
                (
                    Box::new(std::fs::File::create(&file_path)?),
                    format!("a file at {:?}", file_path),
                )
            }
            None => (Box::new(std::io::sink()), "a sink".to_string()),
        };
        tracing::info!("Writing the current metadata schema to: {writer_desc}");
        Metadata::write_schema_json_pretty(writer)?;
    }

    Ok(())
}
