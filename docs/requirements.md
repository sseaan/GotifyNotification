# Nexthrum - 开发需求文档

## 项目概述

Nexthrum 是一款 Windows 桌面应用程序，用于连接 Gotify 消息推送服务器，通过 Windows 原生通知展示收到的消息。

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 前端 | React 18 + TypeScript |
| 状态管理 | Zustand |
| 样式方案 | Tailwind CSS |
| 后端语言 | Rust |
| 数据库 | SQLite (rusqlite, bundled) |
| WebSocket | tokio-tungstenite |
| 平台 | Windows x64 |
| CI/CD | GitHub Actions |

## 核心功能

### 1. 服务器连接

- 支持连接单个 Gotify 服务器
- 通过 WebSocket 实时推送消息
- 支持在设置中配置服务器 URL 和客户端 Token
- 提供"测试连接"按钮验证配置

### 2. 消息管理

- 按时间倒序显示消息列表
- 按 App 来源筛选消息
- 点击消息可复制内容到剪贴板
- 右键菜单可删除单条消息
- "Clear All"按钮可清空本地所有消息
  - 可选同步删除服务器端消息（默认仅清除本地）

### 3. 消息优先级

- 不做差异化处理，所有消息一视同仁

### 4. 历史记录

- 本地 SQLite 持久化存储
- 可配置保留天数（默认 7 天）
- 每小时自动清理过期消息

### 5. 系统托盘

- 托盘图标常驻
- 有未读消息时显示红点提示（无数字角标）
- 右键菜单：Message List / Settings / Exit
- 左键单击打开消息窗口

### 6. Windows 原生通知

- 收到新消息时弹出 Windows 通知
- 免打扰时段内不弹出通知（消息仍保存）
- 点击通知打开主窗口

### 7. 窗口管理

- **消息窗口**：App 分类筛选 + 消息列表 + 清除按钮
- **设置窗口**：与消息窗口在同一窗口内切换显示
- 关闭窗口时最小化到托盘（可配置）

### 8. 设置项

| 设置项 | 说明 | 默认值 |
|--------|------|--------|
| Server URL | Gotify 服务器地址 | 空 |
| Client Token | 客户端认证令牌 | 空 |
| Auto Start | 开机自启 | false |
| Minimize to Tray | 关闭时最小化到托盘 | true |
| History Retention | 消息保留天数 | 7 |
| DND Start | 免打扰开始时间 | 22:00 |
| DND End | 免打扰结束时间 | 08:00 |

### 9. 构建产物

- `.exe` NSIS 安装包
- `.zip` 便携版

## 未来可能扩展（当前不在范围内）

- 多服务器支持
- 基于优先级的通知样式区分
- 消息搜索功能
- 消息内容富文本渲染
