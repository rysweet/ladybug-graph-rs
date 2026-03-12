use std::path::PathBuf;

fn main() {
    // lbug builds yyjson as a static library but doesn't emit a link directive
    // for it. We need to find and link it ourselves.
    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    // The lbug build artifacts are in a sibling build directory.
    // Walk up from our OUT_DIR to find the lbug build output.
    let target_dir = PathBuf::from(&out_dir)
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "build"))
        .map(|p| p.to_path_buf());

    if let Some(build_dir) = target_dir {
        // Search for libyyjson.a under the lbug build output
        for entry in walkdir(&build_dir) {
            if entry.ends_with("libyyjson.a") {
                if let Some(parent) = entry.parent() {
                    println!("cargo:rustc-link-search=native={}", parent.display());
                    println!("cargo:rustc-link-lib=static:+whole-archive=yyjson");
                    return;
                }
            }
        }
    }

    // Fallback: just try to link it and hope the search path is already set
    println!("cargo:rustc-link-lib=static:+whole-archive=yyjson");
}

fn walkdir(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(walkdir(&path));
            } else {
                results.push(path);
            }
        }
    }
    results
}
