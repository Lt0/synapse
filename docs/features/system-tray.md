# System Tray Integration

## Overview
The desktop application provides a system tray (menu bar) icon for background accessibility and quick actions.

## Features
- **Status Indicator**: The Synapse icon remains visible in the system tray when the main window is closed.
- **Context Menu**:
  - **Show**: Restores the main application window.
  - **Hide**: Minimizes the window to the tray.
  - **Quit**: Completely terminates the application.

## Behavior
- **Closing the Window**: Clicking the "X" button on the main window hides it to the tray instead of exiting the process. This ensures background clipboard monitoring continues uninterrupted.
- **Platform Specifics**:
  - **macOS**: Appears in the top-right Menu Bar.
  - **Windows**: Appears in the bottom-right Notification Area (System Tray).
  - **Linux**: Appears in the status area (support varies by Desktop Environment).
