use anyhow::Context;
use anyhow::Result;
use image::DynamicImage;
use qrcode::render::unicode::Dense1x2;
use std::time::Duration;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;

static PROGRESS: &[&str] = &["   ", ".  ", ".. ", "..."];

pub fn capture() -> Result<()> {
    let devices = v4l::context::enum_devices();
    if devices.is_empty() {
        anyhow::bail!("No video devices found");
    }
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
    let device = &devices[i];
    let device = v4l::Device::new(device.index())?;
    let format = device.format()?;
    let mut stream =
        v4l::io::mmap::Stream::with_buffers(&device, Type::VideoCapture, 4)?;
    let mut spinner = 0;

    let preview = viuer::Config {
        restore_cursor: false,
        transparent: false,
        ..Default::default()
    };

    loop {
        let (buf, _) = stream.next()?;
        let img_buf = image::ImageBuffer::from_raw(
            format.width as u32,
            format.height as u32,
            buf.to_owned(),
        )
        .ok_or_else(|| anyhow::anyhow!("Failed to convert buffer to image"))?;
        let image = DynamicImage::ImageLuma8(img_buf);
        if print_image(&image).is_err() {
            viuer::print(&image.fliph(), &preview)?;
            eprint!("\rScanning via camera{}", PROGRESS[spinner]);
            spinner = (spinner + 1) % 4;
        } else {
            break;
        }
    }

    Ok(())
}

fn print_image(image: &DynamicImage) -> Result<()> {
    let image = image.to_luma8();
    let mut img = rqrr::PreparedImage::prepare(image);
    let grids = img.detect_grids();

    if let Some(grid) = grids.first() {
        let (meta, content) = grid.decode()?;
        eprint!("\r                        \r");
        println!();
        let qrcode = qrcode::QrCode::new(content.clone())?;
        let image = qrcode.render::<Dense1x2>().build();
        println!("{}", image);
        // Metadata
        println!();
        println!("Version: {}", meta.version.0);
        println!("Grid Size: {}", meta.version.to_size());
        println!("EC Level: {}", meta.ecc_level);
        println!("Mask: {}", meta.mask);

        // Content
        println!();
        println!("{}", content);
    } else {
        std::thread::sleep(Duration::from_millis(50));
        anyhow::bail!("failed to read")
    };

    Ok(())
}
