# 前端（Vue 3 + TypeScript + Pinia + PrimeVue v4 + Tailwind CSS v4）规范

## 技术栈使用原则

- **必须使用 TypeScript**，禁止使用 JavaScript。启用 `strict: true`。
- 前端使用的是 pnpm 作为包管理器
- 所有组件 props、emits、事件回调、Pinia store 的 state/getters/actions 必须有**精确的类型定义**。
- 优先使用 **PrimeVue v4 组件**（如 `Button`, `DataTable`, `Dialog`）及其内置功能（如 `pt` prop、slots、theming）。
- 样式定制优先通过 PrimeVue 的 **`pt`（Pass Through）属性**或 **Tailwind class 覆盖**实现，而非直接写 CSS。
- 若需深度定制，可使用 Tailwind CSS v4 封装原子组件，但**不得重复造轮子**（如已有 PrimeVue Button，就不要自己写 button）。

## 样式与集成

- 确保 `tailwind.config.js` 中启用了 `cssLayer` 选项以兼容 PrimeVue styled 模式。
- 安装并配置 `tailwindcss-primeui` 插件以获得最佳集成体验。
- Tailwind 仅用于布局、间距、响应式等通用样式；组件内部结构应由 PrimeVue 控制。

## 项目结构

- `composables/`：存放组合式函数（如 `useAuth`, `useTauriCommand`）
- `stores/`：Pinia store，每个 store 对应一个业务域
- `types/`：全局 TypeScript 接口和类型
- `api/`：Tauri 命令调用封装（按模块划分）
- `components/`：业务组件，优先组合 PrimeVue 原子组件

## 类型安全

- 禁止使用 `any`、`unknown`（除非必要且有类型守卫）
- 所有 API 响应（Tauri invoke 返回值）必须定义接口
- 使用 `defineProps<{ ... }>()` 和 `defineEmits<{ ... }>()` 显式声明

## 关联外链

> 以下 URL 链接可以获取更加详细的内容

- <https://primevue.org/setup/>
- <https://cn.vuejs.org/guide/introduction.html>
- <https://pinia.vuejs.org/zh/core-concepts/>
