use gamedon_graphics::{GraphicsState, SurfaceError};
use std::sync::Arc;
use std::sync::OnceLock;
use winit::event_loop;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window,
};

static RED_DATA: OnceLock<[u8; 1024]> = OnceLock::new();
static GREEN_DATA: OnceLock<[u8; 1024]> = OnceLock::new();

/// A graphics app.
struct App {
    /// The graphics state.
    state: Option<GraphicsState>,
    /// Counter.
    counter: usize,
    /// Frame timer.
    timer: std::time::Instant,
}

impl App {
    /// Initialise app.
    fn new() -> Self {
        Self {
            state: None,
            counter: 0,
            timer: std::time::Instant::now(),
        }
    }
}

impl ApplicationHandler<GraphicsState> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = window::Window::default_attributes();

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.state = Some(pollster::block_on(GraphicsState::new(window)).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => state.resize(new_size.width, new_size.height),
            WindowEvent::RedrawRequested => {
                let target_duration = std::time::Duration::from_secs_f64(60.0f64.recip());
                let frame_duration = self.timer.elapsed();
                if frame_duration < target_duration {
                    state.window.request_redraw();
                    return;
                }

                self.timer = std::time::Instant::now();
                tracing::trace!("Frame time = {}ms", frame_duration.as_millis());

                self.counter += 1;
                let data = if (self.counter / 600).is_multiple_of(2) {
                    RED_DATA.get_or_init(|| {
                        let mut data = [0u8; 1024];
                        for row in 0..16 {
                            for col in 0..16 {
                                let index = 4 * (16 * row + col);
                                data[index] = 255;
                                data[index + 1] = 0;
                                data[index + 2] = 0;
                                data[index + 3] = 255;
                            }
                        }
                        data
                    })
                } else {
                    GREEN_DATA.get_or_init(|| {
                        let mut data = [0u8; 1024];
                        for row in 0..16 {
                            for col in 0..16 {
                                let index = 4 * (16 * row + col);
                                data[index] = 0;
                                data[index + 1] = 255;
                                data[index + 2] = 0;
                                data[index + 3] = 255;
                            }
                        }
                        data
                    })
                };
                state.update_framebuffer(data, 16, 16);
                state.update_text(&format!("Frame time: {}ms", frame_duration.as_micros()));
                match state.render() {
                    Ok(()) => {}
                    Err(SurfaceError::SurfaceNeedsRecreation) => {
                        let size = state.get_inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(SurfaceError::Unrecoverable) => {
                        tracing::error!("Surface errored out!");
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: GraphicsState) {
        self.state = Some(event);
    }
}

/// Run a graphics app.
pub fn run() -> color_eyre::Result<()> {
    let event_loop = EventLoop::with_user_event().build()?;
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
