use std::time::Duration;

use windows::Win32::Foundation::HWND;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::WindowId,
};

use crate::{context::Context, dwd::Cache, paint::Pipeline, platform, window::DxWindow};

pub enum AppEvent {
    Refresh,
    Repaint,
}

pub struct Application {
    window: Option<DxWindow>,
    context: Context,
    pipl: Pipeline,
    clicked: bool,
    last_pos: PhysicalPosition<f64>,
    window_pos: Option<PhysicalPosition<i32>>,
    as_background: bool,
}

impl ApplicationHandler<AppEvent> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        assert!(self.window.is_none());
        let window = DxWindow::new(
            event_loop,
            self.context.layout_ctx.image_size,
            !self.as_background,
        )
        .expect("Failed to create window context");
        let hwnd = HWND(u64::from(window.window.id()) as *mut _);
        self.window = Some(window);
        if self.as_background {
            platform::windows::setup_for_hwnd(hwnd).unwrap();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = self.window.as_mut().unwrap();

        match event {
            WindowEvent::RedrawRequested => {
                window.render(&mut self.pipl, &self.context.layout_ctx);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.clicked = state == ElementState::Pressed;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.clicked {
                    let diff = PhysicalPosition::new(
                        position.x - self.last_pos.x,
                        position.y - self.last_pos.y,
                    );
                    let last = self
                        .window_pos
                        .unwrap_or_else(|| window.window.outer_position().unwrap());
                    let next =
                        PhysicalPosition::new(last.x + diff.x as i32, last.y + diff.y as i32);
                    self.window_pos = Some(next);
                    window.window.set_outer_position(next);
                } else {
                    self.last_pos = position;
                }
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.physical_key == PhysicalKey::Code(winit::keyboard::KeyCode::KeyQ) =>
            {
                event_loop.exit();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        let window = self.window.as_mut().unwrap();
        match event {
            AppEvent::Refresh => {
                self.context.replan();
                self.context.relayout(&mut self.pipl);
                window.render(&mut self.pipl, &self.context.layout_ctx);
            }
            AppEvent::Repaint => {
                window.render(&mut self.pipl, &self.context.layout_ctx);
            }
        }
    }
}

impl Application {
    pub fn new(context: Context, pipl: Pipeline, as_background: bool) -> Self {
        Self {
            window: None,
            context,
            pipl,
            clicked: false,
            last_pos: PhysicalPosition::new(0.0, 0.0),
            window_pos: None,
            as_background,
        }
    }

    pub fn run(&mut self) {
        const REPAINT_TICKS: u8 = 8; // -> every 4min

        let event_loop = EventLoop::<AppEvent>::with_user_event().build().unwrap();
        let proxy = event_loop.create_proxy();
        let cache = self.context.cache.clone();

        // every so often we need to repaint the current time
        let mut pending_ticks = REPAINT_TICKS;
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(30));
                if Cache::refetch(&cache).unwrap_or_default() {
                    let _ = proxy.send_event(AppEvent::Refresh);
                    pending_ticks = REPAINT_TICKS;
                }
                if pending_ticks == 0 {
                    let _ = proxy.send_event(AppEvent::Repaint);
                    pending_ticks = REPAINT_TICKS;
                } else {
                    pending_ticks -= 1;
                }
            }
        });
        event_loop.run_app(self).expect("Failed to run event loop");
    }
}
