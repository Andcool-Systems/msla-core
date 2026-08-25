use std::fs::File;
mod gcode;

use anyhow::Result;
use tempfile::{TempDir, tempdir};
use tokio::fs;
use zip::ZipArchive;

use crate::input::{Model, zip::gcode::parse_gcode};

/// Load zip model from file
pub async fn load_zip_model(zip_path: impl AsRef<std::path::Path>) -> Result<Model> {
    let temp_dir = open_zip_temp(zip_path)?;

    let gcode_file = temp_dir.path().join("run.gcode");
    let gcode = fs::read_to_string(gcode_file).await?;

    let ir = parse_gcode(gcode)?;

    Ok(Model {
        ir,
        working_dir: temp_dir,
        model_preview: None,
    })
}

/// Creates temp dir and extracts zip archive into it
fn open_zip_temp(zip_path: impl AsRef<std::path::Path>) -> Result<TempDir> {
    let temp_dir = tempdir()?;

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    archive.extract(temp_dir.path())?;

    Ok(temp_dir)
}
