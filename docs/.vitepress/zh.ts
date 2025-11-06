import { defineConfig, type DefaultTheme } from 'vitepress'

export const zh = defineConfig({
  lang: 'zh-CN',
  description: '由 Rust 和 Polars 驱动的高性能数据库中间件',

  themeConfig: {
    nav: nav(),
    sidebar: sidebar(),

    editLink: {
      pattern: 'https://github.com/yourusername/industrydb/edit/main/docs/:path',
      text: '在 GitHub 上编辑此页'
    },

    footer: {
      message: '基于 MIT 许可发布',
      copyright: 'Copyright © 2024-present IndustryDB Contributors'
    },

    docFooter: {
      prev: '上一页',
      next: '下一页'
    },

    outline: {
      label: '页面导航'
    },

    lastUpdated: {
      text: '最后更新于',
      formatOptions: {
        dateStyle: 'short',
        timeStyle: 'short'
      }
    },

    returnToTopLabel: '回到顶部',
    sidebarMenuLabel: '菜单',
    darkModeSwitchLabel: '主题',
    lightModeSwitchTitle: '切换到浅色模式',
    darkModeSwitchTitle: '切换到深色模式'
  }
})

function nav(): DefaultTheme.NavItem[] {
  return [
    {
      text: '指南',
      link: '/guide/what-is-industrydb',
      activeMatch: '/guide/'
    },
    {
      text: 'API 参考',
      link: '/api/connection',
      activeMatch: '/api/'
    },
    {
      text: '示例',
      link: '/examples/quick-start',
      activeMatch: '/examples/'
    },
    {
      text: '关于',
      items: [
        { text: '架构设计', link: '/zh/about/architecture' },
        { text: '贡献指南', link: '/zh/about/contributing' },
        { text: '更新日志', link: '/zh/about/changelog' }
      ]
    }
  ]
}

function sidebar(): DefaultTheme.Sidebar {
  return {
    '/guide/': {
      base: '/guide/',
      items: [
        {
          text: '简介',
          collapsed: false,
          items: [
            { text: '什么是 IndustryDB？', link: 'what-is-industrydb' },
            { text: '快速开始', link: 'getting-started' },
            { text: '特性', link: 'features' }
          ]
        },
        {
          text: '核心概念',
          collapsed: false,
          items: [
            { text: '连接管理', link: 'connection' },
            { text: '配置', link: 'configuration' },
            { text: 'CRUD 操作', link: 'crud-operations' },
            { text: 'DataFrame 集成', link: 'dataframe' }
          ]
        },
        {
          text: '数据库支持',
          collapsed: false,
          items: [
            { text: 'PostgreSQL', link: 'postgres' },
            { text: 'SQLite', link: 'sqlite' },
            { text: 'MSSQL', link: 'mssql' }
          ]
        },
        {
          text: '高级功能',
          collapsed: false,
          items: [
            { text: '性能优化', link: 'performance' },
            { text: '错误处理', link: 'error-handling' },
            { text: '类型安全', link: 'type-safety' }
          ]
        }
      ]
    },
    '/api/': {
      base: '/api/',
      items: [
        {
          text: 'API 参考',
          items: [
            { text: 'Connection', link: 'connection' },
            { text: 'DatabaseConfig', link: 'config' },
            { text: 'CRUD 方法', link: 'crud' },
            { text: '工具函数', link: 'utilities' },
            { text: '异常类型', link: 'exceptions' }
          ]
        }
      ]
    },
    '/examples/': {
      base: '/examples/',
      items: [
        {
          text: '示例',
          items: [
            { text: '快速开始', link: 'quick-start' },
            { text: '配置示例', link: 'configuration' },
            { text: '基础 CRUD', link: 'basic-crud' },
            { text: '高级查询', link: 'advanced-queries' },
            { text: 'DataFrame 操作', link: 'dataframe-operations' },
            { text: '多数据库', link: 'multiple-databases' }
          ]
        }
      ]
    },
    '/about/': {
      base: '/about/',
      items: [
        {
          text: '关于',
          items: [
            { text: '架构设计', link: 'architecture' },
            { text: '贡献指南', link: 'contributing' },
            { text: '更新日志', link: 'changelog' },
            { text: '许可证', link: 'license' }
          ]
        }
      ]
    }
  }
}

export const search: DefaultTheme.AlgoliaSearchOptions['locales'] = {
  zh: {
    placeholder: '搜索文档',
    translations: {
      button: {
        buttonText: '搜索文档',
        buttonAriaLabel: '搜索文档'
      },
      modal: {
        searchBox: {
          resetButtonTitle: '清除查询条件',
          resetButtonAriaLabel: '清除查询条件',
          cancelButtonText: '取消',
          cancelButtonAriaLabel: '取消'
        },
        startScreen: {
          recentSearchesTitle: '搜索历史',
          noRecentSearchesText: '没有搜索历史',
          saveRecentSearchButtonTitle: '保存至搜索历史',
          removeRecentSearchButtonTitle: '从搜索历史中移除',
          favoriteSearchesTitle: '收藏',
          removeFavoriteSearchButtonTitle: '从收藏中移除'
        },
        errorScreen: {
          titleText: '无法获取结果',
          helpText: '你可能需要检查你的网络连接'
        },
        footer: {
          selectText: '选择',
          navigateText: '切换',
          closeText: '关闭'
        },
        noResultsScreen: {
          noResultsText: '无法找到相关结果',
          suggestedQueryText: '你可以尝试查询',
          reportMissingResultsText: '你认为该查询应该有结果？',
          reportMissingResultsLinkText: '点击反馈'
        }
      }
    }
  }
}
