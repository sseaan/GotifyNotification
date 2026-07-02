# Nexthrum - 开发指南

## 环境要求

### 必需安装

| 工具 | 版本 | 说明 |
|------|------|------|
| Node.js | 20.x LTS | JavaScript 运行时 |
| Rust | stable (1.75+) | 通过 [rustup](https://rustup.rs/) 安装 |
| Microsoft Visual Studio Build Tools | 2022 | C++ 编译工具链（Windows 必需） |
| WebView2 | 预装 | Windows 10 1809+ / Windows 11 预装 |

### 推荐安装

| 工具 | 用途 |
|------|------|
| VS Code + Tauri Extension | IDE |
| Git | 版本管理 |

---

## 快速开始

### 1. 克隆并安装依赖

```bash
cd Nexthrum
npm install
```

### 2. 开发模式

```bash
npm run tauri dev
```

这会同时启动：
- Vite 开发服务器（前端热更新，端口 1420）
- Tauri 应用窗口（Rust 后端 + WebView）

### 3. 生产构建

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`：
- `msi/` / `nsis/` - Windows 安装包
- 可执行文件在 `src-tauri/target/release/nexthrum.exe`

### 4. 仅构建前端（调试用）

```bash
npm run build
```

前端构建产物位于 `dist/`。

---

## 开发工作流

### 目录导航

```
项目根目录
├── src/              ← 前端开发主要在这里
├── src-tauri/src/    ← Rust 后端开发在这里
├── docs/             ← 项目文档（需求/设计/技术/架构）
└── devlog/           ← 每日开发日志
```

### 日常开发步骤

1. **查看 devlog**：`devlog/` 文件夹，按 `YYYY-MM-DD.md` 命名，记录每日完成事项和 TODO
2. **修改前端**：编辑 `src/` 下的 `.tsx` / `.ts` / `.css` 文件，Vite 自动热更新
3. **修改后端**：编辑 `src-tauri/src/` 下的 `.rs` 文件，`cargo check` 检查编译
4. **添加新依赖（前端）**：`npm install <package>`
5. **添加新依赖（Rust）**：编辑 `src-tauri/Cargo.toml`，然后 `cargo update`
6. **提交代码**：遵循 [Git 提交规范](#git-提交规范)

### 调试技巧

- **前端调试**：`npm run tauri dev` 后，在 Tauri 窗口中按 `F12` 打开 DevTools
- **后端日志**：Rust 端使用 `log` crate，日志输出到终端
- **数据库检查**：SQLite 数据库位于 `%APPDATA%/Nexthrum/gotify_messages.db`，可用 [DB Browser for SQLite](https://sqlitebrowser.org/) 打开查看

---

## 代码规范

### TypeScript

- 使用 `const` 和箭头函数
- 接口命名：`I` 前缀或描述性名称（如 `AppSettings`, `GotifyMessage`）
- 组件命名：PascalCase（如 `MessageWindow`, `AppFilter`）
- 文件命名：与默认导出组件同名

### Rust

- 遵循 `rustfmt` 默认格式
- 使用 `#[derive(Debug, Clone, Serialize, Deserialize)]` 标记数据结构
- 错误处理：Command 返回 `Result<T, String>`，内部函数可返回具体错误类型
- 使用 `log::info! / warn! / error!` 分级日志

### CSS / Tailwind

- 自定义颜色通过 CSS 变量定义在 `src/styles/index.css`
- 优先使用 Tailwind 工具类，仅在必要时自定义 CSS
- 使用 `transition-colors` 做悬停过渡

### 文件组织

- 一个组件一个文件
- 紧耦合的子组件放在同一目录下（如 `MessageWindow/` 目录含多个子组件）
- 共享的类型定义放在 `types/`
- 工具函数放在 `utils/`

---

## Git 提交规范

```
<type>(<scope>): <subject>

<body>
```

**Type**：
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档变更
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试
- `chore`: 构建/工具变更

**示例**：
```
feat(websocket): add auto-reconnect on connection loss

Automatically attempt to reconnect every 5 seconds when the
WebSocket connection drops.
```

---

## GitHub Actions CI/CD

工作流文件：`.github/workflows/build.yml`

**触发条件**：
- Push 到 `main` / `master` 分支
- 推送 `v*` 标签
- Pull Request 到主分支
- 手动触发（`workflow_dispatch`）

**构建矩阵**：
- `windows-latest` / `x86_64-pc-windows-msvc`

**产物**：
- NSIS 安装包 (.exe)
- 便携版 (.zip)
- 自动创建 GitHub Release（推送 tag 时）
