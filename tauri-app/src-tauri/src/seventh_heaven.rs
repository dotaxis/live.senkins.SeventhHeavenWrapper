use std::{fs::{self, File}, io::{self, Write}, path::PathBuf, thread, time::Duration};

use log::{as_serde, info};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::{steam_manager::SteamManager, wine_manager::WineManager};

#[derive(Serialize, Clone)]
struct StatusUpdate {
    step: String,
    running: bool,
    success: bool,
}

fn required_packages() -> Vec<String> {
    return vec![
        "d3dx9".to_string(),
        "msls31".to_string(),
        "riched20".to_string(),
        "corefonts".to_string(),
        "d3dcompiler_43".to_string(),
        "d3dcompiler_47".to_string(),
        "dinput".to_string(),
    ];
}

fn prepare_cd_drive(wine_manager: &WineManager) -> io::Result<()> {
    let path = wine_manager.get_c_path("FF7DISC1");

    
    return fs::create_dir_all(&path)
        .and_then(|_| File::create(path.join(".windows-label")))
        .and_then(|mut label_path| label_path.write_all( b"FF7DISC1")
            .and_then(|_| label_path.flush()))
        .and_then(|_| File::create(path.join(".windows-serial")))
        .and_then(|mut label_path| label_path.write_all( b"44000000")
            .and_then(|_| label_path.flush()));
    
    wine_manager.load_cd("FF7DISC1", "x");
}

#[tauri::command]
pub(crate) async fn install_run(app_handle: AppHandle) -> Result<(), ()> {
    info!("Starting install run");
    let required = required_packages();

    let mut wine_manager = WineManager::new();

    let steam_home = with_status(&app_handle, format!("Detecting Steam..."), || -> Result<PathBuf,String> {
        SteamManager::detect_steam_home().ok_or(String::from("Failed to find Steam - is it installed?"))
        // TODO - error handling
    }).unwrap();

    let steam = SteamManager::new(steam_home);

    for package in &required {
        with_status(&app_handle,format!("installing {package}..."), || -> Result<(), String> {
            wine_manager.install_package(&package)
            // TODO - error handling
        }).unwrap();
    }
    with_status(&app_handle,format!("Configuring CD Drive..."), || -> io::Result<()> {
        prepare_cd_drive(&wine_manager)
        // TODO - error handling
    }).unwrap();

    with_status(&app_handle,format!("Setting up FF7..."), || -> io::Result<()> {
        todo!("Fetch FF7 and install it");
        // TODO - error handling
    }).unwrap();

    with_status(&app_handle,format!("Setting up Seventh Heaven..."), || -> io::Result<()> {
        todo!("Fetch Seventh Heaven and install it");
        // TODO - error handling
    }).unwrap();

    with_status(&app_handle,format!("Setting up FFNX..."), || -> io::Result<()> {
        todo!("Fetch FFNX and install it?");
        // TODO - error handling
    }).unwrap();


    with_status(&app_handle,format!("Patching FF7 for Seventh Heaven..."), || -> io::Result<()> {
        todo!("Apply FF7 Steam patch");
        // TODO - error handling
    }).unwrap();


    Ok(())
}

fn status_update(status: StatusUpdate, app_handle: &AppHandle) {
    info!("Posting status: [{}]", as_serde!(status));
    app_handle.emit_all("install_progress", status).unwrap();
    // FIXME - need a sleep to allow events to propagate, otherwise multiple may overwrite each other
    thread::sleep(Duration::from_millis(10));
}

fn with_status<T,R>(app_handle: &AppHandle, status_line: String, mut f: impl FnMut() -> std::result::Result<T,R> ) -> std::result::Result<T,R> {

    status_update(
        StatusUpdate {
            step: status_line.clone(),
            running: true,
            success: false,
        },
        &app_handle,
    );

    let result = match f() {
        Ok(retval) => retval,
        Err(e) => {

            status_update(
                StatusUpdate {
                    step: status_line.clone(),
                    running: false,
                    success: false,
                },
                &app_handle,
            );
            return Err(e)
        }
    };

    
    status_update(
        StatusUpdate {
            step: status_line,
            running: false,
            success: true,
        },
        &app_handle,
    );
    Ok(result)
}