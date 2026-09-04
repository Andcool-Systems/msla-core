use anyhow::Result;
use image::GenericImageView;
use memmap2::MmapOptions;
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::Arc,
};
use tracing::debug;

/// Printer LCD display controller
#[derive(Clone)]
pub struct LCDController {
    fb: Arc<File>,
}

impl LCDController {
    /// Create new LCD controller
    pub fn new() -> Result<Self> {
        let fb = Arc::new(OpenOptions::new().read(true).write(true).open("/dev/fb0")?);

        Ok(Self { fb })
    }

    /// Send image into framebuffer
    pub fn show_image(&self, path: PathBuf) -> Result<()> {
        let img = image::open(path.clone())?;
        let (width, height) = img.dimensions();

        // Calculate the buffer size in bytes (4 bytes per pixel for 32-bit ARGB/XRGB)
        let fb_size = (width * height * 4) as usize;

        let mut mmap = unsafe { MmapOptions::new().len(fb_size).map_mut(&*self.fb)? };
        let mut fb_index = 0;

        for (_x, _y, pixel) in img.pixels() {
            let channels = pixel.0;
            let luma = channels[0];

            let color_val = u32::from_be_bytes([255, luma, luma, luma]);

            let byte_offset = fb_index * 4;
            if byte_offset + 3 < mmap.len() {
                mmap[byte_offset..byte_offset + 4].copy_from_slice(&color_val.to_ne_bytes());
            }

            fb_index += 1;
        }

        mmap.flush()?;

        debug!("Displayed image: {:?}", path);
        Ok(())
    }
}
