use std::path::PathBuf;
use std::env;
use destore_tools::DestoreElf;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        anyhow::bail!("Usage: {} <elf-file>", args[0]);
    }

    let elf_path = PathBuf::from(&args[1]);
    if !elf_path.exists() {
        anyhow::bail!("File not found: {}", elf_path.display());
    }

    let elf = DestoreElf::from_path(&elf_path)?;
    let schema = elf.load_schema()?;
    
    println!("Loaded schema: {:#?}", schema);

    Ok(())
}
