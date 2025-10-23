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
    let current_path = env::var("PATH").expect("env PATH is not set in $PATH");
    let mut paths: Vec<PathBuf> = env::split_paths(&current_path).collect();

    // Add new paths
    for path in new_paths {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }

    // Join paths back together
    let new_path = env::join_paths(paths).expect("env PATH join error");

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
    append_env_path(
        vec![
            "/usr/local/bin",
            "/usr/local/sbin",
            "/opt/homebrew/bin",
            "/opt/homebrew/sbin",
            "/opt/podman/bin",
            "~/bun/bin",
            "~/.bin",
            "~/.local/bin",
            "~/.config/cargo/bin",
        ]
        .into_iter()
        .map(|path| utils::expand_tilde(path))
        .map(PathBuf::from)
        .collect(),
    );
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
            setup_menu(app)?;

            let window = app
                .get_webview_window("main")
                .expect("Failed to get main window");

            #[cfg(target_os = "macos")]
            {
                apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                    .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");
            }

            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125)))
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            Ok(())
        })
        .on_window_event(handle_window_event())
        .invoke_handler(commands::setup_handler())
        .run(tauri::generate_context!())
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

    app.set_menu(menu)?;

    app.on_menu_event(move |app, event| match event.id().0.as_str() {
        "settings" => {
            app.emit("go-to", "/settings")
                .expect("Unable to emit go-to event");
        }
        "refresh" => {
            app.emit("go-to", "page::refresh")
                .expect("Unable to emit window event");
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
        let mut state = state.lock().unwrap();
        let current_monitor_name =
            utils::get_current_monitor_name(w).unwrap_or("default".to_string());
        let window_state = &mut state.window;
        window_state.height = w.inner_size().unwrap().height;
        window_state.width = w.inner_size().unwrap().width;
        window_state.x = w.outer_position().unwrap().x;
        window_state.y = w.outer_position().unwrap().y;
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
        let _ = app.emit(EVENT_NAME, &window_state);
        // println!("App state: {:?}", state);
    }
}

fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = Registry::default()
        .with(env_filter)
        .with(http::HTTPTraceLayer); // Make sure this is the correct type that implements Layer

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
