# Synapse (神经突触)

跨平台剪贴板。

作为常驻任务运行在设备上，同时本地应用或者 Web 支持通过 UI 查看和删除历史记录。

> Synapse 的发音是 [ˈsɪnæps] (重音在第一个音节)。
> 可以近似读作：“C-nap-s” (类似于 "Sina" + "p-s" 的组合音)。

- **第一阶段**: 支持简单的文本数据
- **第二阶段**: 支持非文本数据, 例如截图
- **第三阶段**: 支持文件传输 (例如 Windows 复制文件，Mac 上粘贴到 Finder，通过后端服务传输)

# Features

## Cross Platform
- **Web**: desktop/mobile/PWA (Dioxus)
- **PC**: Windows/macOS/Linux (Tauri 封装)
- **Mobile**: Android/iOS/iPad
- **Server**: Docker

## Personal App
App 主要面向个人用户。

## Authentication
支持 SSO 登陆。

## Sync UI
用户在多端使用时，UI 实时同步。

- **Synced Page Router**:
  用户的路由记录在 Spacetime DB，多端同步。
- **Synced Input**:
  用户当前在输入框的输入内容记录在 Spacetime DB，多端同步。

## Single Binary
整个工具后端就是一个 Single Binary。
这个 Single Binary 打包了前端的 Static Site，Web 版本可以直接访问这个 Server。

# Architecture

整体架构分后端、前端、数据库三大块。

## Backend
后端使用 Rust 开发 (`axum`).

## Frontend
前端使用 Dioxus 开发。

## Database
数据库使用 Spacetime DB。
这个数据库存储包括应用数据和 UI 相关的变量数据。

当数据发生变化时，所有设备上的同一个账号的 App 都会收到事件并执行对应的更新。例如 UI 的路由信息发生了变化，同步到所有设备上之后，所有设备上的路由都跟着更新。

# Development

## Prerequisites

- **[Rust](https://www.rust-lang.org/tools/install)**
- **[Dioxus CLI](https://dioxuslabs.com/learn/0.6/getting_started/cli)**
  ```bash
  cargo install dioxus-cli
  ```

## Build & Run

### Development Mode

Run the frontend and backend in separate terminals for hot-reloading.

**1. Frontend**
```bash
cd frontend
dx serve
```
This typically runs on port 8080 (or as configured).

**2. Backend**
```bash
cd backend
cargo run
```
The backend server listens on `0.0.0.0:3000` (default) and serves the API.

### Production (Single Binary)

To test the "Single Binary" deployment where the Rust backend serves the frontend assets:

**1. Build Frontend Assets**
```bash
cd frontend
dx build --release
```
This builds the web version and places artifacts in `target/dx/frontend/release/web/public`.

**2. Run Backend**
```bash
cd backend
cargo run --release
```
The backend will embed the files from the frontend build. You can access the full app at `http://localhost:3000`.