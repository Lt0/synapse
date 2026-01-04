# Build System & CI/CD

## Development Workflow
Syncing across local environments using Dioxus and Tauri.

### Key Commands
- **Frontend Dev**: `cd frontend && dx serve`
- **Backend Dev**: `cd backend && cargo run`
- **Native Dev**: `cargo tauri dev`

## Cross-Platform Builds
- **Native**: Built via `cargo tauri build` on the respective host OS.
- **Linux on macOS**: Supported via Docker using the provided `Dockerfile` and `build-linux.sh`.

## CI/CD Pipeline
GitHub Actions are used for automated releases.
- **Workflow**: `.github/workflows/release.yml`
- **Targets**: Windows (MSVC), macOS (Universal), Linux (Debian/AppImage).
- **Trigger**: Pushing a version tag (e.g., `v*`).
