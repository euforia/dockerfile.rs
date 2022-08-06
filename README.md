# dockerfile.rs

This repository is a library that allows to programmatically build dockerfiles.

## Example

```rust
...
let mut df = Dockerfile::new();
df.comment("Build image")
    .from("rust")
    .arg("APP_NAME")
    .workdir("/usr/src/${APP_NAME}");

df.comment("Download and cache deps.")
    .copy(&["Cargo.toml", "Cargo.lock", "./"])
    .run("mkdir ./src && touch ./src/lib.rs")
    .run("cargo build --release")
    .run("rm -f ./src/lib.rs");

df.comment("Build the app")
    .copy(&["src", "./src"])
    .run("cargo build --release");

df.comment("Build the app runtime image")
    .from("debian:buster-slim")
    .arg("APP_NAME=APP_NAME")
    .copy(&[
        "--from=0",
        "/usr/src/${APP_NAME}/target/release/${APP_NAME}",
        "/usr/local/bin/${APP_NAME}",
    ])
    .cmd(&["${APP_NAME}"]);

println!("{:?}", df.synth());
...
```