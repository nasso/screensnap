use super::screengrab::{Rectangle, Screenshot, Window};
use custom_error::custom_error;
use glium::{
    self,
    backend::glutin::DisplayCreationError,
    glutin::{
        dpi::LogicalPosition, ContextBuilder, ElementState, Event, EventsLoop, KeyboardInput,
        ModifiersState, MouseButton, VirtualKeyCode, WindowBuilder, WindowEvent,
    },
    implement_vertex,
    index::{BufferCreationError as IboCreationError, IndexBuffer, PrimitiveType},
    program,
    program::{Program, ProgramChooserCreationError},
    texture::{RawImage2d, SrgbTexture2d, TextureCreationError},
    uniform,
    vertex::{BufferCreationError as VboCreationError, VertexBuffer},
    Blend, Display, DrawError, DrawParameters, Surface, SwapBuffersError,
};
use std::time::{Duration, Instant};

// custom error type
custom_error! { pub CropperError
    DisplayCreation{source: DisplayCreationError} = "cannot create display: {source:?}",
    Swap{source: SwapBuffersError} = "cannot swap buffers: {source:?}",
    TextureCreation{source: TextureCreationError} = "cannot create texture: {source:?}",
    VboCreation{source: VboCreationError} = "cannot create vbo: {source:?}",
    IndexBufferCreation{source: IboCreationError} = "cannot create index buffer: {source:?}",
    ProgramCreation{source: ProgramChooserCreationError} = "cannot create program: {source:?}",
    Draw{source: DrawError} = "error when drawing: {source:?}",
}

// vertex buffer type
#[derive(Debug, Copy, Clone)]
struct Vertex {
    pos: [f32; 2],
}

implement_vertex!(Vertex, pos);

// structure holding the programs we use
struct CropperPrograms {
    full_quad_tex: Program,
    sub_quad_tex: Program,
}

// structure holding everything else we'll need
struct Cropper<T> {
    snap: T,

    region: Option<Rectangle>,
    delta: Duration,

    snap_tex: SrgbTexture2d,
    vbo: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    programs: CropperPrograms,
}

// where we do the cool stuff
impl<T: Screenshot> Cropper<T> {
    fn new(snap: T, display: &Display) -> Result<Cropper<T>, CropperError> {
        // return struct
        Ok(Cropper {
            region: None,
            delta: Default::default(),

            // create screenshot texture
            snap_tex: SrgbTexture2d::new(
                display,
                RawImage2d::from_raw_rgb(snap.data().into(), snap.dimensions()),
            )?,

            snap,

            // create a fullscreen quad VBO
            vbo: VertexBuffer::new(
                display,
                &[
                    Vertex { pos: [0.0, 0.0] },
                    Vertex { pos: [1.0, 0.0] },
                    Vertex { pos: [0.0, 1.0] },
                    Vertex { pos: [1.0, 1.0] },
                ],
            )?,

            // indices for the VBO
            index_buffer: IndexBuffer::new(
                display,
                PrimitiveType::TriangleStrip,
                &[0 as u16, 1, 2, 3],
            )?,

            // all the programs we need
            programs: CropperPrograms {
                full_quad_tex: program!(display,
                    140 => {
                        vertex: include_str!("shaders/full_quad_tex/140.vs"),
                        fragment: include_str!("shaders/full_quad_tex/140.fs"),
                    }
                )?,

                sub_quad_tex: program!(display,
                    140 => {
                        vertex: include_str!("shaders/sub_quad_tex/140.vs"),
                        fragment: include_str!("shaders/sub_quad_tex/140.fs"),
                    }
                )?,
            },
        })
    }

    fn render(&mut self, frame: &mut glium::Frame) -> Result<(), CropperError> {
        let draw_params = DrawParameters {
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        // clear to black
        frame.clear_color(0.0, 0.0, 0.0, 1.0);

        // base pass
        let uniforms = uniform! {
            tex: &self.snap_tex,
            opacity: 0.5f32,
        };

        frame.draw(
            &self.vbo,
            &self.index_buffer,
            &self.programs.full_quad_tex,
            &uniforms,
            &draw_params,
        )?;

        // window bounds pass
        if let Some(region) = self.region {
            let uniforms = uniform! {
                tex: &self.snap_tex,
                opacity: 1.0f32,
                bounds: [
                    (region.x as f32) / (self.snap.dimensions().0 as f32),
                    1.0 - (region.y as f32) / (self.snap.dimensions().1 as f32),
                    (region.w as f32) / (self.snap.dimensions().0 as f32),
                    -(region.h as f32) / (self.snap.dimensions().1 as f32)
                ],
            };

            frame.draw(
                &self.vbo,
                &self.index_buffer,
                &self.programs.sub_quad_tex,
                &uniforms,
                &draw_params,
            )?;
        }

        Ok(())
    }
}

// "main" of the cropping part of the program
pub fn apply(snap: impl Screenshot) -> Result<(), CropperError> {
    let mut events_loop = EventsLoop::new();

    let display = Display::new(
        WindowBuilder::new()
            .with_title("Some test")
            .with_visibility(false)
            .with_always_on_top(true)
            .with_decorations(false)
            .with_resizable(false)
            .with_dimensions(snap.dimensions().into()),
        ContextBuilder::new().with_vsync(true),
        &events_loop,
    )?;

    display.gl_window().window().set_position((0, 0).into());
    display.gl_window().window().show();

    // create a cropper
    let mut cropper = Cropper::new(snap, &display)?;

    // becomes true whenever the window should close
    let mut closed = false;

    let mut now = Instant::now();

    // main loop
    while !closed {
        cropper.delta = now.elapsed();
        now = Instant::now();

        // create a frame
        let mut frame = display.draw();

        // store the result of the render
        let render_result = cropper.render(&mut frame);

        // finish the frame first
        frame.finish()?;

        // then we can check the result
        render_result?;

        // handle events
        events_loop.poll_events(|e| match e {
            // window events
            Event::WindowEvent { event, .. } => match event {
                // close requested
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    // set region to None do cancel
                    cropper.region = None;
                    closed = true
                }

                // cursor moved
                WindowEvent::CursorMoved {
                    position: LogicalPosition { x, y },
                    modifiers,
                    ..
                } => match modifiers {
                    ModifiersState { shift: true, .. } => {
                        cropper.region = cropper
                            .snap
                            .windows()
                            .into_iter()
                            .find(|w| w.content_bounds.contains(x as u32, y as u32))
                            .map(|w| w.content_bounds)
                    }
                    _ => cropper.region = None,
                },

                // mouse input
                WindowEvent::MouseInput {
                    button,
                    state: ElementState::Released,
                    ..
                } => match button {
                    MouseButton::Left => closed = true,
                    _ => (),
                },

                // other window events
                _ => (),
            },

            // other events
            _ => (),
        });
    }

    // copy to clipboard!
    if let Some(region) = cropper.region {
        cropper.snap.copy_to_clipboard(region);
    }

    Ok(())
}
