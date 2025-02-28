#![cfg_attr(feature = "no_std", no_std)]

mod dungeon;

#[guests_macro::proving_entrypoint]
pub fn main(watcher_map: [[bool; 8]; 8], dagger: (u8, u8), path: [(u8, u8); 8]) {
    let path = dungeon::Square::from_array(path);
    dungeon::is_valid_path(path, dagger.into());
    dungeon::is_safe_path(path, watcher_map);
}
