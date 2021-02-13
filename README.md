# Learning OpenGL in Rust

This repository documents my process in following the [Learning OpenGL](https://learnopengl.com/) book, but with two twists:
1. Using Rust and [glow](https://github.com/grovesNL/glow/) instead of C
2. Getting both native and web compatibility

Each section will have a corresponding tag to the commit that completes it.

## Setup

Native:

```
cargo run
```

Web:

```
cargo install cargo-make basic-http-server
cargo make build-web
basic-http-server wasm
```

Then visit [http://localhost:4000](http://localhost:4000).
