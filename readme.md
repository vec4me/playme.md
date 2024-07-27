# playme.md
Explore an interactive 3D world within your GitHub's readme.md!

![](assets/pretty.gif)

## Dependencies
- Rust (with `cargo`)

## Building
To build the project, ensure you have Rust and Cargo installed. Then, navigate to the project directory and run:

```sh
cargo build --release
```

For any specific dependencies or setup, refer to the Cargo.toml file in the root directory and the asahi_renderer subdirectory, if applicable.

## Execution
After building, you can run the server with:

```sh
cargo run
```
or
```sh
./target/release/serve
```

## Extra
To deploy this project on Heroku, you need the Rust buildpack. You can find it at https://github.com/emk/heroku-buildpack-rust. Just connect the repository and deploy using this buildpack.