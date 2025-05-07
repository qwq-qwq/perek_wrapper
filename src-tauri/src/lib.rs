use tauri::{Builder, State, AppHandle, Wry, Runtime, WebviewWindowBuilder, WebviewUrl};
use tauri::menu::{Menu, Submenu, MenuItemBuilder};
use tauri_plugin_store::Builder as StoreBuilder;
use tauri_plugin_store::Store;
use serde_json::Value;
use log::LevelFilter;
use tauri_plugin_log::Builder as LogBuilder;

#[tauri::command]
fn open_settings<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
  WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
    .title("Настройки")
    .inner_size(400.0, 300.0)
    .resizable(false)
    .build()
    .map_err(|e| e.to_string())?;
  Ok(())
}


#[tauri::command]
fn load_settings(store: State<'_, Store<Wry>>) -> Value {
  let proxy = store.get("proxy")
    .and_then(|v| v.as_str().map(str::to_string))
    .unwrap_or_default();
  let theme = store.get("theme")
    .and_then(|v| v.as_str().map(str::to_string))
    .unwrap_or_else(|| "light".into());
  let autolaunch = store.get("autolaunch")
    .and_then(|v| v.as_bool())
    .unwrap_or(false);
  serde_json::json!({ "proxy": proxy, "theme": theme, "autolaunch": autolaunch })
}

#[tauri::command]
fn save_settings(store: State<'_, Store<Wry>>, settings: Value) -> Result<(), String> {
  if let Some(p) = settings.get("proxy").and_then(|v| v.as_str()) {
    store.set("proxy".to_string(), Value::String(p.into()));
  }
  if let Some(t) = settings.get("theme").and_then(|v| v.as_str()) {
    store.set("theme".to_string(), Value::String(t.into()));
  }
  if let Some(a) = settings.get("autolaunch").and_then(|v| v.as_bool()) {
    store.set("autolaunch".to_string(), Value::Bool(a));
  }
  store.save().map_err(|e| e.to_string())?;
  Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let mut builder = Builder::default()
    .plugin(StoreBuilder::default().build())
    .invoke_handler(tauri::generate_handler![open_settings, load_settings, save_settings]);

  if cfg!(debug_assertions) {
    builder = builder.plugin(
      LogBuilder::default()
        .level(LevelFilter::Info)
        .build(),
    );
  }
  // Build application menu with Settings
  builder = builder
    .menu(|app| {
      let settings = MenuItemBuilder::new("Настройки").id("settings").build(app).unwrap();
      let quit = MenuItemBuilder::new("Выход").id("quit").build(app).unwrap();
      let file_menu = Submenu::with_items(app, "Файл", true, &[&settings, &quit]).unwrap();
      let menu = Menu::with_items(app, &[&file_menu]).unwrap();
      Ok(menu)
    })
    .on_menu_event(move |app_handle, event| {
      let id = event.id().0.as_str();
      if id == "settings" {
        let _ = open_settings(app_handle.clone());
      } else if id == "quit" {
        std::process::exit(0);
      }
    });

  builder
    .setup(|_| Ok(()))
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
