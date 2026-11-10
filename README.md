# Bplus Rename

Rename many files and folders at once on Windows.

Pick a folder, select the items in the list, set the renaming criteria and check the **New Name**
column before pressing **Rename**. Criteria are applied in this order:

1. RegEx · 2. Name · 3. Replace · 4. Case · 5. Remove · 6. Move/Copy Parts · 7. Add ·
8. Auto Date · 9. Append Folder Name · 10. Numbering · 11. Extension

Also: file filters (mask, folders/files, hidden, sub-folders, name/path length), copy/move to a
location, a Preview of all changes, saved favourites (`.bplus`), click-to-sort list columns, and
**Ctrl+Z** to undo the last rename.

## Download

Get `bplus.exe` from the [latest release](https://github.com/muleskin/Bplus/releases/latest). It is a
single 64-bit program with no installer and no runtime to install. The release also carries the
user guide (`Bplus-Rename-User-Guide.docx`).

## Build

Requires Rust (stable, MSVC toolchain).

```
cd app
cargo build --release
```

The program is written to `app/target/release/bplus.exe`.
