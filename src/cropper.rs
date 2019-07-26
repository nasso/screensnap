use super::{
    focuser,
    screengrab::{Rectangle, Screenshot},
};
use custom_error::custom_error;
use easer::functions::Easing;
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
    texture::{MipmapsOption, RawImage2d, SrgbTexture2d, TextureCreationError},
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

struct CroppingContext {
    started: Instant,
    delta: Duration,

    snap: Screenshot,
    snap_tex: SrgbTexture2d,

    region: Option<Rectangle<f64>>,
    animated_region: Option<Rectangle<f64>>,
    region_appear_time: Option<Instant>,
}

// structure holding everything else we'll need
pub struct Cropper {
    events_loop: EventsLoop,
    display: Display,

    vbo: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    programs: CropperPrograms,
}

// where we do the cool stuff
impl Cropper {
    pub fn new() -> Result<Cropper, CropperError> {
        let events_loop = EventsLoop::new();

        let display = Display::new(
            WindowBuilder::new()
                .with_title("Screenshot")
                .with_visibility(false)
                .with_always_on_top(true)
                .with_decorations(false)
                .with_resizable(false),
            ContextBuilder::new().with_vsync(true),
            &events_loop,
        )?;

        Ok(Cropper {
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

                sub_quad_tex: program!(&display,
                    140 => {
                        vertex: include_str!("shaders/sub_quad_tex/140.vs"),
                        fragment: include_str!("shaders/sub_quad_tex/140.fs"),
                    }
                )?,
            },

            events_loop,
            display,
        })
    }

    pub fn apply(&mut self, snap: Screenshot) -> Result<bool, CropperError> {
        self.display
            .gl_window()
            .window()
            .set_max_dimensions(Some((snap.bounds.w as u32, snap.bounds.h as u32).into()));
        self.display
            .gl_window()
            .window()
            .set_min_dimensions(Some((snap.bounds.w as u32, snap.bounds.h as u32).into()));
        self.display
            .gl_window()
            .window()
            .set_position((snap.bounds.x, snap.bounds.y).into());
        self.display.gl_window().window().show();

        // glutin doesn't let us bring the window to the foreground.
        // well, glutin, im doing it anyway.
        focuser::focus_current_window();

        let mut context = CroppingContext {
            started: Instant::now(),
            delta: Default::default(),

            region: None,
            animated_region: None,
            region_appear_time: None,

            snap_tex: SrgbTexture2d::with_mipmaps(
                &self.display,
                RawImage2d::from_raw_rgb(
                    snap.data.clone(),
                    (snap.bounds.w as u32, snap.bounds.h as u32),
                ),
                MipmapsOption::NoMipmap,
            )?,
            snap: snap,
        };

        // becomes true whenever the window should close
        let mut closed = false;

        // the value this function returns
        let mut should_quit = false;

        // where the left mouse button was pressed
        let mut left_press: Option<(f64, f64)> = None;

        // tracks the position of the cursor
        let mut cursor_pos = (0.0, 0.0);

        // right now
        let mut now = Instant::now();

        // empty the event queue
        self.events_loop.poll_events(|_| ());

        // main loop
        while !closed {
            context.delta = now.elapsed();
            now = Instant::now();

            // create a frame
            let mut frame = self.display.draw();

            // store the result of the render
            let render_result = self.render(&mut frame, &mut context);

            // finish the frame first
            frame.finish()?;

            // then we can check the result
            render_result?;

            // handle events
            self.events_loop.poll_events(|e| match e {
                // window events
                Event::WindowEvent { event, .. } => match event {
                    // kill process
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Q),
                                state: ElementState::Pressed,
                                modifiers:
                                    ModifiersState {
                                        ctrl: true,
                                        shift: true,
                                        ..
                                    },
                                ..
                            },
                        ..
                    } => {
                        should_quit = true;
                        closed = true
                    }

                    // cancel screenshot
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        // set region to None do cancel
                        context.region = None;
                        closed = true
                    }

                    // cursor moved
                    WindowEvent::CursorMoved {
                        position: LogicalPosition { x, y },
                        modifiers,
                        ..
                    } => {
                        cursor_pos = (x, y);

                        if let Some((px, py)) = left_press {
                            context.region = Some(Rectangle {
                                x: px.min(x),
                                y: py.min(y),
                                w: (px - x).abs(),
                                h: (py - y).abs(),
                            });

                            // disable animation
                            context.animated_region = context.region;
                        } else {
                            match modifiers {
                                ModifiersState { shift: true, .. } => {
                                    context.region = context
                                        .snap
                                        .windows
                                        .iter()
                                        .find(|w| w.bounds.contains(x as i32, y as i32))
                                        .map(|w| Rectangle {
                                            x: w.bounds.x as f64,
                                            y: w.bounds.y as f64,
                                            w: w.bounds.w as f64,
                                            h: w.bounds.h as f64,
                                        })
                                }
                                _ => context.region = None,
                            }
                        }
                    }

                    // mouse input
                    WindowEvent::MouseInput { button, state, .. } => match (button, state) {
                        (MouseButton::Left, ElementState::Released) => closed = true,
                        (MouseButton::Left, ElementState::Pressed) => left_press = Some(cursor_pos),
                        _ => (),
                    },

                    // other window events
                    _ => (),
                },

                // other events
                _ => (),
            });
        }

        self.display.gl_window().window().hide();

        // copy to clipboard!
        if let Some(region) = context.region {
            context.snap.copy_to_clipboard(Rectangle {
                x: region.x as u32,
                y: region.y as u32,
                w: region.w as u32,
                h: region.h as u32,
            });
        }

        Ok(should_quit)
    }

    fn render(
        &mut self,
        frame: &mut glium::Frame,
        ctx: &mut CroppingContext,
    ) -> Result<(), CropperError> {
        if let (Some(areg), Some(reg)) = (ctx.animated_region, ctx.region) {
            let delta_s = ctx.delta.as_millis() as f64 / 1000.0;

            ctx.animated_region = Some(Rectangle {
                x: areg.x + (reg.x - areg.x) * delta_s * 20.0,
                y: areg.y + (reg.y - areg.y) * delta_s * 20.0,
                w: areg.w + (reg.w - areg.w) * delta_s * 20.0,
                h: areg.h + (reg.h - areg.h) * delta_s * 20.0,
            });
        } else {
            ctx.animated_region = ctx.region;
        }

        let draw_params = DrawParameters {
            blend: Blend::alpha_blending(),
            ..Default::default()
        };

        // clear to black
        frame.clear_color(0.0, 0.0, 0.0, 1.0);

        // base pass
        let uniforms = uniform! {
            tex: &ctx.snap_tex,
            opacity: easer::functions::Cubic::ease_out(
                ctx.started.elapsed().as_millis().min(200) as f32,
                1.0f32,
                -0.5f32,
                200.0f32
            ),
        };

        frame.draw(
            &self.vbo,
            &self.index_buffer,
            &self.programs.full_quad_tex,
            &uniforms,
            &draw_params,
        )?;

        // active region pass
        if let Some(areg) = ctx.animated_region {
            if let None = ctx.region_appear_time {
                ctx.region_appear_time = Some(Instant::now());
            }

            let uniforms = uniform! {
                tex: &ctx.snap_tex,
                opacity: easer::functions::Cubic::ease_out(
                    ctx.region_appear_time.unwrap().elapsed().as_millis().min(200) as f32,
                    0.5f32,
                    0.5f32,
                    200.0f32
                ),
                bounds: [
                    (areg.x as f32) / (ctx.snap.bounds.w as f32),
                    1.0 - (areg.y as f32) / (ctx.snap.bounds.h as f32),
                    (areg.w as f32) / (ctx.snap.bounds.w as f32),
                    -(areg.h as f32) / (ctx.snap.bounds.h as f32)
                ],
            };

            frame.draw(
                &self.vbo,
                &self.index_buffer,
                &self.programs.sub_quad_tex,
                &uniforms,
                &draw_params,
            )?;
        } else {
            ctx.region_appear_time = None;
        }

        Ok(())
    }
}
