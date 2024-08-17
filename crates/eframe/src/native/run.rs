use std::{cell::RefCell, time::Instant};

use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

use ahash::HashMap;

use super::winit_integration::{UserEvent, WinitApp};
use crate::{
    epi,
    native::{event_loop_context, winit_integration::EventResult},
    Result,
};

// Initialize the event loop with custom options
fn create_event_loop(native_options: &mut epi::NativeOptions) -> Result<EventLoop<UserEvent>> {
    crate::profile_function!();
    let mut builder = EventLoop::with_user_event();

    if let Some(hook) = std::mem::take(&mut native_options.event_loop_builder) {
        hook(&mut builder);
    }

    crate::profile_scope!("EventLoopBuilder::build");
    Ok(builder.build()?)
}

// Manage a thread-local event loop for reusability
fn with_event_loop<R>(
    mut native_options: epi::NativeOptions,
    f: impl FnOnce(&mut EventLoop<UserEvent>, epi::NativeOptions) -> R,
) -> Result<R> {
    thread_local!(static EVENT_LOOP: RefCell<Option<EventLoop<UserEvent>>> = RefCell::new(None));

    EVENT_LOOP.with(|event_loop| {
        let mut event_loop_lock = event_loop.borrow_mut();
        let event_loop = if let Some(event_loop) = &mut *event_loop_lock {
            event_loop
        } else {
            event_loop_lock.insert(create_event_loop(&mut native_options)?)
        };
        Ok(f(event_loop, native_options))
    })
}

// Wrapper for WinitApp to implement ApplicationHandler
struct WinitAppWrapper<T: WinitApp> {
    windows_next_repaint_times: HashMap<WindowId, Instant>,
    winit_app: T,
    return_result: Result<(), crate::Error>,
    run_and_return: bool,
}

impl<T: WinitApp> WinitAppWrapper<T> {
    fn new(winit_app: T, run_and_return: bool) -> Self {
        Self {
            windows_next_repaint_times: HashMap::default(),
            winit_app,
            return_result: Ok(()),
            run_and_return,
        }
    }

    // Process event results and manage application flow
    fn handle_event_result(
        &mut self,
        event_loop: &ActiveEventLoop,
        event_result: Result<EventResult>,
    ) {
        let mut exit = false;

        log::trace!("event_result: {event_result:?}");

        let combined_result = event_result.and_then(|event_result| {
            match event_result {
                EventResult::Wait => {
                    event_loop.set_control_flow(ControlFlow::Wait);
                    Ok(event_result)
                }
                EventResult::RepaintNow(window_id) => {
                    log::trace!("RepaintNow of {window_id:?}",);

                    if cfg!(target_os = "windows") {
                        // Windows-specific handling to prevent flickering
                        self.windows_next_repaint_times.remove(&window_id);
                        self.winit_app.run_ui_and_paint(event_loop, window_id)
                    } else {
                        // Non-Windows handling
                        self.windows_next_repaint_times
                            .insert(window_id, Instant::now());
                        Ok(event_result)
                    }
                }
                EventResult::RepaintNext(window_id) => {
                    log::trace!("RepaintNext of {window_id:?}",);
                    self.windows_next_repaint_times
                        .insert(window_id, Instant::now());
                    Ok(event_result)
                }
                EventResult::RepaintAt(window_id, repaint_time) => {
                    self.windows_next_repaint_times.insert(
                        window_id,
                        self.windows_next_repaint_times
                            .get(&window_id)
                            .map_or(repaint_time, |last| (*last).min(repaint_time)),
                    );
                    Ok(event_result)
                }
                EventResult::Exit => {
                    exit = true;
                    Ok(event_result)
                }
            }
        });

        if let Err(err) = combined_result {
            log::error!("Exiting due to error: {err}");
            exit = true;
            self.return_result = Err(err);
        };

        if exit {
            if self.run_and_return {
                log::debug!("Requesting event loop exit...");
                event_loop.exit();
            } else {
                log::debug!("Quitting - saving application state...");
                self.winit_app.save_and_destroy();

                log::debug!("Exiting with return code 0");

                #[allow(clippy::exit)]
                std::process::exit(0);
            }
        }

        self.check_redraw_requests(event_loop);
    }

    // Manage redraw requests for windows
    fn check_redraw_requests(&mut self, event_loop: &ActiveEventLoop) {
        let mut next_repaint_time = self.windows_next_repaint_times.values().min().copied();

        self.windows_next_repaint_times
            .retain(|window_id, repaint_time| {
                if Instant::now() < *repaint_time {
                    return true; // Not yet time to repaint
                };

                next_repaint_time = None;
                event_loop.set_control_flow(ControlFlow::Poll);

                if let Some(window) = self.winit_app.window(*window_id) {
                    log::trace!("Requesting redraw for {window_id:?}");
                    let is_minimized = window.is_minimized().unwrap_or(false);
                    if is_minimized {
                        false
                    } else {
                        window.request_redraw();
                        true
                    }
                } else {
                    log::trace!("Window not found for {window_id:?}");
                    false
                }
            });

        if let Some(next_repaint_time) = next_repaint_time {
            // iOS-specific handling
            #[cfg(target_os = "ios")]
            winit_app
                .window_id_from_viewport_id(egui::ViewportId::ROOT)
                .map(|window_id| {
                    winit_app
                        .window(window_id)
                        .map(|window| window.request_redraw())
                });

            event_loop.set_control_flow(ControlFlow::WaitUntil(next_repaint_time));
        };
    }
}

impl<T: WinitApp> ApplicationHandler<UserEvent> for WinitAppWrapper<T> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if let winit::event::StartCause::ResumeTimeReached { .. } = cause {
            log::trace!("Woke up to check next_repaint_time");
        }

        self.check_redraw_requests(event_loop);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        crate::profile_function!("Event::Resumed");

        // Ensure context is dropped after function returns
        event_loop_context::with_event_loop_context(event_loop, move || {
            let event_result = self.winit_app.resumed(event_loop);
            self.handle_event_result(event_loop, event_result);
        });
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        crate::profile_function!(match &event {
            UserEvent::RequestRepaint { .. } => "UserEvent::RequestRepaint",
            #[cfg(feature = "accesskit")]
            UserEvent::AccessKitActionRequest(_) => "UserEvent::AccessKitActionRequest",
        });

        event_loop_context::with_event_loop_context(event_loop, move || {
            let event_result = match event {
                UserEvent::RequestRepaint {
                    when,
                    frame_nr,
                    viewport_id,
                } => {
                    let current_frame_nr = self.winit_app.frame_nr(viewport_id);
                    if current_frame_nr == frame_nr || current_frame_nr == frame_nr + 1 {
                        log::trace!("UserEvent::RequestRepaint scheduling repaint at {when:?}");
                        if let Some(window_id) =
                            self.winit_app.window_id_from_viewport_id(viewport_id)
                        {
                            Ok(EventResult::RepaintAt(window_id, when))
                        } else {
                            Ok(EventResult::Wait)
                        }
                    } else {
                        log::trace!("Received outdated UserEvent::RequestRepaint");
                        Ok(EventResult::Wait) // Outdated request - repaint already occurred
                    }
                }
                #[cfg(feature = "accesskit")]
                UserEvent::AccessKitActionRequest(request) => {
                    self.winit_app.on_accesskit_event(request)
                }
            };
            self.handle_event_result(event_loop, event_result);
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        crate::profile_function!(egui_winit::short_window_event_description(&event));

        // Ensure context is dropped after function returns
        event_loop_context::with_event_loop_context(event_loop, move || {
            let event_result = match event {
                winit::event::WindowEvent::RedrawRequested => {
                    self.windows_next_repaint_times.remove(&window_id);
                    self.winit_app.run_ui_and_paint(event_loop, window_id)
                }
                _ => self.winit_app.window_event(event_loop, window_id, event),
            };

            self.handle_event_result(event_loop, event_result);
        });
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        crate::profile_function!(egui_winit::short_device_event_description(&event));

        // Ensure context is dropped after function returns
        event_loop_context::with_event_loop_context(event_loop, move || {
            let event_result = self.winit_app.device_event(event_loop, device_id, event);
            self.handle_event_result(event_loop, event_result);
        });
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        crate::profile_function!("Event::Suspended");

        event_loop_context::with_event_loop_context(event_loop, move || {
            let event_result = self.winit_app.suspended(event_loop);
            self.handle_event_result(event_loop, event_result);
        });
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        // Save state on Mac Cmd-Q as run_app_on_demand doesn't return
        log::debug!("Received Event::LoopExiting - saving application state...");
        event_loop_context::with_event_loop_context(event_loop, move || {
            self.winit_app.save_and_destroy();
        });
    }
}

#[cfg(not(target_os = "ios"))]
fn run_and_return(event_loop: &mut EventLoop<UserEvent>, winit_app: impl WinitApp) -> Result {
    use winit::platform::run_on_demand::EventLoopExtRunOnDemand;

    log::trace!("Entering winit event loop (run_app_on_demand)...");

    let mut app = WinitAppWrapper::new(winit_app, true);
    event_loop.run_app_on_demand(&mut app)?;
    log::debug!("eframe window closed");
    app.return_result
}

fn run_and_exit(event_loop: EventLoop<UserEvent>, winit_app: impl WinitApp + 'static) -> Result {
    log::trace!("Entering winit event loop (run_app)...");

    let mut app = WinitAppWrapper::new(winit_app, false);
    event_loop.run_app(&mut app)?;

    log::debug!("winit event loop unexpectedly returned");
    Ok(())
}

// Glow-specific implementation
#[cfg(feature = "glow")]
pub fn run_glow(
    app_name: &str,
    mut native_options: epi::NativeOptions,
    app_creator: epi::AppCreator,
) -> Result {
    #![allow(clippy::needless_return_with_question_mark)] // False positive

    use super::glow_integration::GlowWinitApp;

    #[cfg(not(target_os = "ios"))]
    if native_options.run_and_return {
        return with_event_loop(native_options, |event_loop, native_options| {
            let glow_eframe = GlowWinitApp::new(event_loop, app_name, native_options, app_creator);
            run_and_return(event_loop, glow_eframe)
        })?;
    }

    let event_loop = create_event_loop(&mut native_options)?;
    let glow_eframe = GlowWinitApp::new(&event_loop, app_name, native_options, app_creator);
    run_and_exit(event_loop, glow_eframe)
}

// WGPU-specific implementation
#[cfg(feature = "wgpu")]
pub fn run_wgpu(
    app_name: &str,
    mut native_options: epi::NativeOptions,
    app_creator: epi::AppCreator,
) -> Result {
    #![allow(clippy::needless_return_with_question_mark)] // False positive

    use super::wgpu_integration::WgpuWinitApp;

    #[cfg(not(target_os = "ios"))]
    if native_options.run_and_return {
        return with_event_loop(native_options, |event_loop, native_options| {
            let wgpu_eframe = WgpuWinitApp::new(event_loop, app_name, native_options, app_creator);
            run_and_return(event_loop, wgpu_eframe)
        })?;
    }

    let event_loop = create_event_loop(&mut native_options)?;
    let wgpu_eframe = WgpuWinitApp::new(&event_loop, app_name, native_options, app_creator);
    run_and_exit(event_loop, wgpu_eframe)
}