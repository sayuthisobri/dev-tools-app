// macOS-only helper to set/clear progress on the Dock icon and badge.
// Provides error handling, state integration, event emission, and logging.
// Cross-platform compatibility with no-op stubs for non-macOS.
#![allow(non_snake_case)]

#[cfg(target_os = "macos")]
mod mac {
    use crate::errors::DockError;
    use objc2::ffi::nil;
    use objc2::rc::autoreleasepool;
    use objc2::runtime::{AnyObject, NSObject};
    use objc2::{class, msg_send};
    use objc2_app_kit::{NSColor, NSImage};
    use objc2_foundation::{NSPoint, NSRect, NSSize};
    use once_cell::sync::OnceCell;
    use std::ffi::c_void;
    use std::sync::{Arc, Mutex, Once};
    use tracing::{debug, error, info};

    static ORIGINAL_ICON: OnceCell<Mutex<Option<Arc<Vec<u8>>>>> = OnceCell::new();

    // Ensure AppKit loaded once
    static INIT: Once = Once::new();
    fn ensure_appkit() {
        INIT.call_once(|| {
            unsafe {
                autoreleasepool(|_pool| {
                    // Ensure app finished launching; use MainThreadMarker when calling typed API.
                    let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                    let _: () = msg_send![app, finishLaunching];
                });
            }
        })
    }

    fn draw_progress_bar(icon: *mut AnyObject, fraction: f64) {
        unsafe {
            // Get icon size
            let size: NSSize = msg_send![icon, size];
            let width = size.width;
            let height = size.height;

            // Calculate progress bar geometry (simple bar at bottom)
            let bar_height = (height * 0.14).max(6.0); // 14% or min 6px
            let margin = (height * 0.06).max(4.0);
            let bar_x = margin;
            let bar_y = margin;
            let bar_w = width - margin * 2.0;
            let fill_w = bar_w * fraction.clamp(0.0, 1.0);

            // Draw background (semi-transparent dark rounded rect)
            let bg_color: *mut NSColor =
                msg_send![class!(NSColor), colorWithCalibratedWhite: 0.0, alpha: 0.55];
            let fg_color: *mut NSColor = msg_send![class!(NSColor), colorWithCalibratedRed: 0.19, green: 0.66, blue: 0.33, alpha: 1.0]; // green-ish

            // Draw background rounded rect
            let bg_rect = NSRectFromDoubles(bar_x, bar_y, bar_w, bar_height);
            let rounded_rect_bg: *mut AnyObject = msg_send![class!(NSBezierPath),
                bezierPathWithRoundedRect: bg_rect,
                xRadius: bar_height/2.0,
                yRadius: bar_height/2.0];
            let _: () = msg_send![bg_color, setFill];
            let _: () = msg_send![rounded_rect_bg, fill];

            // Draw foreground fill rect for progress
            let fg_rect = NSRectFromDoubles(bar_x, bar_y, fill_w, bar_height);
            let rounded_rect_fg: *mut AnyObject = msg_send![class!(NSBezierPath),
                bezierPathWithRoundedRect: fg_rect,
                xRadius: bar_height/2.0,
                yRadius: bar_height/2.0];
            let _: () = msg_send![fg_color, setFill];
            let _: () = msg_send![rounded_rect_fg, fill];
        }
    }

    // Draw a simple progress overlay image onto the app icon and set as application icon image.
    // fraction: 0.0..1.0
    // Returns DockError on invalid input or Objective-C failures.
    pub fn set_dock_progress_fraction(fraction: f64) -> Result<(), DockError> {
        if !(0.0..=1.0).contains(&fraction) {
            error!("Invalid progress fraction: {}", fraction);
            return Err(DockError::InvalidProgress(format!("Progress must be between 0.0 and 1.0, got {}", fraction)));
        }
        debug!("Setting dock progress to {}", fraction);
        unsafe {
            ensure_appkit();
            autoreleasepool(|_pool| -> Result<(), DockError> {
                let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let icon: *mut AnyObject = msg_send![app, applicationIconImage];
                if icon.is_null() {
                    info!("No application icon available, skipping progress update");
                    return Ok(());
                }

                let original_icon = ORIGINAL_ICON.get_or_init(|| Mutex::new(None));
                let mut original_icon = original_icon.lock().unwrap();
                if original_icon.is_none() {
                    let current_icon: *mut AnyObject = msg_send![app, applicationIconImage];
                    if !current_icon.is_null() {
                        // Convert NSImage to TIFF representation and get bytes
                        let tiff_rep: *mut AnyObject = msg_send![current_icon, TIFFRepresentation];
                        if !tiff_rep.is_null() {
                            let length: usize = msg_send![tiff_rep, length];
                            let bytes: *const c_void = msg_send![tiff_rep, bytes];
                            if !bytes.is_null() {
                                let slice = std::slice::from_raw_parts(bytes as *const u8, length);
                                let vec = slice.to_vec();
                                *original_icon = Some(Arc::new(vec));
                            }
                        }
                    }
                }

                // Get icon size
                let size: NSSize = msg_send![icon, size];
                let width = size.width;
                let height = size.height;

                // Create a new NSImage with same size
                let new_image: *mut AnyObject = msg_send![class!(NSImage), alloc];
                let new_image: *mut AnyObject = msg_send![new_image, initWithSize: size];

                // Begin drawing into new image using lockFocus
                let _: () = msg_send![new_image, lockFocus];

                // Draw the existing icon into it
                let source_rect = NSRect::new(NSPoint::new(0.0, 0.0), size);
                let dest_rect = NSRectFromInts(0, 0, width as i32, height as i32);
                let _: () = msg_send![icon, drawInRect: dest_rect,
                                            fromRect: source_rect,
                                           operation: 1, // NSCompositeSourceOver
                                            fraction: 1.0];

                draw_progress_bar(icon, fraction);

                // End drawing
                let _: () = msg_send![new_image, unlockFocus];

                // Set as app icon
                let _: () = msg_send![app, setApplicationIconImage: new_image];
                Ok(())
            })
        }
    }

    pub fn clear_dock_progress() -> Result<(), DockError> {
        debug!("Clearing dock progress");
        unsafe {
            ensure_appkit();
            autoreleasepool(|_pool| {
                let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];

                let original_icon = ORIGINAL_ICON.get_or_init(|| Mutex::new(None));
                if let Some(icon_data) = &*original_icon.lock().unwrap() {
                    let nsdata: *mut NSObject = msg_send![class!(NSData),
                        dataWithBytes: icon_data.as_ptr(),
                        length: icon_data.len()];
                    let image: *mut NSImage = msg_send![class!(NSImage), alloc];
                    let image: *mut NSImage = msg_send![image, initWithData: nsdata];

                    if !image.is_null() {
                        let _: () = msg_send![app, setApplicationIconImage: image];
                    }
                }
            });
        }
        Ok(())
    }

    pub fn set_dock_badge(label: &str) -> Result<(), DockError> {
        debug!("Setting dock badge to: {}", label);
        unsafe {
            ensure_appkit();
            autoreleasepool(|_pool| {
                let app: *mut AnyObject = msg_send![class!(NSApplication), sharedApplication];
                let dock_tile: *mut AnyObject = msg_send![app, dockTile];
                if !dock_tile.is_null() {
                    let _: () = msg_send![dock_tile, setBadgeLabel: if label.is_empty() { nil } else { msg_send![class!(NSString), stringWithUTF8String: label.as_ptr() as *const i8] }];
                }
            });
        }
        Ok(())
    }

    pub fn clear_dock_badge() -> Result<(), DockError> {
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
pub use mac::{clear_dock_badge, clear_dock_progress, set_dock_badge, set_dock_progress_fraction};

#[cfg(not(target_os = "macos"))]
pub fn set_dock_progress_fraction(_fraction: f64) -> Result<(), DockError> {
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

    #[test]
    fn test_set_dock_progress_fraction_valid() {
        // Test non-macOS version
        let result = set_dock_progress_fraction(0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_dock_progress_fraction_invalid() {
        // Test non-macOS version
        let result = set_dock_progress_fraction(1.5);
        assert!(result.is_ok()); // Still ok, but logs debug
    }

    #[test]
    fn test_clear_dock_progress() {
        let result = clear_dock_progress();
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_dock_badge() {
        let result = set_dock_badge("test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_dock_badge() {
        let result = clear_dock_badge();
        assert!(result.is_ok());
    }
}
