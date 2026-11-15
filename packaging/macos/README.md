# macOS packaging

`cargo-bundle` uses the `[package.metadata.bundle]` and
`[package.metadata.bundle.osx]` sections in `Cargo.toml`. The document types
for `.md` / `.markdown` come from `packaging/macos/Info.plist.ext`, which is
appended to the generated `Info.plist`.

Build the app bundle:

```bash
cargo install cargo-bundle
cargo bundle --release
```

The result is `target/release/bundle/osx/MD Viewer.app`. Copy it to
`/Applications` to make it available in Finder's "Open With" menu. macOS may
still require choosing "Other..." and selecting the app once before it appears
in the list. Launching a document with `open -a "MD Viewer" file.md` also works.
