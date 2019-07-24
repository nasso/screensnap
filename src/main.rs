use custom_error::custom_error;

mod cropper;
mod focuser;
mod hotkey;
mod msgbox;
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
        match cropper.apply(screengrab::snap()) {
            Err(e) => {
                msgbox::error(&format!("{:?}", e));
                true
            }
            Ok(should_quit) => should_quit,
        }
    });

    Ok(())
}
