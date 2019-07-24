use custom_error::custom_error;

mod cropper;
mod hotkey;
mod screengrab;

use cropper::Cropper;

custom_error! { ScreenshotError
    Cropping{source: cropper::CropperError} = "error while cropping: {source:?}",
}

fn main() -> Result<(), ScreenshotError> {
    // create the cropper
    let mut cropper = Cropper::new()?;

    hotkey::register(true, || {
        // get screenshot
        match cropper.apply(screengrab::snap(screengrab::Bounds::FullScreen)) {
            Err(e) => eprintln!("{:?}", e),
            _ => (),
        }

        true
    });

    Ok(())
}
