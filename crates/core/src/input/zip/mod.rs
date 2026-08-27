use std::fs::File;
mod gcode;

use anyhow::{Result, anyhow};
use tempfile::{TempDir, tempdir};
use tokio::fs;
use zip::ZipArchive;

use crate::input::{Model, zip::gcode::GCodeParser};

/// Load zip model from file
pub async fn load_zip_model(zip_path: impl AsRef<std::path::Path>) -> Result<Model> {
    let temp_dir = open_zip_temp(zip_path)?;

    let gcode_file = temp_dir.path().join("run.gcode");
    let gcode = fs::read_to_string(gcode_file)
        .await
        .map_err(|e| anyhow!("Cannot open gcode file: {}", e))?;

    let mut gparser = GCodeParser::new();
    gparser.parse_gcode(gcode)?;

    Ok(Model {
        ir: gparser.ir,
        model_meta: gparser.meta,
        working_dir: temp_dir,
        model_preview: None,
    })
}

/// Creates temp dir and extracts zip archive into it
fn open_zip_temp(zip_path: impl AsRef<std::path::Path>) -> Result<TempDir> {
    let temp_dir = tempdir().map_err(|e| anyhow!("Cannot create temp dir: {}", e))?;

    let file = File::open(zip_path).map_err(|e| anyhow!("Cannot open model .zip file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| anyhow!("Cannot get .zip model metadata: {}", e))?;

    archive
        .extract(temp_dir.path())
        .map_err(|e| anyhow!("Cannot unarchive .zip model: {}", e))?;

    Ok(temp_dir)
}
