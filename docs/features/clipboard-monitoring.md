# Clipboard Monitoring Feature

## Overview
Synapse monitors the system clipboard across different platforms to capture and synchronize data.

## Implementation Details

### Desktop (Windows, macOS, Linux)
- **Mechanism**: Event-driven monitoring using `tauri-plugin-clipboard`.
- **Event Flow**:
  1. System clipboard content changes.
  2. `tauri-plugin-clipboard` detects the change via native OS APIs (where available) or optimized polling.
  3. The plugin triggers a callback in the Rust backend.
  4. The backend logs the change and notifies the frontend via Tauri events.
  5. Content is synchronized to the persistence layer (planned SpacetimeDB).

### Mobile (Android, iOS)
- **Constraints**: Background access is restricted by OS security policies.
- **Strategy**:
  - **Active Sync**: Clipboard is checked when the application returns to the foreground.
  - **Passive Capture**: Users may be prompted to manually sync or use specific system sharing mechanisms.
  - **Foreground Service (Android)**: May be used to keep the process alive, though direct background clipboard access remains restricted.

## Future Enhancements
- Support for non-text data (images, HTML).
- Intelligence-based filtering (avoiding sensitive data like passwords).
