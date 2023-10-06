use std::path::PathBuf;

use ethers::prelude::*;

const ABI_PATH: &str = "erc20_abi.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_file: PathBuf = [".", "src", "erc20.rs"].iter().collect();
    // std::env::temp_dir()
    // let out_file = PathBuf::from(".").join("src\erc20.rs");
    println!(
        "cargo:warning=create contract rust file: {:?}",
        out_file.clone().to_string_lossy()
    );
    if out_file.exists() {
        std::fs::remove_file(&out_file)?;
    }
    Abigen::new("Erc20Token", ABI_PATH)?
        .generate()?
        .write_to_file(out_file)?;
    Ok(())
}
