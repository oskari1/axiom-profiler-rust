# Rust Axiom Profiler prototype

### Installation
- `git clone` repositories
- Graphviz is needed to render SVG images. See https://www.graphviz.org/download/.
- It is recommended to install a linker such as lld or mold to speed up compilation (see https://nnethercote.github.io/perf-book/compile-times.html). If not using mold, the rustflags line of `.cargo/config.toml` must be changed; for example, with lld it should be: 
```rustflags = ["-C", "link-arg=-fuse-ld=lld"]```
For using the default linker, the line can be removed.
- To try parser directly, `cargo run --bin prototype`. Unfortunately, trying to stopping the program manually will halt it completely instead of continuing with a partially processed trace file.

### Actix server
- `git clone`
- `cargo run`
Server will remain active until stopped manually (e.g. Ctrl+C or Cmd+C). If a panic occurs in parsing/outputting/rendering, the server should still respond to new requests.

### frontend
- `trunk serve` and then go to http://127.0.0.1:8080 in a browser, or `trunk serve --open` to open it in browser (it may still be necessary to switch windows manually). Manual stopping is also needed.