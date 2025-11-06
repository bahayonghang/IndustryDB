# IndustryDB 文档

IndustryDB 的官方文档，使用 VitePress 构建，支持中英文双语。

## 快速开始

### 安装依赖

```bash
cd docs
npm install
```

或使用 just：

```bash
cd docs
just install
```

### 开发

启动开发服务器：

```bash
# 使用 npm
npm run docs:dev

# 使用 just（推荐）
just dev        # 中文版
just dev-en     # 英文版
```

访问 http://localhost:5173 查看文档。

### 构建

构建生产版本：

```bash
# 使用 npm
npm run docs:build

# 使用 just
just build
```

构建产物位于 `.vitepress/dist/` 目录。

### 预览

预览构建结果：

```bash
# 使用 npm
npm run docs:preview

# 使用 just
just preview
```

## 文档结构

```
docs/
├── .vitepress/          # VitePress 配置
│   ├── config.ts        # 主配置文件
│   ├── en.ts           # 英文配置
│   └── zh.ts           # 中文配置
├── index.md            # 中文首页（默认）
├── guide/              # 中文指南
│   ├── what-is-industrydb.md
│   ├── getting-started.md
│   └── features.md
├── api/                # 中文 API 文档
├── examples/           # 中文示例
├── en/                 # 英文版本
│   ├── index.md        # 英文首页
│   ├── guide/          # 英文指南
│   ├── api/            # 英文 API 文档
│   └── examples/       # 英文示例
├── public/             # 静态资源
│   └── logo.svg
├── package.json        # npm 配置
└── justfile           # just 任务定义
```

## Just 命令

使用 `just --list` 查看所有可用命令：

```bash
just --list
```

常用命令：

| 命令 | 说明 |
|------|------|
| `just install` | 安装依赖 |
| `just start` | 安装依赖（首次运行） |
| `just dev` | 启动开发服务器（中文，默认） |
| `just dev-en` | 启动开发服务器（英文） |
| `just build` | 构建生产版本 |
| `just preview` | 预览构建结果 |
| `just clean` | 清理构建产物 |
| `just release` | 完整构建流程 |
| `just deploy-prep` | 准备部署包 |

## 编写文档

### Markdown 扩展

VitePress 支持丰富的 Markdown 扩展：

- [自定义容器](https://vitepress.dev/guide/markdown#custom-containers)
- [代码块](https://vitepress.dev/guide/markdown#syntax-highlighting-in-code-blocks)
- [代码组](https://vitepress.dev/guide/markdown#code-groups)
- [表格](https://vitepress.dev/guide/markdown#github-style-tables)
- 等等

### 示例

#### 代码组

```markdown
::: code-group
```python [示例1]
print("Hello")
```

```python [示例2]
print("World")
```
:::
```

#### 自定义容器

```markdown
::: tip 提示
这是一个提示
:::

::: warning 警告
这是一个警告
:::

::: danger 危险
这是一个危险警告
:::
```

## 国际化

文档支持中英文双语：

- **中文（默认）**：位于 `docs/` 根目录
- **英文**：位于 `docs/en/` 目录

语言切换器会自动显示在导航栏。

## 部署

构建后的文档可以部署到任何静态网站托管服务：

- GitHub Pages
- Netlify
- Vercel
- Cloudflare Pages
- 等等

### GitHub Pages 示例

```yaml
# .github/workflows/docs.yml
name: Deploy Docs

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 18
      - run: cd docs && npm install
      - run: cd docs && npm run docs:build
      - uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: docs/.vitepress/dist
```

## 贡献

欢迎贡献文档改进！请确保：

1. 遵循现有文档风格
2. 同时更新中英文版本
3. 运行 `just build` 确保构建成功
4. 检查链接和格式

## 许可证

MIT License
