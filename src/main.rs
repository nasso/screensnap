use custom_error::custom_error;

mod cropper;
mod screengrab;

custom_error! { ScreenshotError
    Cropping{source: cropper::CropperError} = "error while cropping: {source:?}",
}

fn main() -> Result<(), ScreenshotError> {
    // get screenshot
    let snap = screengrab::snap(screengrab::Bounds::FullScreen);

    // crop the screenshot
    cropper::apply(snap)?;

    Ok(())
}
