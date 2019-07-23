use super::screengrab::Screenshot;
use custom_error::custom_error;
use glium::{
    self,
    backend::glutin::DisplayCreationError,
    glutin::{
        ContextBuilder, Event, EventsLoop, KeyboardInput, VirtualKeyCode, WindowBuilder,
        WindowEvent,
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
    sub_quad_color: Program,
}

// structure holding everything else we'll need
struct Cropper<T> {
    snap: T,

    events_loop: EventsLoop,
    display: Display,
    snap_tex: SrgbTexture2d,
    vbo: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    programs: CropperPrograms,
}

// where we do the cool stuff
impl<T: Screenshot> Cropper<T> {
    fn new(snap: T) -> Result<Cropper<T>, CropperError> {
        let events_loop = EventsLoop::new();

        let display = Display::new(
            WindowBuilder::new()
                .with_title("Some test")
                .with_visibility(false)
                .with_always_on_top(true)
                .with_decorations(false)
                .with_resizable(false)
                .with_dimensions(snap.dimensions().into()),
            ContextBuilder::new(),
            &events_loop,
        )?;

        display.gl_window().window().set_position((0, 0).into());
        display.gl_window().window().show();

        // return struct
        Ok(Cropper {
            // the event loop
            events_loop,

            // create screenshot texture
            snap_tex: SrgbTexture2d::new(
                &display,
                RawImage2d::from_raw_rgb(snap.data().into(), snap.dimensions()),
            )?,

            snap,

            // create a fullscreen quad VBO
            vbo: VertexBuffer::new(
                &display,
                &[
                    Vertex { pos: [0.0, 0.0] },
                    Vertex { pos: [1.0, 0.0] },
                    Vertex { pos: [0.0, 1.0] },
                    Vertex { pos: [1.0, 1.0] },
                ],
            )?,

            // indices for the VBO
            index_buffer: IndexBuffer::new(
                &display,
                PrimitiveType::TriangleStrip,
                &[0 as u16, 1, 2, 3],
            )?,

            // all the programs we need
            programs: CropperPrograms {
                full_quad_tex: program!(&display,
                    140 => {
                        vertex: include_str!("shaders/full_quad_tex/140.vs"),
                        fragment: include_str!("shaders/full_quad_tex/140.fs"),
                    }
                )?,

                sub_quad_color: program!(&display,
                    140 => {
                        vertex: include_str!("shaders/sub_quad_color/140.vs"),
                        fragment: include_str!("shaders/sub_quad_color/140.fs"),
                    }
                )?,
            },

            // this must be given last so that it doesn't take ownership before
            display,
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

        // windows pass
        for window in self.snap.windows() {
            let uniforms = uniform! {
                bounds: [
                    (window.content_bounds.x as f32) / (self.snap.dimensions().0 as f32),
                    1.0 - (window.content_bounds.y as f32) / (self.snap.dimensions().1 as f32),
                    (window.content_bounds.w as f32) / (self.snap.dimensions().0 as f32),
                    -(window.content_bounds.h as f32) / (self.snap.dimensions().1 as f32)
                ],
                color: [1.0f32, 1.0, 1.0, 0.1],
            };

            frame.draw(
                &self.vbo,
                &self.index_buffer,
                &self.programs.sub_quad_color,
                &uniforms,
                &draw_params,
            )?;
        }

        Ok(())
    }
}

// "main" of the cropping part of the program
pub fn apply(snap: impl Screenshot) -> Result<Option<(u64, u64, u64, u64)>, CropperError> {
    // create a cropper
    let mut cropper = Cropper::new(snap)?;

    // becomes true whenever the window should close
    let mut closed = false;

    // main loop
    while !closed {
        // create a frame
        let mut frame = cropper.display.draw();

        // store the result of the render
        let render_result = cropper.render(&mut frame);

        // finish the frame first
        frame.finish()?;

        // then we can check the result
        render_result?;

        // handle events
        cropper.events_loop.poll_events(|e| match e {
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
                } => closed = true,

                // other window events
                _ => (),
            },

            // other events
            _ => (),
        });
    }

    Ok(None)
}
