# Rust Axiom Profiler prototype

### Installation
- Clone this repository.
- Graphviz is needed to render SVG images. See https://www.graphviz.org/download/.
- It is recommended to install a linker such as `lld` or `mold` to speed up Rust compilation (see https://nnethercote.github.io/perf-book/compile-times.html). If not using `mold`, the rustflags line of `.cargo/config.toml` must be changed; for example, with `lld`` it should instead be: 

    ```rustflags = ["-C", "link-arg=-fuse-ld=lld"]```

    If using the default linker, remove the line.
- To try parser directly, `cargo run --bin prototype`. Currently, this binary does not provide a way to manually stop parsing (skip remaining lines) and have the program continue.
### Actix server
- In the top-level directory of the crate, enter `cargo run --bin actix-server` in terminal. 
- The server will remain active until stopped manually (i.e. Ctrl+C or Cmd+C). If a panic occurs in parsing/outputting/rendering, the server seems to still respond to new requests.

### Yew frontend
- See https://github.com/richardluo20/axiom-profiler-yew-GUI/ for repo and instructions.