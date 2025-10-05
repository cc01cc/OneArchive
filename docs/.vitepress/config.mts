// docs/.vitepress/config.mts
import { defineConfig } from 'vitepress'
export default defineConfig({
  base: '/OneArchive/',
  title: "OneArchive",
  description: "文件归档分卷工具，支持智能去重、分卷存储和 Reed-Solomon 纠删码数据保护等",
  rewrites: {
    '01-需求分析-00-通用.md': 'requirements/general.md',
    '01-需求分析-01-场景分析.md': 'requirements/scenario-analysis.md',
    '02-开发规范-00-通用开发原则.md': 'development/general-principles.md',
    '02-开发规范-01-前端规范.md': 'development/frontend-guide.md',
    '02-开发规范-02-Rust开发指南.md': 'development/rust-guide.md',
    '02-开发规范-03-API设计规范.md': 'development/api-design.md',
    '02-开发规范-04-文档规范.md': 'development/docs-guide.md',
    '03-详细设计-00-通用.md': 'design/general.md',
    '03-详细设计-01-数据库设计.md': 'design/database-design.md',
    '03-详细设计-02-场景设计.md': 'design/scenario-design.md',
    '03-详细设计-03-数据记录状态设计.md': 'design/status-design.md',
    '03-详细设计-04-灾备方案设计.md': 'design/disaster-recovery.md',
    '04-测试文档.md': 'testing.md',
    'guide/getting-started.md': 'guide/getting-started.md'
  },
  themeConfig: {
    nav: [
      { text: '首页', link: '/' },
      { text: '快速开始', link: '/guide/getting-started' },
      { text: '需求分析', link: '/requirements/general' },
      { text: '开发规范', link: '/development/general-principles' },
      { text: '详细设计', link: '/design/general' },
      { text: '测试文档', link: '/testing' }
    ],
    sidebar: [
      {
        text: '指南',
        items: [
          { text: '快速开始', link: '/guide/getting-started' }
        ]
      },
      {
        text: '需求分析',
        items: [
          { text: '通用', link: '/requirements/general' },
          { text: '场景分析', link: '/requirements/scenario-analysis' }
        ]
      },
      {
        text: '开发规范',
        items: [
          { text: '通用开发原则', link: '/development/general-principles' },
          { text: '前端规范', link: '/development/frontend-guide' },
          { text: 'Rust开发指南', link: '/development/rust-guide' },
          { text: 'API设计规范', link: '/development/api-design' },
          { text: '文档规范', link: '/development/docs-guide' }
        ]
      },
      {
        text: '详细设计',
        items: [
          { text: '通用', link: '/design/general' },
          { text: '数据库设计', link: '/design/database-design' },
          { text: '场景设计', link: '/design/scenario-design' },
          { text: '数据记录状态设计', link: '/design/status-design' },
          { text: '灾备方案设计', link: '/design/disaster-recovery' }
        ]
      },
      {
        text: '测试文档',
        items: [
          { text: '测试文档', link: '/testing' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/cc01cc/OneArchive' }
    ]
  },
})
