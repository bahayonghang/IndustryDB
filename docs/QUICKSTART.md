# 文档快速启动指南

## 🚀 快速开始（推荐使用 just）

### 1. 进入文档目录
```bash
cd docs
```

### 2. 安装依赖
```bash
just start
```

这将安装 npm 依赖。

### 3. 启动开发服务器
```bash
# 启动开发服务器（中文，默认）
just dev

# 启动开发服务器（英文）
just dev-en

# 构建生产版本
just build

# 预览构建结果
just preview

# 清理构建产物
just clean
```

## 📦 使用 npm（如果没有 just）

### 1. 安装 just（推荐）

**macOS / Linux:**
```bash
cargo install just
# 或
brew install just
```

**Windows:**
```powershell
cargo install just
# 或
scoop install just
```

### 2. 或直接使用 npm

```bash
# 进入文档目录
cd docs

# 安装依赖
npm install

# 启动开发服务器
npm run docs:dev

# 构建
npm run docs:build

# 预览
npm run docs:preview
```

## 📝 编辑文档

### 中文文档（默认）
```
docs/
├── index.md              # 首页
├── guide/                # 指南
│   ├── what-is-industrydb.md
│   ├── getting-started.md
│   └── features.md
├── api/                  # API 文档
└── examples/             # 示例
```

### 英文文档
```
docs/en/
├── index.md              # 首页
├── guide/                # 指南
│   ├── what-is-industrydb.md
│   ├── getting-started.md
│   └── features.md
├── api/                  # API 文档
└── examples/             # 示例
```

### 热重载

文档服务器支持热重载：
- 保存文件后自动刷新浏览器
- 实时预览更改
- 无需重启服务器

## 🎨 自定义

### 修改主题配置

编辑 `.vitepress/config.ts`:
```typescript
export default defineConfig({
  title: "IndustryDB",  // 修改标题
  description: "...",   // 修改描述
  // ...
})
```

### 修改导航栏

编辑 `.vitepress/en.ts` 或 `.vitepress/zh.ts`:
```typescript
function nav(): DefaultTheme.NavItem[] {
  return [
    { text: 'Guide', link: '/guide/what-is-industrydb' },
    // 添加更多导航项
  ]
}
```

### 修改侧边栏

同样在 `en.ts` 或 `zh.ts` 中的 `sidebar()` 函数。

## 🔧 故障排除

### 依赖安装失败

```bash
# 清理并重新安装
cd docs
rm -rf node_modules package-lock.json
npm install
```

### 端口被占用

如果 5173 端口被占用：
```bash
# 使用不同端口
npm run docs:dev -- --port 3000
```

### 构建失败

```bash
# 清理缓存
just clean
# 重新构建
just build
```

## 📚 学习资源

- [VitePress 官方文档](https://vitepress.dev/)
- [Markdown 扩展语法](https://vitepress.dev/guide/markdown)
- [主题配置](https://vitepress.dev/reference/default-theme-config)

## 🆘 需要帮助？

- 查看 `docs/README.md` 了解详细说明
- 运行 `just help` 查看所有命令
- 查看 GitHub Issues

---

**提示**: 建议使用 `just` 命令，它提供了更简洁的开发体验！
