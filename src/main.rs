mod component;
mod components;

use std::path::{Path};



fn main() {
    let elements = components::scan_for_component(Path::new("./"));
    for i in elements {
        let cmpt = component::load_from_pathbuf(&i).unwrap();
        println!("{}", cmpt.Run)
    }
    println!("Hello, world!");
}