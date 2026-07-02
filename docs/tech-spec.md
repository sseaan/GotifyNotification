# Nexthrum - 技术规范

## 技术选型理由

### Tauri 2.0

- 相比 Electron，包体积更小（~5MB vs ~150MB）
- 使用系统原生 WebView2（Windows 10+ 预装）
- Rust 后端性能优异，内存占用低
- 原生系统托盘和通知支持

### React 18 + TypeScript

- TypeScript 提供类型安全，减少运行时错误
- React 生态成熟，组件化开发效率高

### Zustand

- 比 Redux 轻量，API 简洁
- 无需 Provider 包裹，直接在组件中使用
- 支持 TypeScript 类型推导

### Tailwind CSS

- 原子化 CSS，无需命名规范
- 构建时 tree-shaking，最终 CSS 极小
- 暗色主题开箱即用

### SQLite (rusqlite bundled)

- 嵌入式数据库，无需额外安装
- `bundled` feature 从源码编译 SQLite，无外部依赖
- 适合本地消息持久化场景

---

## 项目结构

```
GotifyNotification/
├── src/                          # React 前端源码
│   ├── main.tsx                  # 入口，挂载 React
│   ├── App.tsx                   # 根组件，路由切换与初始化
│   ├── vite-env.d.ts             # Vite 类型声明
│   ├── styles/
│   │   └── index.css             # Tailwind 指令 + 全局样式 + CSS 变量
│   ├── types/
│   │   └── index.ts              # TypeScript 类型定义
│   ├── store/
│   │   └── useAppStore.ts        # Zustand 全局状态
│   ├── hooks/
│   │   └── useTauriEvents.ts     # Tauri 事件监听 Hook
│   ├── utils/
│   │   └── format.ts             # 日期/文本格式化工具
│   └── components/
│       ├── MessageWindow/
│       │   ├── MessageWindow.tsx   # 消息窗口容器
│       │   ├── Toolbar.tsx         # 顶部工具栏
│       │   ├── AppFilter.tsx       # App 来源筛选栏
│       │   ├── MessageList.tsx     # 消息列表
│       │   └── MessageItem.tsx     # 单条消息组件
│       ├── SettingsWindow/
│       │   └── SettingsWindow.tsx  # 设置面板
│       └── common/                 # 公共组件（预留）
│
├── src-tauri/                    # Tauri/Rust 后端源码
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置
│   ├── build.rs                  # 构建脚本
│   ├── capabilities/
│   │   └── default.json          # 权限声明
│   ├── icons/                    # 应用图标
│   └── src/
│       ├── main.rs               # Rust 入口
│       ├── lib.rs                # Tauri 核心：命令、状态、setup
│       ├── websocket.rs          # WebSocket 连接管理
│       ├── storage.rs            # SQLite 存储 + 设置持久化
│       └── tray.rs               # 系统托盘
│
├── docs/                         # 项目文档
│   ├── requirements.md           # 开发需求
│   ├── tech-spec.md              # 技术规范（本文件）
│   ├── design-spec.md            # 设计规范
│   ├── architecture.md           # 架构设计
│   └── development-guide.md      # 开发指南
│
├── devlog/                       # 开发日志
│   └── YYYY-MM-DD.md
│
├── .github/workflows/
│   └── build.yml                 # CI/CD 构建流水线
│
├── package.json                  # Node 依赖与脚本
├── tsconfig.json                 # TypeScript 配置
├── vite.config.ts                # Vite 构建配置
├── tailwind.config.js            # Tailwind 配置
├── postcss.config.js             # PostCSS 配置
├── index.html                    # HTML 入口
├── AGENTS.md                     # Agent 工作指引
├── .gitignore
├── LICENSE
└── README.md
```

---

## 数据流架构

```
┌──────────────┐     WebSocket      ┌──────────────┐
│ Gotify       │ ◄──────────────► │ Rust Backend │
│ Server       │   ws://.../stream │ (tokio-      │
└──────────────┘                    │  tungstenite)│
                                    └───┬──────┬───┘
                                        │      │
                                   emit │      │ SQLite
                                  event │      │ (rusqlite)
                                        │      │
                                    ┌───▼──────▼───┐
                                    │ React 前端   │
                                    │ (Zustand     │
                                    │  Store)      │
                                    └──────────────┘
```

1. **连接建立**：Rust 后端读取设置中的 `server_url` 和 `client_token`，通过 `tokio-tungstenite` 建立 WebSocket 连接到 Gotify `/stream?token=xxx`
2. **消息接收**：WebSocket 收到 JSON 消息 → 存入 SQLite → 通过 `app.emit("new-message", msg)` 推送给前端 → 前端 Zustand store 更新列表
3. **通知发送**：Rust 后端通过 `tauri-plugin-notification` 调用 Windows 原生通知 API
4. **前端操作**（删除/标为已读/清空）→ `invoke("command")` 调 Rust command → Rust 操作 SQLite → 返回结果
5. **设置保存**：前端表单 → `invoke("save_settings")` → Rust 写入 JSON 文件 + 重连 WebSocket

---

## 关键依赖版本

| 依赖 | 版本 |
|------|------|
| tauri | 2.x |
| tauri-build | 2.x |
| tauri-plugin-notification | 2.x |
| tauri-plugin-autostart | 2.x |
| tokio | 1.x (full features) |
| tokio-tungstenite | 0.24.x (native-tls) |
| rusqlite | 0.31.x (bundled) |
| reqwest | 0.12.x |
| serde / serde_json | 1.x |
| chrono | 0.4.x |
| react / react-dom | 18.x |
| zustand | 4.x |
| tailwindcss | 3.x |
| vite | 5.x |
| typescript | 5.x |
