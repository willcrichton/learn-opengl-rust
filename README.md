# Learn OpenGL in Rust

This repository documents my process in following the [Learn OpenGL](https://learnopengl.com/) book, but with a few twists:
1. Using Rust and [glow](https://github.com/grovesNL/glow/) instead of C
2. Getting both native and web compatibility
3. Incrementally designing Rustic abstractions over GL patterns

Each chapter has a corresponding tag for the commit that implements it. You can view every tag online using WebGL: https://willcrichton.net/learn-opengl-rust/

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
