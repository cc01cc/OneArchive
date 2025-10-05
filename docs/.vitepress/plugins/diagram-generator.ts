import fs from 'fs'
import path from 'path'
import { execSync } from 'child_process'
import type MarkdownIt from 'markdown-it'

export default function diagramGenerator(md: MarkdownIt) {
  const defaultFence = md.renderer.rules.fence!
  md.renderer.rules.fence = (tokens, idx, options, env, self) => {
    const token = tokens[idx]
    const lang = token.info.trim()
    if (lang === 'plantuml' || lang === 'mermaid') {
      const content = token.content
      const outDir = path.resolve(process.cwd(), 'public/diagrams')
      fs.mkdirSync(outDir, { recursive: true })
      const fileBase = `${path.basename(env.filePath || 'unknown', '.md')}-${lang}-${Buffer.from(content).toString('base64url').slice(0, 10)}`
      const outPath = path.join(outDir, `${fileBase}.svg`)
      if (!fs.existsSync(outPath)) {
        console.log(`生成图表：${fileBase}`)
        try {
          if (lang === 'plantuml') {
            const tmp = path.join(outDir, `${fileBase}.pu`)
            fs.writeFileSync(tmp, content)
            execSync(`npx plantuml-cli -tsvg "${tmp}"`, { stdio: 'inherit' })
            fs.unlinkSync(tmp)
          } else {
            const tmp = path.join(outDir, `${fileBase}.mmd`)
            fs.writeFileSync(tmp, content)
            execSync(`npx mmdc -i "${tmp}" -o "${outPath}" --no-sandbox`, { stdio: 'inherit' })
            fs.unlinkSync(tmp)
          }
        } catch (error) {
          console.error(`生成图表失败 ${fileBase}:`, error)
          return `<pre><code>${content}</code></pre>` // 失败时保留原始代码块
        }
      }
      const svgContent = fs.readFileSync(outPath, 'utf-8')
      const base64 = Buffer.from(svgContent).toString('base64')
      return `<img src="data:image/svg+xml;base64,${base64}" alt="diagram">`
    }
    return defaultFence(tokens, idx, options, env, self)
  }
}