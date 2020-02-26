use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn scan_for_component(path: &Path) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter() {
        if let Ok(e) = entry {
            if !e.path().to_str().unwrap().contains("test") {
                if e.file_name() == "Component.yaml" {
                    let fname = e.into_path();
                    result.push(fname);
                }
            }
        }
    }
    result
}


#[test]
fn test_scan_find_file(){
    let p = Path::new("./");
    let result = scan_for_component(p);
    println!("{:?}", result);
    assert_eq!(vec![PathBuf::from("./resources/Component.yaml")],result);
}