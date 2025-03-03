use clap::{Args, Parser, Subcommand};
use destore_tools::{unpack_partition, Cache, SchemaRestorer};
use espflash::cli::config::Config;
use espflash::cli::{connect, ConnectArgs};
use espflash::targets::Chip;
use log::{info, LevelFilter};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_module("espflash", LevelFilter::Warn)
        .filter_level(LevelFilter::Info)
        .format_timestamp(None)
        .format_module_path(false)
        .parse_default_env()
        .init();

    let cli = Cli::parse();
    cli.command.run().await
}

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Dump the destore records from a connected device and output them to stdout
    Dump(DumpCommand),

    /// Decodes a destore partition file and outputs the records to stdout
    Decode(DecodeCommand),

    /// Intercepts (& executes) the command and stores the postcard schema found in the ELF file
    Proxy(ProxyCommand),
}

impl Commands {
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Commands::Dump(cmd) => cmd.run(),
            Commands::Decode(cmd) => cmd.run(),
            Commands::Proxy(cmd) => cmd.run(),
        }
    }
}

#[derive(Args)]
pub struct DumpCommand {
    #[clap(value_parser=clap_num::maybe_hex::<u32>)]
    start: u32,

    #[clap(value_parser=clap_num::maybe_hex::<u32>)]
    size: u32,

    /// Store the partition to this file. It can later be analyzed with the decode command
    #[clap(long)]
    store_partition: Option<PathBuf>,

    #[clap(flatten)]
    common_args: CommonArgs,

    #[clap(flatten)]
    connect_args: ConnectArgs,
}

#[derive(Args)]
pub struct DecodeCommand {
    /// The partition file to decode
    part: PathBuf,

    #[clap(flatten)]
    common_args: CommonArgs,
}

#[derive(Args)]
pub struct ProxyCommand {
    #[arg(last = true)]
    args: Vec<String>,
}

impl ProxyCommand {
    fn run(self) -> anyhow::Result<()> {
        if let Some(last) = self.args.last() {
            if fs::exists(last)? {
                let last = last.clone();
                std::thread::spawn(move || -> anyhow::Result<()> {
                    let elf = SchemaRestorer::from_path(last)?;
                    let schema = elf.load_schema_from_symbol("_DESTORE_SCHEMA")?;
                    //info!("Schema found: {:?}", &schema);
                    let mut cache = Cache::new();
                    cache.store(&schema)?;
                    Ok(())
                });
            }
        }

        // run the command in args
        let output = std::process::Command::new(&self.args[0])
            .args(&self.args[1..])
            .output()?;
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;

        std::process::exit(output.status.code().unwrap());
    }
}

#[derive(Args)]
pub struct CommonArgs {}

impl DumpCommand {
    fn run(mut self) -> anyhow::Result<()> {
        if self.connect_args.chip.is_none() {
            self.connect_args.chip = Some(Chip::Esp32c6);
        }

        let mut flasher = connect(
            &self.connect_args,
            &Config::load()
                .map_err(|e| anyhow::anyhow!("Failed to read espflash config {:?}", e))?,
            false,
            false,
        )
        .map_err(|e| anyhow::anyhow!("Failed to connect to device: {:?}", e))?;

        let mut vec = Vec::new();
        let mut rest = self.size;
        let mut offset = self.start;
        while rest > 0 {
            let tmp_file = NamedTempFile::new()?;
            flasher.read_flash(offset, 4096, 4096, 64, tmp_file.path().to_path_buf())?;
            let buf = fs::read(tmp_file.path())?;

            assert_eq!(buf.len(), 4096);
            vec.extend_from_slice(buf.as_slice());
            if buf.iter().all(|&b| b == 0xFF) {
                break;
            }
            rest -= 4096;
            offset += 4096;
        }

        if let Some(store_path) = self.store_partition.as_ref() {
            fs::write(store_path, &vec)?;
            info!("Partition stored to {:?}", store_path);
        }

        output_records(vec.as_mut_slice(), &self.common_args)
    }
}

impl DecodeCommand {
    fn run(self) -> anyhow::Result<()> {
        if !self.part.exists() {
            return Err(anyhow::anyhow!(
                "Partition file {:?} does not exist",
                self.part
            ));
        }

        let mut partition = fs::read(&self.part)?;
        output_records(&mut partition, &self.common_args)
    }
}

fn output_records(partition: &mut [u8], _common_args: &CommonArgs) -> anyhow::Result<()> {
    unpack_partition(partition)?;

    Ok(())
}
