use std::path::{Path, PathBuf};
use directories::{BaseDirs, ProjectDirs, UserDirs};
use crate::world::*;

fn fatal_msgbox(window: &sdl2::video::Window, msg: &str) {
    use sdl2::messagebox::{show_simple_message_box, MessageBoxFlag};
    show_simple_message_box(MessageBoxFlag::ERROR, "Fatal Error", msg, window).unwrap();
    panic!();
}

fn expect_msgbox<T: std::fmt::Debug, E: std::fmt::Debug>(
    window: &sdl2::video::Window,
    r: Result<T, E>,
    msg: &str,
) -> T {
    if r.is_ok() {
        r.unwrap()
    } else {
        fatal_msgbox(window, format!("{} - {:?}", msg, r.unwrap_err()).as_str());
        panic!();
    }
}

fn get_save_directory_path() -> PathBuf {
    // TODO msgbox the unwrap
    PathBuf::from(
        ProjectDirs::from("com", "creikey", "Explain")
            .unwrap()
            .data_dir(),
    )
}

fn get_save_file_path() -> PathBuf {
    get_save_directory_path().join("save.explain")
}

pub fn save(window: &sdl2::video::Window, world: &World) {
    let save_directory = get_save_directory_path();
    expect_msgbox(
        &window,
        std::fs::create_dir_all(&save_directory),
        format!(
            "failed to create save directory in {}",
            save_directory.to_str().unwrap()
        )
        .as_str(),
    );
    use std::fs::File;
    use std::io::prelude::*;
    let saved_world = SavedWorld::from_world(world);
    let encoded = bincode::serialize(&saved_world).unwrap();
    let save_file_path = get_save_file_path();
    println!(
        "{} | {}",
        save_directory.to_str().unwrap(),
        save_file_path.to_str().unwrap()
    );

    use std::fs::OpenOptions;
    let mut save_file = if save_file_path.exists() {
        OpenOptions::new().write(true).open(save_file_path).unwrap()
    } else {
        File::create(save_file_path).unwrap()
    };
    save_file.write_all(encoded.as_slice()).unwrap();
}

pub fn load_or_new_world() -> World {
    // TODO this stuff should definitely expect_msgbox to show that the save file is corrupt
    let save_path = get_save_file_path();
    let to_return: World;
    if save_path.exists() {
        let bytes = std::fs::read(save_path).unwrap();
        let saved_world: SavedWorld = bincode::deserialize(bytes.as_slice()).unwrap();
        to_return = saved_world.into_world();
    } else {
        to_return = World::new();
    }

    to_return
}