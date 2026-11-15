# MD Viewer

A small desktop Markdown viewer built with Rust, eframe/egui, and
`egui_commonmark`.

## Features

- Open a Markdown file from the command line: `mdview path/to/file.md`
- Open files with the native file dialog (`Ctrl+O`)
- Rendered headings, lists, tables, code blocks with syntax highlighting,
  links, and images resolved relative to the Markdown file
- Auto-reload when the file changes on disk, keeping the scroll position
- Zoom with `Ctrl++`, `Ctrl+-`, `Ctrl+0`
- Follows the system light/dark theme
- One-command per-user install: `mdview --install`

## Requirements

- Rust 1.95 or newer (2021 edition)
- Linux only: a working X11 or Wayland session and the usual GUI build
  dependencies for `eframe` (`libxkbcommon`, `libwayland`, OpenGL drivers).

## Build

```bash
cargo build --release
```

The binary is written to `target/release/mdview`
(`target\release\mdview.exe` on Windows).

Run it:

```bash
./target/release/mdview README.md
```

No arguments opens an empty window with an "Open file..." button.

## Install

`mdview --install` does everything: it copies the executable to a stable
per-user location, installs the desktop/registry entries, and registers the
`.md` / `.markdown` file types.

### Linux

```bash
cargo build --release
./target/release/mdview --install
```

It installs to `~/.local/bin/mdview`, writes
`~/.local/share/applications/mdview.desktop`, refreshes the desktop database,
and makes MD Viewer the default handler for `text/markdown`. `~/.local/bin`
must be on `PATH` so the desktop entry can launch the app.

### Windows

```powershell
cargo build --release
.\target\release\mdview.exe --install
```

It installs to `%LOCALAPPDATA%\Programs\MD Viewer\mdview.exe` and registers the
app under `HKEY_CURRENT_USER\Software\Classes`. Right-click a `.md` file ->
"Open with" -> "MD Viewer" to make it the default. The release build uses the
Windows GUI subsystem, so no terminal window appears. If Explorer still shows
an old menu, restart Explorer or sign out and back in.

### macOS

See `packaging/macos/README.md`:

```bash
cargo install cargo-bundle
cargo bundle --release
cp -R "target/release/bundle/osx/MD Viewer.app" /Applications/
```

## Project layout

```
src/main.rs       command-line argument handling and eframe startup
src/app.rs        application state, UI, shortcuts, auto-reload
src/watcher.rs    notify-based file watcher
packaging/        "Open with" integration for Linux, Windows, macOS
```

## License

MIT
