mod errors;
mod services;
mod states;
mod store;
mod utils;

use services::commands;
use services::http;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{AboutMetadata, MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    App, Emitter, Manager, Window, WindowEvent,
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

type SharedAppState = Arc<Mutex<states::AppState>>;

fn append_env_path(new_paths: Vec<PathBuf>) {
    // Get the current PATH variable
    let current_path = match env::var("PATH") {
        Ok(path) => path,
        Err(e) => {
            log::error!("Failed to get PATH environment variable: {}", e);
            return;
        }
    };
    let mut paths: Vec<PathBuf> = env::split_paths(&current_path).collect();

    // Add new paths
    for path in new_paths {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }

    // Join paths back together
    let new_path = match env::join_paths(paths) {
        Ok(path) => path,
        Err(e) => {
            log::error!("Failed to join PATH: {}", e);
            return;
        }
    };

    // Set the new PATH
    env::set_var("PATH", new_path);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // let mut window_state = WindowState {
    //     width: 1280, // Default width
    //     height: 720, // Default height
    //     x: 100,      // Default x position
    //     y: 100,      // Default y position
    // };
    // let window_state = Rc::new(RefCell::new(window_state));
    let paths_to_append = vec![
        "/usr/local/bin",
        "/usr/local/sbin",
        "/opt/homebrew/bin",
        "/opt/homebrew/sbin",
        "/opt/podman/bin",
        "~/bun/bin",
        "~/.bin",
        "~/.local/bin",
        "~/.config/cargo/bin",
    ];
    let expanded_paths: Vec<PathBuf> = paths_to_append
        .into_iter()
        .map(|path| utils::expand_tilde(path))
        .collect();
    append_env_path(expanded_paths);
    init_logging();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Trace)
                // .level_for("t", log::LevelFilter::Debug)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(3))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_shellx::init(true))
        .plugin(tauri_plugin_system_info::init())
        // .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(SharedAppState::new(Mutex::new(states::AppState::default())))
        .setup(|app| {
            // Initialize crash reporting
            let crash_log_path = app.path().app_log_dir()?.join("msms-dev-tools-crash.log");
            utils::crash_reporter::init_crash_reporting(crash_log_path);

            setup_menu(app)?;

            let window = match app.get_webview_window("main") {
                Some(w) => w,
                None => {
                    log::error!("Failed to get main window");
                    return Err("Failed to get main window".into());
                }
            };

            #[cfg(target_os = "macos")]
            {
                if let Err(e) = apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None) {
                    log::error!("Failed to apply vibrancy: {}", e);
                }
            }

            #[cfg(target_os = "windows")]
            if let Err(e) = apply_blur(&window, Some((18, 18, 18, 125))) {
                log::error!("Failed to apply blur: {}", e);
            }

            Ok(())
        })
        .on_window_event(handle_window_event())
        .invoke_handler(commands::setup_handler())
        .run(tauri::generate_context!())
        .map_err(|e| {
            log::error!("Failed to run Tauri application: {}", e);
            e
        })
        .expect("error while running tauri application");

    // tracing_subscriber::registry()
    //     .with(http::HTTPTraceLayer)
    //     .init();
}

fn setup_menu(app: &mut App) -> Result<(), Box<dyn Error>> {
    #[cfg(desktop)]
    let settings = MenuItemBuilder::new("Settings")
        .id("settings")
        .accelerator("CmdOrCtrl+,")
        .build(app)?;
    let mut app_submenu_builder = SubmenuBuilder::new(app, "App")
        .about(Some(AboutMetadata {
            ..Default::default()
        }))
        .separator()
        .item(&settings)
        .separator();
    // #[cfg(debug_assertions)]
    {
        let refresh = MenuItemBuilder::new("Refresh")
            .id("refresh")
            .accelerator("CmdOrCtrl+r")
            .build(app)?;
        app_submenu_builder = app_submenu_builder.separator().item(&refresh);
    }
    let app_submenu = app_submenu_builder.services().separator().quit().build()?;
    let menu = MenuBuilder::new(app)
        .items(&[
            &app_submenu,
            // ... include references to any other submenus
        ])
        .build()?;

    app.set_menu(menu).map_err(|e| {
        log::error!("Failed to set menu: {}", e);
        e
    })?;

    app.on_menu_event(move |app, event| match event.id().0.as_str() {
        "settings" => {
            if let Err(e) = app.emit("go-to", "/settings") {
                log::error!("Unable to emit go-to event: {}", e);
            }
        }
        "refresh" => {
            if let Err(e) = app.emit("go-to", "page::refresh") {
                log::error!("Unable to emit window event: {}", e);
            }
        }
        _ => {}
    });
    Ok(())
}

fn handle_window_event() -> fn(&Window, &WindowEvent) {
    |w, _event| {
        const EVENT_NAME: &str = "window:event";
        let app = w.app_handle();
        let state = app.state::<SharedAppState>();
        let mut state = match state.lock() {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to lock app state: {}", e);
                return;
            }
        };
        let current_monitor_name =
            utils::get_current_monitor_name(w).unwrap_or("default".to_string());
        let window_state = &mut state.window;
        window_state.height = w.inner_size().unwrap_or_default().height;
        window_state.width = w.inner_size().unwrap_or_default().width;
        window_state.x = w.outer_position().unwrap_or_default().x;
        window_state.y = w.outer_position().unwrap_or_default().y;
        window_state.monitor_name = current_monitor_name.clone();
        window_state.scale_factor = w.scale_factor().unwrap_or(-1.0);
        // match event {
        //     WindowEvent::Resized(size) => {
        //         // Update the width and height when the window is resized
        //         window_state.width = size.width;
        //         window_state.height = size.height;
        //         let _ = app.emit(EVENT_NAME, &window_state);
        //     }
        //     WindowEvent::Moved(position) => {
        //         // Update the x and y position when the window is moved
        //         window_state.x = position.x;
        //         window_state.y = position.y;
        //         window_state.monitor_name = current_monitor_name.clone();
        //         let _ = app.emit(EVENT_NAME, &window_state);
        //     }
        //     _ => {}
        // }
        if let Err(e) = app.emit(EVENT_NAME, &window_state) {
            log::error!("Failed to emit window event: {}", e);
        }
        // println!("App state: {:?}", state);
    }
}

fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = Registry::default()
        .with(env_filter)
        .with(http::HTTPTraceLayer); // Make sure this is the correct type that implements Layer

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Failed to set tracing subscriber: {}", e);
    }

    // Install panic hook to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().unwrap_or_else(|| std::panic::Location::caller());
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s
        } else {
            "Unknown panic"
        };
        log::error!("Panic occurred at {}:{}: {}", location.file(), location.line(), message);
        // Report crash to file
        utils::crash_reporter::report_panic(panic_info);
    }));
}
