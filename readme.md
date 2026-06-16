# wppr
Cross platform wallpaper scraper that downloads the latest wallpapers from [wallpaper a day](https://www.wallpaper-a-day.com).<br>
Downloaded wallpapers are saved under `~/Pictures/wppr/`.<br>
Logs are saved under `~/.cache/wppr/logs/`

### Dependencies
[Rust](https://rust-lang.org/) toolchain.<br>
Build dependencies are automatically downloaded by cargo.

### Build
To build the project clone the git repo and cd into it, then run `cargo build --release`.<br>
To install `wppr` to your system run `cargo install --bin wppr --path=./crates/wppr` followed by adding `~/.cargo/bin` to your path, or copy the compiled binary's into your path.

### Usage
After installation run `wppr` inside of your terminal to start the program.

### Acknowledgments
Inspired by [dwu](https://github.com/starrieste/dwu)
