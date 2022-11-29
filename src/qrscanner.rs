use std::time::Duration;

use color_eyre::eyre::{self, Context};
use color_eyre::Result;
use image::DynamicImage;
use indicatif::{ProgressBar, ProgressStyle};
use qrcode::render::unicode::Dense1x2;
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;

use crate::config::Config;

pub fn capture(config: &Config) -> Result<String> {
    let devices = v4l::context::enum_devices();
    if devices.is_empty() {
        eyre::bail!("No video devices found");
    }
    let device = {
        // pick the first one if there is only one, otherwise ask the user
        if devices.len() == 1 {
            &devices[0]
        } else {
            let devices_display = devices
                .iter()
                .map(|d| {
                    format!(
                        "{}: {} ({})",
                        d.index(),
                        d.name().unwrap_or_default(),
                        d.path().display()
                    )
                })
                .collect::<Vec<_>>();
            let selected_device = inquire::Select::new(
                "Select your camera device to use",
                devices_display.clone(),
            )
            .prompt()
            .context("Failed to select device")?;
            let i = devices_display
                .iter()
                .position(|d| d == &selected_device)
                .unwrap_or_default();
            &devices[i]
        }
    };
    let device = v4l::Device::new(device.index())?;
    let format = device.format()?;
    let mut stream = Stream::with_buffers(&device, Type::VideoCapture, 4)?;
    let preview = viuer::Config {
        restore_cursor: false,
        transparent: false,
        ..Default::default()
    };
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["☱", "☲", "☴"])
            .template("{spinner:.green} {msg}")?,
    );
    progress_bar.set_message("Scanning the QR Code");
    loop {
        let (buf, _) = stream.next()?;
        let img_buf = image::ImageBuffer::from_raw(
            format.width as u32,
            format.height as u32,
            buf.to_owned(),
        )
        .ok_or_else(|| eyre::eyre!("Failed to convert buffer to image"))?;
        let image = DynamicImage::ImageLuma8(img_buf);
        match try_decode(config, &image) {
            Ok(content) => {
                progress_bar.finish_and_clear();
                return Ok(content);
            }
            Err(_) => {
                if config.debug {
                    viuer::print(&image.fliph(), &preview)?;
                }
                progress_bar.tick();
            }
        };
    }
}

pub fn display_qr_code(config: &Config, content: &str) -> Result<()> {
    let qr_code = qrcode::QrCode::new(content)?;
    let image = qr_code.render::<Dense1x2>().build();
    println!("{image}");
    if config.debug {
        println!("QR Code Content: {}", content);
    }
    Ok(())
}

fn try_decode(config: &Config, image: &DynamicImage) -> Result<String> {
    let image = image.to_luma8();
    let mut img = rqrr::PreparedImage::prepare(image);
    let grids = img.detect_grids();

    if let Some(grid) = grids.first() {
        let (meta, content) = grid.decode()?;
        if config.debug {
            eprint!("\r                        \r");
            display_qr_code(config, &content)?;
            // Metadata
            println!();
            println!("Version: {}", meta.version.0);
            println!("Grid Size: {}", meta.version.to_size());
            println!("EC Level: {}", meta.ecc_level);
            println!("Mask: {}", meta.mask);
        }
        Ok(content)
    } else {
        std::thread::sleep(Duration::from_millis(20));
        eyre::bail!("failed to read")
    }
}
