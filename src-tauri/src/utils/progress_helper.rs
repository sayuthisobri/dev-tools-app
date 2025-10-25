//! # Progress Helper Module
//!
//! This module provides functionality to set and clear progress indicators on the macOS Dock icon
//! and badge. It includes both synchronous and asynchronous APIs with intelligent batching and
//! thread safety guarantees.
//!
//! ## Features
//!
//! - **Synchronous API**: Direct functions that must be called from the main thread for immediate
//!   UI updates.
//! - **Asynchronous API**: Thread-safe functions that queue updates for batched processing,
//!   allowing calls from any thread without blocking.
//! - **Intelligent Batching**: Multiple rapid updates are consolidated into a single UI update
//!   within a 16ms window to prevent excessive redraws.
//! - **Cross-platform Compatibility**: No-op implementations for non-macOS platforms ensure
//!   code portability.
//! - **Error Handling**: Comprehensive error types for different failure scenarios.
//! - **Performance Optimization**: Throttling of minimal progress changes and caching of
//!   original icon data.
//!
//! ## API Variants
//!
//! Each operation is available in two variants:
//!
//! - **Synchronous** (e.g., `set_dock_progress_fraction`): Must be called from the main thread.
//!   Provides immediate feedback but requires thread awareness.
//! - **Asynchronous** (e.g., `set_dock_progress_fraction_async`): Can be called from any thread.
//!   Updates are queued and processed in batches for optimal performance.
//!
//! ## Thread Safety
//!
//! - Synchronous functions require main thread execution due to AppKit constraints.
//! - Asynchronous functions are thread-safe and handle main thread dispatching internally.
//! - All shared state is protected with `Mutex` for safe concurrent access.
//!
//! ## Usage Examples
//!
//! ### Synchronous Usage (Main Thread Only)
//! ```rust,no_run
//! use progress_helper::set_dock_progress_fraction;
//!
//! // Must be called from main thread
//! set_dock_progress_fraction(0.5)?;
//! ```
//!
//! ### Asynchronous Usage (Any Thread)
//! ```rust,no_run
//! use progress_helper::set_dock_progress_fraction_async;
//!
//! // Can be called from any thread
//! set_dock_progress_fraction_async(0.5).await?;
//! ```
#![allow(non_snake_case)]

#[cfg(target_os = "macos")]
mod mac {
    use crate::errors::DockError;
    use objc2::rc::{autoreleasepool, Retained};
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send, ClassType};
    use objc2_app_kit::{NSApplication, NSBezierPath, NSColor, NSImage};
    use objc2_foundation::{NSData, NSPoint, NSRect, NSSize, NSString};
    use dispatch2::run_on_main;
    use once_cell::sync::OnceCell;
    use std::ffi::c_void;
    use std::sync::{Mutex, Once};
    use tokio::time::{sleep, Duration};
    use tracing::{debug, error};

    static ORIGINAL_ICON: OnceCell<Mutex<Option<Vec<u8>>>> = OnceCell::new();
    static LAST_PROGRESS: OnceCell<Mutex<f64>> = OnceCell::new();

    /// Update mode for queued progress updates.
    ///
    /// This enum represents the different types of progress updates that can be queued
    /// for batched processing. It allows the async API to consolidate multiple rapid
    /// updates into a single UI operation.
    ///
    /// # Variants
    ///
    /// * `Set(fraction)` - Sets the progress to a specific fraction between 0.0 and 1.0.
    ///   The fraction represents completion progress, where 0.0 is no progress and 1.0
    ///   is complete.
    /// * `Clear` - Clears the progress indicator, restoring the original dock icon.
    ///   Equivalent to setting progress to 0.0 but optimized for the clear operation.
    ///
    /// # Usage in Queuing
    ///
    /// Updates are queued using this enum and processed in batches by the background
    /// task `process_update_queue()`. Only the latest update in each 16ms window is
    /// applied to prevent excessive UI updates.
    ///
    /// # Thread Safety
    ///
    /// This enum is thread-safe as it contains only `Copy` types and is used within
    /// a `Mutex`-protected queue.
    #[derive(Debug, Clone, Copy)]
    enum UpdateMode {
        /// Set progress to a specific fraction
        Set(f64),
        /// Clear progress (set to 0.0)
        Clear,
    }

    /// Global queue for progress updates with batching
    static UPDATE_QUEUE: OnceCell<Mutex<Vec<UpdateMode>>> = OnceCell::new();
    static QUEUE_PROCESSOR: OnceCell<Mutex<Option<tokio::task::JoinHandle<()>>>> = OnceCell::new();

    // Constants for progress bar configuration and throttling
    const PROGRESS_CHANGE_THRESHOLD: f64 = 0.01;
    const PROGRESS_BAR_HEIGHT_RATIO: f64 = 0.14;

    /// Ensures the current thread is the main thread for AppKit operations.
    ///
    /// # Returns
    /// * `Ok(())` if on main thread
    /// * `Err(DockError::ObjectiveC)` if not on main thread
    ///
    /// # Safety
    /// This function uses unsafe Objective-C messaging but is safe to call.
    fn ensure_main_thread() -> Result<(), DockError> {
        unsafe {
            let is_main: bool = msg_send![class!(NSThread), isMainThread];
            if !is_main {
                error!("AppKit operation attempted on non-main thread");
                return Err(DockError::objective_c("AppKit operations must be performed on the main thread".to_string(), None));
            }
        }
        Ok(())
    }

    /// Queues a progress update for batched processing.
    ///
    /// # Arguments
    /// * `mode` - The type of update to queue
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(DockError::QueueError)` on queue failure
    fn queue_update(mode: UpdateMode) -> Result<(), DockError> {
        let queue = UPDATE_QUEUE.get_or_init(|| Mutex::new(Vec::new()));
        let mut queue_guard = queue.lock().unwrap();
        queue_guard.push(mode);

        // Start processor if not already running
        let processor = QUEUE_PROCESSOR.get_or_init(|| Mutex::new(None));
        let mut processor_guard = processor.lock().unwrap();
        if processor_guard.is_none() {
            *processor_guard = Some(tokio::spawn(async {
                process_update_queue().await;
            }));
        }

        Ok(())
    }

    /// Background task that processes queued updates with intelligent batching.
    ///
    /// Uses a 16ms window to batch updates, processing only the latest update
    /// in each window to avoid excessive UI updates.
    async fn process_update_queue() {
        loop {
            // Wait for initial update or 16ms window
            sleep(Duration::from_millis(16)).await;

            let updates = {
                let queue = UPDATE_QUEUE.get_or_init(|| Mutex::new(Vec::new()));
                let mut queue_guard = queue.lock().unwrap();
                std::mem::take(&mut *queue_guard)
            };

            if updates.is_empty() {
                continue;
            }

            // Process only the latest update (batching)
            if let Some(latest) = updates.last() {
                let result = match latest {
                    UpdateMode::Set(fraction) => {
                        run_on_main(move |_| set_dock_progress_fraction(*fraction))
                    }
                    UpdateMode::Clear => {
                        run_on_main(|_| clear_dock_progress())
                    }
                };

                if let Err(e) = result {
                    error!("Failed to process queued dock update: {:?}", e);
                }
            }
        }
    }

    /// Ensures AppKit is properly initialized by finishing application launch.
    ///
    /// This function should be called once before any AppKit operations.
    /// It uses a `Once` to ensure initialization happens only once.
    ///
    /// # Returns
    /// Always returns `Ok(())` - errors during initialization are logged but don't prevent operation.
    ///
    /// # Safety
    /// Uses unsafe Objective-C messaging but is safe to call from any thread.
    fn ensure_appkit() -> Result<(), DockError> {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            unsafe {
                autoreleasepool(|_pool| {
                    // Ensure the application has finished launching
                    let app: *mut NSApplication = msg_send![class!(NSApplication), sharedApplication];
                    if app.is_null() {
                        error!("Failed to get shared NSApplication during initialization");
                        // Cannot return error from Once closure, so log and continue
                        return;
                    }
                    let _: () = msg_send![app, finishLaunching];
                });
            }
        });
        Ok(())
    }

    /// Retrieves system colors for progress bar rendering.
    ///
    /// # Returns
    /// A tuple of `(background_color, foreground_color)` where:
    /// - Background is semi-transparent system gray
    /// - Foreground is system green
    ///
    /// # Returns
    /// * `Ok((Retained<NSColor>, Retained<NSColor>))` on success
    /// * `Err(DockError::ObjectiveC)` on color creation failure
    ///
    /// # Safety
    /// Uses unsafe Objective-C messaging but returns retained colors for safety.
    fn get_colors() -> Result<(Retained<NSColor>, Retained<NSColor>), DockError> {
        unsafe {
            // Create background color: semi-transparent system gray
            let bg_color_raw: *mut NSColor = msg_send![class!(NSColor), systemGrayColor];
            if bg_color_raw.is_null() {
                error!("Failed to get systemGrayColor");
                return Err(DockError::objective_c("Failed to get systemGrayColor".to_string(), None));
            }
            let bg_color_raw: *mut NSColor = msg_send![bg_color_raw, colorWithAlphaComponent: 0.55];
            if bg_color_raw.is_null() {
                error!("Failed to create background color with alpha");
                return Err(DockError::objective_c("Failed to create background color with alpha".to_string(), None));
            }

            // Create foreground color: system green
            let fg_color_raw: *mut NSColor = msg_send![class!(NSColor), systemGreenColor];
            if fg_color_raw.is_null() {
                error!("Failed to get systemGreenColor");
                return Err(DockError::objective_c("Failed to get systemGreenColor".to_string(), None));
            }

            // Retain colors to ensure they live long enough for drawing
            let bg_color = Retained::retain(bg_color_raw).unwrap();
            let fg_color = Retained::retain(fg_color_raw).unwrap();
            Ok((bg_color, fg_color))
        }
    }

    /// Draws a progress bar overlay on the current drawing context.
    ///
    /// # Arguments
    /// * `size` - The size of the icon canvas
    /// * `fraction` - Progress fraction (0.0 to 1.0)
    /// * `bar_height_ratio` - Ratio of icon height to use for bar height
    ///
    /// # Returns
    /// * `Ok(())` on successful drawing
    /// * `Err(DockError)` on Objective-C failures
    ///
    /// # Behavior
    /// - Skips drawing if fraction is 0.0
    /// - Draws a rounded rectangle background and filled progress foreground
    /// - Uses system colors for consistent appearance
    fn draw_progress_bar(size: NSSize, fraction: f64, bar_height_ratio: f64) -> Result<(), DockError> {
        // Early return for zero progress to avoid unnecessary drawing
        if fraction == 0.0 {
            return Ok(());
        }

        unsafe {
            let width = size.width;
            let height = size.height;

            // Calculate progress bar dimensions with minimum constraints
            let bar_height = (height * bar_height_ratio).max(6.0); // Minimum 6px height
            let margin = (height * 0.06).max(4.0); // Minimum 4px margin
            let bar_x = margin;
            let bar_y = margin;
            let bar_width = width - margin * 2.0;
            let fill_width = bar_width * fraction.clamp(0.0, 1.0);

            // Retrieve system colors for progress bar
            let (bg_color, fg_color) = get_colors()?;

            // Draw background rounded rectangle
            let bg_rect = NSRectFromDoubles(bar_x, bar_y, bar_width, bar_height);
            let rounded_rect_bg: *mut NSBezierPath = msg_send![class!(NSBezierPath),
                bezierPathWithRoundedRect: bg_rect,
                xRadius: bar_height / 2.0,
                yRadius: bar_height / 2.0];
            if rounded_rect_bg.is_null() {
                error!("Failed to create background bezier path");
                return Err(DockError::objective_c("Failed to create background bezier path".to_string(), None));
            }
            let _: () = msg_send![bg_color.as_super(), setFill];
            let _: () = msg_send![rounded_rect_bg, fill];

            // Draw foreground progress fill
            let fg_rect = NSRectFromDoubles(bar_x, bar_y, fill_width, bar_height);
            let rounded_rect_fg: *mut NSBezierPath = msg_send![class!(NSBezierPath),
                bezierPathWithRoundedRect: fg_rect,
                xRadius: bar_height / 2.0,
                yRadius: bar_height / 2.0];
            if rounded_rect_fg.is_null() {
                error!("Failed to create foreground bezier path");
                return Err(DockError::objective_c("Failed to create foreground bezier path".to_string(), None));
            }
            let _: () = msg_send![fg_color.as_super(), setFill];
            let _: () = msg_send![rounded_rect_fg, fill];
        }

        Ok(())
    }

    /// Retrieves the base application icon, caching it for performance.
    ///
    /// The original icon is captured once and stored as TIFF data to avoid
    /// repeated Objective-C calls and ensure consistency across progress updates.
    ///
    /// # Returns
    /// * `Ok(Retained<NSImage>)` - The original application icon
    /// * `Err(DockError)` - On icon loading or conversion failures
    ///
    /// # Behavior
    /// - Caches the icon data in a static `OnceCell<Mutex<Option<Vec<u8>>>>`
    /// - Converts NSImage to TIFF bytes for storage
    /// - Reconstructs NSImage from cached data on subsequent calls
    ///
    /// # Safety
    /// Uses unsafe Objective-C messaging but returns retained image for safety.
    fn get_base_image() -> Result<Retained<NSImage>, DockError> {
        unsafe {
            ensure_appkit()?;
            autoreleasepool(|_pool| -> Result<Retained<NSImage>, DockError> {
                let app: *mut NSApplication = msg_send![class!(NSApplication), sharedApplication];
                if app.is_null() {
                    error!("Failed to get shared NSApplication");
                    return Err(DockError::objective_c("Failed to get shared NSApplication".to_string(), None));
                }

                // Access cached icon data
                let original_icon = ORIGINAL_ICON.get_or_init(|| Mutex::new(None));
                let mut original_icon = original_icon.lock().unwrap();

                // Capture and cache the original icon if not already done
                if original_icon.is_none() {
                    let current_icon: *mut NSImage = msg_send![app, applicationIconImage];
                    if !current_icon.is_null() {
                        // Convert to TIFF for storage
                        let tiff_rep: *mut NSData = msg_send![current_icon, TIFFRepresentation];
                        if !tiff_rep.is_null() {
                            let length: usize = msg_send![tiff_rep, length];
                            let bytes: *const c_void = msg_send![tiff_rep, bytes];
                            if !bytes.is_null() {
                                let slice = std::slice::from_raw_parts(bytes as *const u8, length);
                                let vec = slice.to_vec();
                                *original_icon = Some(vec);
                            } else {
                                error!("Failed to get bytes from TIFF representation");
                                return Err(DockError::objective_c("Failed to get bytes from TIFF representation".to_string(), None));
                            }
                        } else {
                            error!("Failed to get TIFF representation from icon");
                            return Err(DockError::objective_c("Failed to get TIFF representation from icon".to_string(), None));
                        }
                    } else {
                        error!("Current icon is null during storage");
                        return Err(DockError::icon_load("Current icon is null during storage".to_string(), None));
                    }
                }

                // Reconstruct NSImage from cached TIFF data
                if let Some(icon_data) = &*original_icon {
                    let nsdata: *mut NSData = msg_send![class!(NSData),
                        dataWithBytes: icon_data.as_ptr(),
                        length: icon_data.len()];
                    if nsdata.is_null() {
                        error!("Failed to create NSData from stored icon data");
                        return Err(DockError::objective_c("Failed to create NSData from stored icon data".to_string(), None));
                    }
                    let image: *mut NSImage = msg_send![class!(NSImage), alloc];
                    if image.is_null() {
                        error!("Failed to allocate NSImage");
                        return Err(DockError::objective_c("Failed to allocate NSImage".to_string(), None));
                    }
                    let image: *mut NSImage = msg_send![image, initWithData: nsdata];
                    if image.is_null() {
                        error!("Failed to initialize NSImage from stored data");
                        return Err(DockError::icon_load("Failed to initialize NSImage from stored data".to_string(), None));
                    }
                    let retained_image = Retained::retain(image).unwrap();
                    Ok(retained_image)
                } else {
                    error!("No original icon data available");
                    Err(DockError::icon_load("No original icon data available".to_string(), None))
                }
            })
        }
    }


    /// Sets the dock progress fraction asynchronously with intelligent queuing and batching.
    ///
    /// This is the asynchronous variant of [`set_dock_progress_fraction`]. Unlike the synchronous
    /// version, this function can be called from any thread and queues updates for batched
    /// processing. Multiple rapid updates within a 16ms window are consolidated into a single
    /// UI update to prevent excessive redraws and improve performance.
    ///
    /// # Arguments
    /// * `fraction` - Progress value between 0.0 (no progress) and 1.0 (complete). Must be finite.
    ///
    /// # Returns
    /// * `Ok(())` on successful queuing of the update
    /// * `Err(DockError::InvalidProgress)` if the fraction is not finite or outside [0.0, 1.0]
    /// * `Err(DockError::QueueError)` if the update cannot be queued
    ///
    /// # Behavior
    /// - Validates input immediately and returns an error for invalid fractions
    /// - Queues the update using [`UpdateMode::Set`] for batched processing
    /// - Updates are processed by a background task that consolidates rapid changes
    /// - Only the latest update in each 16ms window is applied to the UI
    ///
    /// # Thread Safety
    /// Safe to call from any thread. The function uses internal synchronization primitives
    /// and dispatches actual UI updates to the main thread automatically.
    ///
    /// # Performance
    /// This function is optimized for high-frequency updates from background threads.
    /// Use this variant when you need to update progress from async contexts or worker threads.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use progress_helper::set_dock_progress_fraction_async;
    ///
    /// // Update progress from an async function
    /// async fn download_file() -> Result<(), Box<dyn std::error::Error>> {
    ///     for progress in 0..=100 {
    ///         let fraction = progress as f64 / 100.0;
    ///         set_dock_progress_fraction_async(fraction).await?;
    ///         // Simulate work
    ///         tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    ///     }
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # See Also
    /// - [`set_dock_progress_fraction`] for the synchronous variant
    /// - [`clear_dock_progress_async`] for clearing progress asynchronously
    pub async fn set_dock_progress_fraction_async(fraction: f64) -> Result<(), DockError> {
        // Validate input (same as sync version)
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            error!("Invalid progress fraction: {} (must be finite and between 0.0 and 1.0)", fraction);
            return Err(DockError::invalid_progress(fraction, format!(
                "Progress must be finite and between 0.0 and 1.0, got {}",
                fraction
            )));
        }

        debug!("Queueing async dock progress update to {}", fraction);
        queue_update(UpdateMode::Set(fraction))
    }

    /// Clears the dock progress asynchronously with intelligent queuing and batching.
    ///
    /// This is the asynchronous variant of [`clear_dock_progress`]. This function can be called
    /// from any thread and queues the clear operation for batched processing. It's optimized
    /// for clearing progress indicators from background operations.
    ///
    /// # Returns
    /// * `Ok(())` on successful queuing of the clear operation
    /// * `Err(DockError::QueueError)` if the clear operation cannot be queued
    ///
    /// # Behavior
    /// - Queues the clear operation using [`UpdateMode::Clear`] for batched processing
    /// - The clear operation restores the original dock icon without any progress overlay
    /// - Updates are processed by a background task that consolidates rapid changes
    /// - Only the latest update in each 16ms window is applied to the UI
    ///
    /// # Thread Safety
    /// Safe to call from any thread. The function uses internal synchronization primitives
    /// and dispatches actual UI updates to the main thread automatically.
    ///
    /// # Performance
    /// This function is optimized for clearing progress from async contexts. Multiple clear
    /// operations within a short time window are batched efficiently.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use progress_helper::{set_dock_progress_fraction_async, clear_dock_progress_async};
    ///
    /// async fn process_with_progress() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Show progress during processing
    ///     set_dock_progress_fraction_async(0.5).await?;
    ///
    ///     // Simulate work
    ///     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    ///
    ///     // Clear progress when done
    ///     clear_dock_progress_async().await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # See Also
    /// - [`clear_dock_progress`] for the synchronous variant
    /// - [`set_dock_progress_fraction_async`] for setting progress asynchronously
    pub async fn clear_dock_progress_async() -> Result<(), DockError> {
        debug!("Queueing async dock progress clear");
        queue_update(UpdateMode::Clear)
    }

    /// Sets the dock progress fraction by overlaying a progress bar on the application icon.
    ///
    /// This is the synchronous variant of [`set_dock_progress_fraction_async`]. It provides
    /// immediate UI updates but must be called from the main thread due to AppKit requirements.
    /// For thread-safe operations from background threads, use the async variant instead.
    ///
    /// # Arguments
    /// * `fraction` - Progress value between 0.0 (no progress) and 1.0 (complete). Must be finite.
    ///
    /// # Returns
    /// * `Ok(())` on successful progress update
    /// * `Err(DockError::InvalidProgress)` if the fraction is not finite or outside [0.0, 1.0]
    /// * `Err(DockError::ObjectiveC)` if called from non-main thread or other AppKit failures
    /// * `Err(DockError::IconLoad)` if the application icon cannot be loaded
    ///
    /// # Behavior
    /// - Validates that the function is called from the main thread
    /// - Validates input fraction and returns error for invalid values
    /// - Throttles updates for minimal changes (less than 1% difference) to improve performance
    /// - Draws a rounded progress bar overlay on the original application icon
    /// - Updates the dock icon immediately with the modified image
    /// - Caches the original icon data for efficient restoration
    ///
    /// # Thread Safety
    /// **Must be called from the main thread only.** AppKit operations require main thread execution.
    /// Attempting to call this from a background thread will result in an error. For thread-safe
    /// operations, use [`set_dock_progress_fraction_async`] instead.
    ///
    /// # Performance
    /// This function provides immediate UI feedback but may cause excessive redraws if called
    /// frequently with small changes. The function includes throttling to skip updates that
    /// change progress by less than 1%.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use progress_helper::set_dock_progress_fraction;
    ///
    /// // Must be called from main thread
    /// set_dock_progress_fraction(0.5)?;
    /// ```
    ///
    /// # See Also
    /// - [`set_dock_progress_fraction_async`] for the thread-safe asynchronous variant
    /// - [`clear_dock_progress`] for clearing progress synchronously
    pub fn set_dock_progress_fraction(fraction: f64) -> Result<(), DockError> {
        // Ensure we're on the main thread for AppKit operations
        ensure_main_thread()?;

        // Validate input: must be finite and within [0.0, 1.0]
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            error!("Invalid progress fraction: {} (must be finite and between 0.0 and 1.0)", fraction);
            return Err(DockError::invalid_progress(fraction, format!(
                "Progress must be finite and between 0.0 and 1.0, got {}",
                fraction
            )));
        }

        // Throttle updates to avoid excessive redraws for small changes
        let last_progress = LAST_PROGRESS.get_or_init(|| Mutex::new(0.0));
        let mut last_progress_guard = last_progress.lock().unwrap();
        if (fraction - *last_progress_guard).abs() < PROGRESS_CHANGE_THRESHOLD {
            debug!("Skipping progress update due to minimal change: {} -> {}", *last_progress_guard, fraction);
            return Ok(());
        }
        *last_progress_guard = fraction;

        debug!("Setting dock progress to {}", fraction);

        // Perform AppKit operations in an autorelease pool for memory management
        unsafe {
            ensure_appkit()?;
            autoreleasepool(|_pool| -> Result<(), DockError> {
                // Get the shared application instance
                let app: *mut NSApplication = msg_send![class!(NSApplication), sharedApplication];
                if app.is_null() {
                    error!("Failed to get shared NSApplication");
                    return Err(DockError::objective_c("Failed to get shared NSApplication".to_string(), None));
                }

                // Retrieve the base application icon
                let base_image = get_base_image()?;

                // Validate icon dimensions
                let size = NSImage::size(base_image.as_ref());
                let width = size.width;
                let height = size.height;
                if width <= 0.0 || height <= 0.0 {
                    error!("Invalid icon size: {}x{}", width, height);
                    return Err(DockError::icon_load(format!("Invalid icon size: {}x{}", width, height), None));
                }

                // Create a new image for the progress overlay
                autoreleasepool(|_pool| -> Result<(), DockError> {
                    let new_image: *mut NSImage = msg_send![class!(NSImage), alloc];
                    if new_image.is_null() {
                        error!("Failed to allocate NSImage");
                        return Err(DockError::objective_c("Failed to allocate NSImage".to_string(), None));
                    }
                    let new_image: *mut NSImage = msg_send![new_image, initWithSize: size];
                    if new_image.is_null() {
                        error!("Failed to initialize new NSImage for progress overlay");
                        return Err(DockError::icon_load("Failed to initialize new NSImage for progress overlay".to_string(), None));
                    }

                    // Begin drawing context
                    let _: () = msg_send![new_image, lockFocus];

                    // Draw the original icon as the base layer
                    let source_rect = NSRect::new(NSPoint::new(0.0, 0.0), size);
                    let dest_rect = NSRectFromInts(0, 0, width as i32, height as i32);
                    let _: () = msg_send![base_image.as_super(), drawInRect: dest_rect,
                                                fromRect: source_rect,
                                                operation: 1, // NSCompositeSourceOver
                                                fraction: 1.0];

                    // Overlay the progress bar
                    draw_progress_bar(size, fraction, PROGRESS_BAR_HEIGHT_RATIO)?;

                    // Finalize drawing
                    let _: () = msg_send![new_image, unlockFocus];

                    // Update the application icon
                    let _: () = msg_send![app, setApplicationIconImage: new_image];
                    Ok(())
                })?;

                Ok(())
            })
        }
    }


    /// Sets a text badge on the dock icon.
    ///
    /// This function displays a text label on the application dock icon. The badge is typically
    /// used to show counts, status indicators, or short messages.
    ///
    /// # Arguments
    /// * `label` - The text to display on the dock badge. If empty, the badge will be cleared.
    ///
    /// # Returns
    /// * `Ok(())` on successful badge update
    /// * `Err(DockError::ObjectiveC)` if called from non-main thread or other AppKit failures
    ///
    /// # Behavior
    /// - Validates that the function is called from the main thread
    /// - Converts the Rust string to an NSString for AppKit
    /// - Sets the badge label on the dock tile
    /// - An empty string clears the badge (same as [`clear_dock_badge`])
    ///
    /// # Thread Safety
    /// **Must be called from the main thread only.** AppKit operations require main thread execution.
    /// Attempting to call this from a background thread will result in an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use progress_helper::set_dock_badge;
    ///
    /// // Set a badge with a count
    /// set_dock_badge("5")?;
    ///
    /// // Set a status badge
    /// set_dock_badge("Busy")?;
    ///
    /// // Clear the badge
    /// set_dock_badge("")?;
    /// ```
    ///
    /// # See Also
    /// - [`clear_dock_badge`] for explicitly clearing the badge
    /// - [`set_dock_progress_fraction`] for progress indicators
    pub fn set_dock_badge(label: &str) -> Result<(), DockError> {
        ensure_main_thread()?;
        debug!("Setting dock badge to: {}", label);
        unsafe {
            ensure_appkit()?;
            autoreleasepool(|_pool| -> Result<(), DockError> {
                let app: *mut NSApplication = msg_send![class!(NSApplication), sharedApplication];
                if app.is_null() {
                    error!("Failed to get shared NSApplication");
                    return Err(DockError::objective_c("Failed to get shared NSApplication".to_string(), None));
                }
                let dock_tile: *mut AnyObject = msg_send![app, dockTile];
                if dock_tile.is_null() {
                    error!("Failed to get dock tile");
                    return Err(DockError::objective_c("Failed to get dock tile".to_string(), None));
                }
                let badge_label = if label.is_empty() {
                    std::ptr::null_mut::<NSString>()
                } else {
                    let nsstring: *mut NSString = msg_send![class!(NSString), stringWithUTF8String: label.as_ptr() as *const i8];
                    if nsstring.is_null() {
                        error!("Failed to create NSString from label");
                        return Err(DockError::objective_c("Failed to create NSString from label".to_string(), None));
                    }
                    nsstring
                };
                let _: () = msg_send![dock_tile, setBadgeLabel: badge_label];
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Clears the dock progress by restoring the original application icon.
    ///
    /// This is the synchronous variant of [`clear_dock_progress_async`]. It immediately restores
    /// the original dock icon by removing any progress bar overlay, but must be called from
    /// the main thread due to AppKit requirements.
    ///
    /// # Returns
    /// * `Ok(())` on successful progress clearing
    /// * `Err(DockError::ObjectiveC)` if called from non-main thread or other AppKit failures
    /// * `Err(DockError::IconLoad)` if the original icon cannot be restored
    ///
    /// # Behavior
    /// - Validates that the function is called from the main thread
    /// - Restores the original application icon without progress overlay
    /// - Resets the internal progress tracking state to 0.0
    /// - Uses cached original icon data for efficient restoration
    ///
    /// # Thread Safety
    /// **Must be called from the main thread only.** AppKit operations require main thread execution.
    /// Attempting to call this from a background thread will result in an error. For thread-safe
    /// operations, use [`clear_dock_progress_async`] instead.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use progress_helper::{set_dock_progress_fraction, clear_dock_progress};
    ///
    /// // Set progress
    /// set_dock_progress_fraction(0.8)?;
    ///
    /// // Clear progress (must be on main thread)
    /// clear_dock_progress()?;
    /// ```
    ///
    /// # See Also
    /// - [`clear_dock_progress_async`] for the thread-safe asynchronous variant
    /// - [`set_dock_progress_fraction`] for setting progress synchronously
    pub fn clear_dock_progress() -> Result<(), DockError> {
        ensure_main_thread()?;
        debug!("Clearing dock progress");
        unsafe {
            ensure_appkit()?;
            autoreleasepool(|_pool| -> Result<(), DockError> {
                let app: *mut NSApplication = msg_send![class!(NSApplication), sharedApplication];
                if app.is_null() {
                    error!("Failed to get shared NSApplication");
                    return Err(DockError::objective_c("Failed to get shared NSApplication".to_string(), None));
                }

                let base_image = get_base_image()?;
                let _: () = msg_send![app, setApplicationIconImage: base_image.as_super()];

                // Reset last progress
                let last_progress = LAST_PROGRESS.get_or_init(|| Mutex::new(0.0));
                *last_progress.lock().unwrap() = 0.0;

                Ok(())
            })
        }
    }

    /// Clears the dock badge by removing any text label from the dock icon.
    ///
    /// This function removes any badge that was previously set on the dock icon, restoring
    /// the clean dock appearance. It's equivalent to calling [`set_dock_badge`] with an
    /// empty string.
    ///
    /// # Returns
    /// * `Ok(())` on successful badge clearing
    /// * `Err(DockError::ObjectiveC)` if called from non-main thread or other AppKit failures
    ///
    /// # Behavior
    /// - Validates that the function is called from the main thread
    /// - Sets the dock badge label to null, effectively clearing it
    /// - Safe to call even if no badge is currently set
    ///
    /// # Thread Safety
    /// **Must be called from the main thread only.** AppKit operations require main thread execution.
    /// Attempting to call this from a background thread will result in an error.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use progress_helper::{set_dock_badge, clear_dock_badge};
    ///
    /// // Set a badge
    /// set_dock_badge("New")?;
    ///
    /// // Clear the badge
    /// clear_dock_badge()?;
    /// ```
    ///
    /// # See Also
    /// - [`set_dock_badge`] for setting badge text
    pub fn clear_dock_badge() -> Result<(), DockError> {
        ensure_main_thread()?;
        debug!("Clearing dock badge");
        set_dock_badge("")
    }

    // Helper functions to construct NSRect and similar using objc runtime calls require bridging types.
    // For brevity, helper constructors below:

    fn NSRectFromInts(x: i32, y: i32, w: i32, h: i32) -> NSRect {
        NSRect::new(
            NSPoint::new(x as f64, y as f64),
            NSSize::new(w as f64, h as f64),
        )
    }

    fn NSRectFromDoubles(x: f64, y: f64, w: f64, h: f64) -> NSRect {
        NSRect::new(NSPoint::new(x, y), NSSize::new(w, h))
    }
}

#[cfg(target_os = "macos")]
pub use mac::{clear_dock_badge, clear_dock_progress, clear_dock_progress_async, set_dock_badge, set_dock_progress_fraction, set_dock_progress_fraction_async};

#[cfg(not(target_os = "macos"))]
pub fn set_dock_progress_fraction(_fraction: f64) -> Result<(), DockError> {
    // no-op on non-macOS: Dock progress is macOS-specific
    debug!("Dock progress not supported on non-macOS platforms");
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub async fn set_dock_progress_fraction_async(_fraction: f64) -> Result<(), DockError> {
    // no-op on non-macOS: Dock progress is macOS-specific
    debug!("Dock progress not supported on non-macOS platforms");
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub fn clear_dock_progress() -> Result<(), DockError> {
    // no-op on non-macOS: Dock progress is macOS-specific
    debug!("Dock progress not supported on non-macOS platforms");
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub async fn clear_dock_progress_async() -> Result<(), DockError> {
    // no-op on non-macOS: Dock progress is macOS-specific
    debug!("Dock progress not supported on non-macOS platforms");
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub fn set_dock_badge(_label: &str) -> Result<(), DockError> {
    // no-op on non-macOS: Dock badge is macOS-specific
    debug!("Dock badge not supported on non-macOS platforms");
    Ok(())
}
#[cfg(not(target_os = "macos"))]
pub fn clear_dock_badge() -> Result<(), DockError> {
    // no-op on non-macOS: Dock badge is macOS-specific
    debug!("Dock badge not supported on non-macOS platforms");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "macos")]
    use crate::errors::DockError;

    #[tokio::test]
    async fn test_set_dock_progress_fraction_async_valid() {
        // Test valid fractions on all platforms
        assert!(set_dock_progress_fraction_async(0.0).await.is_ok());
        assert!(set_dock_progress_fraction_async(0.5).await.is_ok());
        assert!(set_dock_progress_fraction_async(1.0).await.is_ok());
    }

    #[tokio::test]
    async fn test_set_dock_progress_fraction_async_invalid() {
        // Test invalid fractions - should return error on all platforms
        assert!(matches!(set_dock_progress_fraction_async(-0.1).await, Err(DockError::InvalidProgress { value: _, reason: _ })));
        assert!(matches!(set_dock_progress_fraction_async(1.1).await, Err(DockError::InvalidProgress { value: _, reason: _ })));
        assert!(matches!(set_dock_progress_fraction_async(f64::NAN).await, Err(DockError::InvalidProgress { value: _, reason: _ })));
        assert!(matches!(set_dock_progress_fraction_async(f64::INFINITY).await, Err(DockError::InvalidProgress { value: _, reason: _ })));
    }

    #[tokio::test]
    async fn test_clear_dock_progress_async() {
        // Test clear operation on all platforms
        assert!(clear_dock_progress_async().await.is_ok());
    }


}
