use std::cell::Cell;
use winit::event_loop::ActiveEventLoop;

// Thread-local storage for keeping track of the current event loop.
// This ensures that only one event loop is active at a time.
thread_local! {
    static CURRENT_EVENT_LOOP: Cell<Option<*const ActiveEventLoop>> = Cell::new(None);
}

/// Guard struct to manage the lifecycle of the event loop.
///
/// This guard ensures that the event loop pointer is set when a new event loop is introduced
/// and clears it when the guard is dropped, preventing multiple event loops from being set at the same time.
struct EventLoopGuard;

impl EventLoopGuard {
    /// Creates a new `EventLoopGuard` and sets the current event loop.
    ///
    /// # Arguments
    /// * `event_loop` - A reference to the active event loop to be set.
    ///
    /// # Panics
    /// Panics if an event loop is already set, enforcing that only one event loop can be active at a time.
    fn new(event_loop: &ActiveEventLoop) -> Self {
        CURRENT_EVENT_LOOP.with(|cell| {
            assert!(
                cell.get().is_none(),
                "Attempted to set a new event loop while one is already set"
            );
            cell.set(Some(event_loop as *const ActiveEventLoop));
        });
        Self
    }
}

impl Drop for EventLoopGuard {
    /// Clears the event loop reference when the `EventLoopGuard` is dropped.
    ///
    /// This ensures that the thread-local storage is cleaned up and no stale references to the event loop remain.
    fn drop(&mut self) {
        CURRENT_EVENT_LOOP.set(None);
    }
}

/// Helper function to safely access the current event loop.
///
/// # Arguments
/// * `f` - A closure that takes a reference to the active event loop and returns a value of type `R`.
///
/// # Returns
/// * `Option<R>` - The result of applying the closure to the event loop, or `None` if no event loop is set.
///
/// # Safety
/// - The pointer is guaranteed to be valid when it is `Some` because the `EventLoopGuard` that created it
///   lives at least as long as the reference, and it clears the reference when it is dropped. The guard ensures
///   that there are no mutable references to the `ActiveEventLoop` while the pointer is in use.
#[allow(unsafe_code)]
pub fn with_current_event_loop<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&ActiveEventLoop) -> R,
{
    CURRENT_EVENT_LOOP.with(|cell| {
        cell.get().map(|ptr| {
            // SAFETY:
            // 1. The pointer is valid when `Some` because the `EventLoopGuard` that created it
            //    lives at least as long as the reference, and clears it when dropped. Only `with_event_loop_context`
            //    creates a new `EventLoopGuard`, and does not leak it.
            // 2. Since the pointer was created from a borrow which lives at least as long as this pointer,
            //    there are no mutable references to the `ActiveEventLoop`.
            let event_loop = unsafe { &*ptr };
            f(event_loop)
        })
    })
}

/// Provides a context for using the event loop safely.
///
/// # Arguments
/// * `event_loop` - A reference to the active event loop to be set in the context.
/// * `f` - A closure to be executed with the event loop context.
///
/// # Notes
/// - The guard must not be leaked to ensure safety. The guard manages setting and clearing
///   the event loop reference in thread-local storage.
pub fn with_event_loop_context(event_loop: &ActiveEventLoop, f: impl FnOnce()) {
    // NOTE: For safety, this guard must NOT be leaked.
    let _guard = EventLoopGuard::new(event_loop);
    f();
}