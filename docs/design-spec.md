# GotifyNotification - 设计规范

## 设计理念

- **暗色主题为主**：消息通知类应用适合暗色界面，减少视觉干扰
- **极简风格**：功能聚焦，不冗余
- **Windows 11 风格**：圆角、柔和阴影、流畅过渡动画

## 色彩体系

通过 CSS 变量定义，位于 `src/styles/index.css`：

| 变量名 | 色值 | 用途 |
|--------|------|------|
| `--color-bg` | `#1a1a2e` | 主背景色 |
| `--color-surface` | `#16213e` | 卡片/面板背景 |
| `--color-surface-hover` | `#1f3460` | 悬停态 |
| `--color-primary` | `#3b82f6` | 主色调（蓝色） |
| `--color-primary-hover` | `#2563eb` | 主色调悬停 |
| `--color-text` | `#e2e8f0` | 主文字色 |
| `--color-text-secondary` | `#94a3b8` | 次要文字色 |
| `--color-border` | `#334155` | 边框色 |
| `--color-danger` | `#ef4444` | 危险操作色 |
| `--color-success` | `#22c55e` | 成功状态色 |

## 布局结构

### 消息窗口

```
┌────────────────────────────────────────┐
│  Gotify Messages  ●(连接状态)  [Clear] [⚙]│  ← Toolbar
├────────────────────────────────────────┤
│  [All Apps] [App1] [App2] [App3]       │  ← AppFilter
├────────────────────────────────────────┤
│  ● 消息标题              3分钟前        │
│  消息内容预览...                         │  ← MessageList
├────────────────────────────────────────┤
│  ○ 消息标题              1小时前        │
│  消息内容预览...                         │
├────────────────────────────────────────┤
│  ○ 消息标题              昨天           │
│  消息内容预览...                         │
└────────────────────────────────────────┘
```

- 窗口默认尺寸：800×600
- 最小尺寸：400×300
- 消息列表可滚动
- App 筛选栏可横向滚动

### 设置窗口

```
┌────────────────────────────────────────┐
│  ← Back    Settings                    │
├────────────────────────────────────────┤
│                                        │
│  SERVER CONNECTION                     │
│  ┌──────────────────────────────────┐  │
│  │ Server URL                       │  │
│  └──────────────────────────────────┘  │
│  ┌──────────────────────────────────┐  │
│  │ Client Token              ****   │  │
│  └──────────────────────────────────┘  │
│  [Test Connection]                     │
│                                        │
│  GENERAL                               │
│  Start with Windows          [toggle]  │
│  Minimize to tray on close   [toggle]  │
│  Message retention (days)    [ 7    ]  │
│                                        │
│  DO NOT DISTURB                        │
│  Notifications suppressed during...    │
│  Start: [22:00]   to   End: [08:00]   │
│                                        │
│  ┌──────────────────────────────────┐  │
│  │         Save Settings            │  │
│  └──────────────────────────────────┘  │
└────────────────────────────────────────┘
```

- 设置面板居中，最大宽度 `max-w-lg`（512px）
- 分区标题使用大写 + 小字

## 交互规范

### 消息项

- **悬停**：背景变 `--color-surface-hover`
- **未读消息**：背景为 `--color-surface`（比普通稍亮），左侧蓝色圆点
- **点击**：复制消息内容到剪贴板；若未读则标记已读
- **右键**：弹出上下文菜单 "Delete Message"

### 按钮

- 主操作按钮：`--color-primary` 背景，白色文字
- 次要按钮：透明背景，悬停时显示边框
- 危险操作按钮：`--color-danger` 背景

### 过渡动画

- 使用 Tailwind `transition-colors`（颜色变化 150ms ease）

## 系统托盘图标

- 程序化生成的 32×32 RGBA 蓝色铃铛图标
- 可扩展为：有未读时叠加红点

## 字体

- 系统默认 sans-serif 字体栈：
  ```
  -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
  Oxygen, Ubuntu, Cantarell, "Fira Sans", "Droid Sans",
  "Helvetica Neue", sans-serif
  ```
- 等宽字体（如需要展示代码/日志）：系统默认 monospace

## 响应式

- 仅桌面端，不做移动端适配
- 最小窗口尺寸 400×300 保证基本可用性
