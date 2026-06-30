// lbug bundles and links yyjson itself (its build script emits the link
// directive for the static libyyjson.a it builds). Earlier lbug releases did
// not, so this crate used to locate and link libyyjson.a manually; doing so
// against a modern lbug double-links yyjson and fails with duplicate-symbol
// errors at link time. There is nothing for this build script to do.
fn main() {}
