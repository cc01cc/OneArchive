<!-- docs/.vitepress/theme/Layout.vue -->
<script setup>
import { useData } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import { ref, computed } from 'vue'

const { Layout } = DefaultTheme
const { page, frontmatter } = useData()

// 使用 computed 属性来检测是否为首页，基于 frontmatter 的 layout 属性
const isHome = computed(() => {
    return frontmatter.value.layout === 'home' || page.value.relativePath === 'index.md'
})

const showFooter = ref(true)

// 当 frontmatter 改变时自动更新
showFooter.value = isHome.value
</script>

<template>
    <Layout>
        <template #layout-bottom>
            <div v-if="showFooter" class="fixed-footer">
                <p>基于 Apache 2.0 协议发布</p>
                <p>版权所有 &copy; {{ new Date().getFullYear() }} 郑一弘</p>
            </div>
        </template>
    </Layout>
</template>

<style>
.fixed-footer {
    position: fixed;
    bottom: 0;
    left: 0;
    width: 100%;
    text-align: center;
    padding: 1rem 0;
    border-top: 1px solid var(--vp-c-divider);
    background-color: var(--vp-c-bg);
    z-index: 100;
    font-size: 1rem;
    color: var(--vp-c-text-2);
    font-family: var(--vp-font-family-base);
    line-height: 1.6;
}

.fixed-footer p {
    margin: 0.2rem 0;
    font-size: 0.9em;
    color: var(--vp-c-text-2);
    font-family: var(--vp-font-family-base);
}

.VPHome {
    padding-bottom: 70px;
}

.VPDoc.has-aside .content-container {
    padding-bottom: 2rem;
}
</style>