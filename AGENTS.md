# AGENTS.md - Nexthrum 项目工作指引

## 项目概述

Nexthrum 是一款 Windows 桌面应用，连接 Gotify 服务器并通过 Windows 原生通知展示消息。
技术栈：**Tauri 2.0 + React 18 + TypeScript + Rust + SQLite**。

## 标准文档路径

在开始任何开发工作前，请先阅读相关标准文档：

| 文档 | 路径 | 内容 |
|------|------|------|
| 开发需求 | `docs/requirements.md` | 功能需求、用户故事、验收标准 |
| 技术规范 | `docs/tech-spec.md` | 技术栈、项目结构、数据流、依赖版本 |
| 设计规范 | `docs/design-spec.md` | UI 布局、色彩体系、交互规范、组件规格 |
| 架构设计 | `docs/architecture.md` | 系统架构图、模块职责、数据持久化、通信协议 |
| 开发指南 | `docs/development-guide.md` | 环境搭建、快速开始、开发流程、代码规范、Git 规范 |
| 开发日志 | `devlog/YYYY-MM-DD.md` | 每日开发记录（完成事项 + 待办事项） |

## 工作原则

### 1. 文档先行

- 修改功能前，先更新对应的标准文档
- 新功能必须在 `docs/requirements.md` 中有对应条目
- API 变更必须在 `docs/architecture.md` 中更新通信协议表

### 2. 代码风格

- **TypeScript**：严格模式，函数组件 + Hooks，Zustand 管理全局状态
- **Rust**：`rustfmt` 标准格式，`log` crate 分级日志，`#[tauri::command]` 暴露接口
- **CSS**：Tailwind 工具类优先，自定义颜色使用 CSS 变量（定义在 `src/styles/index.css`）

### 3. 开发流程

1. **开始工作前**：
   - 查看 `devlog/` 最新日志了解当前进度
   - 阅读相关标准文档理解上下文

2. **开发中**：
   - 前端代码放在 `src/` 对应目录
   - 后端代码放在 `src-tauri/src/` 对应模块
   - 新增 Tauri command 需同步更新 `docs/architecture.md` 通信协议表
   - 新增前端组件需遵循 `docs/design-spec.md` 设计规范

3. **完成后**：
   - 运行 `npx tsc --noEmit` 检查前端类型
   - 运行 `cargo check`（需本地 Rust 环境）检查后端编译
   - 更新 `devlog/YYYY-MM-DD.md` 记录完成事项和待办

### 4. Git 提交规范

```
<type>(<scope>): <subject>
```

- `feat`: 新功能
- `fix`: 修复
- `docs`: 文档
- `refactor`: 重构
- `chore`: 构建/工具

### 5. 关键约定

- **数据存储**：SQLite 在 `%APPDATA%/Nexthrum/nexthrum_messages.db`，设置 JSON 在同目录子文件夹
- **事件通信**：后端 → 前端用 `app.emit()`，前端 → 后端用 `invoke()`
- **WebSocket**：由 Rust 后端管理（`websocket.rs`），断线自动重连（5 秒间隔）
- **通知**：通过 `tauri-plugin-notification` 发送 Windows 原生通知
- **窗口**：目前只有单窗口（`message`），消息和设置通过前端视图切换
- **系统托盘**：托盘图标程序化生成，右键菜单含 Message List / Settings / Exit

## 常见任务指引

### 添加新设置项

1. 在 `src/types/index.ts` 的 `AppSettings` 接口中添加字段
2. 在 `src-tauri/src/lib.rs` 的 `AppSettings` struct 中添加字段（含 `Default`）
3. 在 `src/store/useAppStore.ts` 的 `defaultSettings` 中添加默认值
4. 在 `src/components/SettingsWindow/SettingsWindow.tsx` 中添加表单控件
5. 在 `src-tauri/src/storage.rs` 的 `save_settings` / `load_settings` 中（JSON 自动处理）

### 添加新 Tauri Command

1. 在 `src-tauri/src/lib.rs` 中定义 `#[tauri::command]` 函数
2. 在 `generate_handler![]` 宏中注册
3. 在 `docs/architecture.md` 的通信协议表中添加记录
4. 在前端通过 `invoke("command_name", { params })` 调用

### 添加新前端组件

1. 在 `src/components/` 下创建目录和文件
2. 遵循 `docs/design-spec.md` 的色彩和布局规范
3. 使用 Zustand store 获取/更新状态
4. 如需后端交互，通过 `invoke()` 调用 Tauri command

### 调试

- 前端：`npm run tauri dev` 后按 F12 打开 DevTools
- 后端日志：`RUST_LOG=debug npm run tauri dev`
- 数据库：用 DB Browser for SQLite 打开 `%APPDATA%/Nexthrum/nexthrum_messages.db`
