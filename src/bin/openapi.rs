//! Prints the OpenAPI document as JSON, e.g. for generating a typed frontend client:
//! `make openapi` writes it to `docs/openapi.json`.

fn main() {
    let spec = backend::openapi::openapi()
        .to_pretty_json()
        .expect("serializing the OpenAPI document");
    println!("{spec}");
}
