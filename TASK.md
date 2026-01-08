# Motivation
TODO

# 架构设计
## 项目结构
TODO

# 视觉
## Logo 设计
生成 logo.svg 到 VI/logo 目录下, 要求契合功能和项目名称, 尽可能简洁, 并且提供矢量图
注意视觉中心居中
注意上下左右的空白 margin 合理, 确保用于 icon 时能够以合理的大小显示在浏览器的标签, 桌面任务栏, dock 栏等地方

## Web App Icon
根据 logo.svg 生成 favicon.ico
应用到 web app 项目中

## 桌面版 & iOS Icon
根据 logo.svg 生成 macos-app-icon.svg, 要求: 
1. 遵循 Apple Human Interface Guidelines (HIG)
2. 添加一个精致的深色渐变底盘

应用到 windows/linux/macOS/iOS tauri app 项目中, 包括 app icon, tray icon, 安装包 icon 等

## Android App Icon
根据 logo.svg 分离出前景（Logo）和背景（深色底色），确保它支持 Android 的自适应系统
应用到 android tauri app 项目中, 包括 app icon, tray icon, 安装包 icon等

## Icon Generator
把 logo 的生效做成编译时自动生成的效果:
1. 用 rust 写一个工具, 用这个工具来根据 VI/logo/logo.svg 和 VI/logo/macos-app-icon.svg 来生成所有平台实际使用的 logo 到合适的位置, 包括各种尺寸要求等你能想到的需求
2. 生成的前端 icon 放到 frontend/public 目标下
3. 自动生成 mac 的 tray icon 满足 mac 的要求, 将原本的 logo 线条变成纯白色, 并且加粗线条, 背景还是透明的保持和系统其它图标风格一致
4. 在 cargo 编译(包括开发环境和 CI 环境) 之前自动调用这个工具生成所有平台的 icon 到合适的位置, cargo 有这种 build script 机制
5. 为了避免开发时由于 hot-reload 导致无限重启, 这个工具只在检测到关键 icon 缺失时才生成 icons

# 登录功能
TODO

# 版本
1. 在 build 的时候将版本号注入到 backend/frontend 中
2. 在 server 端的 log 显示当前版本
3. 在 frontend 支持查看当前版本

# CI/CD
## 自动构建发布
实现 github action 到 .github/workflows/release.yml, 要求实现:
1. 自动构建以下平台和架构的安装包:
  1.1 Mac OS: aarch64(dmg)
  1.2 Windows: x64(msi 和 setup.exe)
  1.3 Linux: amd64(deb, rpm, AppImage), aarch64(app.tar.gz)

2. Mac 版本签名
在 github action 中设置以下 Secrets 以便 tauri-action 能自动处理签名和公证：
APPLE_CERTIFICATE (Base64 编码的 .p12 证书)
APPLE_CERTIFICATE_PASSWORD
APPLE_SIGNING_IDENTITY
APPLE_ID
APPLE_PASSWORD (App-specific password)

# 更新
