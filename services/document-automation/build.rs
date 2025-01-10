fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=src/db/documents.rs");
    println!("cargo:rerun-if-changed=src/db/workflows.rs");
    println!("cargo:rerun-if-changed=migrations/");

    // Tell Cargo that the build script depends on all SQL files in the workspace
    println!("cargo:rerun-if-changed=**/*.sql");
}
