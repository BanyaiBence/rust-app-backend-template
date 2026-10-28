// `sqlx::migrate!()` embeds migrations/ into the binary at compile time; without this, adding a
// migration would not trigger a rebuild and the server would run a stale set.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
