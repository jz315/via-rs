use std::io::Read;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use sha2::{Digest, Sha256};

#[derive(Debug, Parser)]
#[command(about = "Maintainer-only VIA workspace tasks")]
struct Xtask {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Footprints {
        #[command(subcommand)]
        command: FootprintsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum FootprintsCommand {
    /// Build a deterministic release bundle and its SHA-256 sidecar.
    Bundle(BundleArgs),
}

#[derive(Debug, Args)]
struct BundleArgs {
    #[arg(long, default_value = via_kicad_footprints::DEFAULT_KICAD_FOOTPRINTS_VERSION)]
    version: String,
    #[arg(long, value_name = "DIR")]
    cache_dir: Option<PathBuf>,
    #[arg(long, value_name = "FILE")]
    out: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xtask = Xtask::parse();
    match xtask.command {
        Command::Footprints {
            command: FootprintsCommand::Bundle(args),
        } => bundle_footprints(args),
    }
}

fn bundle_footprints(args: BundleArgs) -> Result<(), Box<dyn std::error::Error>> {
    let report =
        via_kicad_footprints::bundle_cache_archive(&args.version, args.cache_dir, &args.out)?;
    let digest = sha256_file(&args.out)?;
    let checksum_path = PathBuf::from(format!("{}.sha256", args.out.display()));
    let bundle_name = args
        .out
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("kicad-footprints.tar.zst");
    std::fs::write(&checksum_path, format!("{digest}  {bundle_name}\n"))?;

    println!(
        "bundled {} KiCad footprints for version {} into {}",
        report.footprint_count,
        report.version,
        report.output.display()
    );
    println!("sha256: {digest}");
    println!("checksum: {}", checksum_path.display());
    Ok(())
}

fn sha256_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
