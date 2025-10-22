// Path: src-tauri/src/dock_progress.rs
// Intended action: macOS-only helper to set/clear progress on the Dock icon
#![allow(non_snake_case)]

#[cfg(target_os = "macos")]
mod mac {
    use cocoa::appkit::NSApp;
    use cocoa::base::{id, nil};
    use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize};
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};
    use std::sync::Once;

    // Ensure AppKit loaded once
    static INIT: Once = Once::new();

    fn ensure_appkit() {
        INIT.call_once(|| {
            unsafe {
                // create an autorelease pool to be safe
                let _pool = NSAutoreleasePool::new(nil);
                // ensure we have a shared application instance (should already exist in Tauri)
                let app: id = NSApp();
                let _: id = msg_send![app, finishLaunching];
            }
        })
    }

    // Draw a simple progress overlay image onto the app icon and set as application icon image.
    // fraction: 0.0..1.0
    pub fn set_dock_progress_fraction(fraction: f64) {
        unsafe {
            ensure_appkit();
            let pool = NSAutoreleasePool::new(nil);

            // Get the shared app icon image
            let app: id = NSApp();
            let icon: id = msg_send![app, applicationIconImage];
            if icon == nil {
                // nothing to do
                return;
            }

            // Get icon size
            let size: NSSize = msg_send![icon, size];
            let width = size.width;
            let height = size.height;

            // Create a new NSImage with same size
            let nsimage_class = Class::get("NSImage").unwrap();
            let new_image: id = msg_send![nsimage_class, alloc];
            let new_image: id = msg_send![new_image, initWithSize: size];

            // Begin drawing into new image using lockFocus
            let _: () = msg_send![new_image, lockFocus];

            // Draw the existing icon into it
            let _: () = msg_send![icon, drawInRect: NSRectFromInts(0, 0, width as i32, height as i32)
                                                fromRect: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, height)) operation: 1 fraction: 1.0];

            // Calculate progress bar geometry (simple bar at bottom)
            let bar_height = (height * 0.14).max(6.0); // 14% or min 6px
            let margin = (height * 0.06).max(4.0);
            let bar_x = margin;
            let bar_y = margin;
            let bar_w = width - margin * 2.0;
            let fill_w = bar_w * fraction.clamp(0.0, 1.0);

            // Draw background (semi-transparent dark rounded rect)
            // Use NSBezierPath and set fill color
            let nscolor_class = Class::get("NSColor").unwrap();
            let bg_color: id = msg_send![nscolor_class, colorWithCalibratedWhite: 0.0 alpha: 0.55];
            let fg_color: id = msg_send![nscolor_class, colorWithCalibratedRed: 0.19 green: 0.66 blue: 0.33 alpha: 1.0]; // green-ish

            let nsbezier_class = Class::get("NSBezierPath").unwrap();
            let rounded_rect_bg: id = msg_send![nsbezier_class, bezierPathWithRoundedRect: NSRectFromDoubles(bar_x, bar_y, bar_w, bar_height) xRadius: bar_height/2.0 yRadius: bar_height/2.0];
            let _: () = msg_send![bg_color, setFill];
            let _: () = msg_send![rounded_rect_bg, fill];

            // Draw foreground fill rect for progress
            let rounded_rect_fg: id = msg_send![nsbezier_class, bezierPathWithRoundedRect: NSRectFromDoubles(bar_x, bar_y, fill_w, bar_height) xRadius: bar_height/2.0 yRadius: bar_height/2.0];
            let _: () = msg_send![fg_color, setFill];
            let _: () = msg_send![rounded_rect_fg, fill];

            // End drawing
            let _: () = msg_send![new_image, unlockFocus];

            // Set as app icon
            let _: () = msg_send![app, setApplicationIconImage: new_image];

            let _: () = pool.drain();
        }
    }

    pub fn clear_dock_progress() {
        unsafe {
            ensure_appkit();
            let pool = NSAutoreleasePool::new(nil);
            let app: id = NSApp();
            // restore the original icon by passing nil (AppKit will use bundle icon)
            let _: () = msg_send![app, setApplicationIconImage: nil];
            let _: () = pool.drain();
        }
    }

    // Helper functions to construct NSRect and similar using objc runtime calls require bridging types.
    // For brevity, helper constructors below:

    fn NSRectFromInts(x: i32, y: i32, w: i32, h: i32) -> NSRect {
        NSRect::new(NSPoint::new(x as f64, y as f64), NSSize::new(w as f64, h as f64))
    }

    fn NSRectFromDoubles(x: f64, y: f64, w: f64, h: f64) -> NSRect {
        NSRect::new(NSPoint::new(x, y), NSSize::new(w, h))
    }
}

#[cfg(target_os = "macos")]
pub use mac::{clear_dock_progress, set_dock_progress_fraction};

#[cfg(not(target_os = "macos"))]
pub fn set_dock_progress_fraction(_fraction: f64) {
    // no-op on non-macOS
}
#[cfg(not(target_os = "macos"))]
pub fn clear_dock_progress() {}

