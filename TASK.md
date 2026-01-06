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



# 登录功能
TODO

# 版本
1. 在 build 的时候将版本号注入到 backend/frontend 中
2. 在 server 端的 log 显示当前版本
3. 在 frontend 支持查看当前版本