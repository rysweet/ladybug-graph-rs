use std::path::PathBuf;

fn main() {
    // lbug builds yyjson as a static library but doesn't emit a link directive
    // for it. We need to find and link it ourselves.
    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    // Walk up from our OUT_DIR to find the target build directory, then search
    // for libyyjson.a anywhere under the lbug build output.
    let target_dir = PathBuf::from(&out_dir)
        .ancestors()
        .find(|p| p.file_name().is_some_and(|n| n == "build"))
        .map(|p| p.to_path_buf());

    if let Some(build_dir) = target_dir {
        if let Some(path) = find_file(&build_dir, "libyyjson.a") {
            if let Some(parent) = path.parent() {
                println!("cargo:rustc-link-search=native={}", parent.display());
                println!("cargo:rustc-link-lib=static:+whole-archive=yyjson");
                return;
            }
        }
    }

    // Second attempt: walk up further to "debug" or "release" profile dir and
    // search under its "build" subdirectory. This handles the case where we're
    // compiled as a dependency and our OUT_DIR is nested deeper.
    let profile_dir = PathBuf::from(&out_dir)
        .ancestors()
        .find(|p| {
            p.file_name()
                .is_some_and(|n| n == "debug" || n == "release")
        })
        .map(|p| p.join("build"));

    if let Some(build_dir) = profile_dir {
        if let Some(path) = find_file(&build_dir, "libyyjson.a") {
            if let Some(parent) = path.parent() {
                println!("cargo:rustc-link-search=native={}", parent.display());
                println!("cargo:rustc-link-lib=static:+whole-archive=yyjson");
                return;
            }
        }
    }

    // Fallback: just try to link it and hope the search path is already set
    println!("cargo:rustc-link-lib=static:+whole-archive=yyjson");
}

fn find_file(dir: &std::path::Path, name: &str) -> Option<PathBuf> {
    for entry in walkdir(dir) {
        if entry.file_name().is_some_and(|n| n == name) {
            return Some(entry);
        }
    }
    None
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
