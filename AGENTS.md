# Project Instructions & Guidelines

## Architecture & Boundaries
- **Frontend:** Angular 22+ located in `src/` (Standalone components, Signals for state).
- **Backend/Desktop:** Tauri + Rust located in `src-tauri/`.
- Do NOT mix Angular logic into the Rust layer except through Tauri `invoke()` commands.

## Verification & Checks
- Always verify frontend changes with: `npm run build` or `ng test --watch=false`.
- Always verify Rust changes with: `cargo check` inside `src-tauri/`.
- Run `cargo clippy` to check for idiom/lint issues before declaring a backend task complete.

## Conventions
- Use `tauri::command` for all Rust handlers exposed to JS/TS.
