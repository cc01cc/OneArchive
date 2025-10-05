import fs from 'fs'
import path from 'path'
import { execSync } from 'child_process'
import crypto from 'crypto'
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
      // 清理旧的缓存文件，确保使用最新参数重新生成
      fs.rmSync(outDir, { recursive: true, force: true })
      fs.mkdirSync(outDir, { recursive: true })
      const contentHash = crypto.createHash('md5').update(content).digest('hex')
      const fileBase = `${path.basename(env.filePath || 'unknown', '.md')}-${lang}-${contentHash}`
      const outPath = path.join(outDir, `${fileBase}.svg`)
      if (!fs.existsSync(outPath)) {
        console.log(`生成图表：${fileBase}`)
        try {
          if (lang === 'plantuml') {
            const tmp = path.join(outDir, `${fileBase}.pu`)
            fs.writeFileSync(tmp, content)
            try {
              execSync(`npx plantuml-cli "${tmp}" -tsvg -o "${outDir}" -DPLANTUML_LIMIT_SIZE=16384 -DdefaultFontSize=14 -DdefaultFontName="DejaVu Sans, WenQuanYi Micro Hei, Noto Sans CJK SC, sans-serif" -DminimumWidth=200 -Ddpi=150 -Dnodesep=20 -Dranksep=40 -Dpadding=10`, { stdio: 'inherit' })
              // PlantUML 根据标题生成文件名，检查可能的输出文件
              let actualOutPath = outPath
              if (!fs.existsSync(outPath)) {
                // 尝试查找根据标题命名的文件
                const titleMatch = content.match(/@startuml\s+(.+)/)
                if (titleMatch) {
                  const title = titleMatch[1].trim()
                  const titlePath = path.join(outDir, `${title}.svg`)
                  if (fs.existsSync(titlePath)) {
                    actualOutPath = titlePath
                  }
                }
                // 如果还没找到，查找最新生成的 SVG 文件
                if (!fs.existsSync(actualOutPath)) {
                  const files = fs.readdirSync(outDir)
                    .filter(f => f.endsWith('.svg'))
                    .map(f => ({ name: f, mtime: fs.statSync(path.join(outDir, f)).mtime }))
                    .sort((a, b) => b.mtime.getTime() - a.mtime.getTime())
                  if (files.length > 0) {
                    actualOutPath = path.join(outDir, files[0].name)
                  }
                }
              }
              // 如果找到了文件，重命名到期望的位置
              if (fs.existsSync(actualOutPath) && actualOutPath !== outPath) {
                fs.renameSync(actualOutPath, outPath)
              }
            } catch (error) {
              console.error(`PlantUML 命令执行失败: ${error}`)
            }
            fs.unlinkSync(tmp)
          } else {
            const tmp = path.join(outDir, `${fileBase}.mmd`)
            fs.writeFileSync(tmp, content)
            // 创建临时 Puppeteer 配置文件
            const puppeteerConfig = {
              args: ['--no-sandbox', '--disable-setuid-sandbox']
            }
            const configFile = path.join(outDir, `${fileBase}-config.json`)
            fs.writeFileSync(configFile, JSON.stringify(puppeteerConfig))
            execSync(`npx mmdc -i "${tmp}" -o "${outPath}" -p "${configFile}"`, { stdio: 'inherit' })
            fs.unlinkSync(tmp)
            fs.unlinkSync(configFile)
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