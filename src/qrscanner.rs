use anyhow::Result;
use fast_qr::QRBuilder;
use image::DynamicImage;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::CameraIndex;
use nokhwa::utils::RequestedFormat;
use nokhwa::utils::RequestedFormatType;
use nokhwa::Camera;
use std::time::Duration;

static PROGRESS: &[&str] = &["   ", ".  ", ".. ", "..."];

pub fn capture() -> Result<()> {
    let backend = nokhwa::native_api_backend().ok_or_else(|| {
        anyhow::anyhow!("No backend found, please check your installation")
    })?;
    let devices = nokhwa::query(backend)?;
    let first_device = devices
        .first()
        .ok_or_else(|| anyhow::anyhow!("No camera found"))?;
    let requested = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::HighestResolutionAbs,
    );
    // let format = RequestedFormat::new_from(640, 480, FrameFormat::MJPEG, 30);
    let mut camera = Camera::new(first_device.index().clone(), requested)?;
    let mut spinner = 0;

    let preview = viuer::Config {
        x: 0,
        y: 0,
        restore_cursor: false,
        transparent: false,
        absolute_offset: true,
        ..Default::default()
    };

    camera.open_stream()?;

    loop {
        let frame = camera.frame()?;
        let image = DynamicImage::ImageRgb8(frame.decode_image::<RgbFormat>()?);
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
        let qrcode = QRBuilder::new(content.clone())
            .build()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        qrcode.print();
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
        std::thread::sleep(Duration::from_millis(200));
        anyhow::bail!("failed to read")
    };

    Ok(())
}
