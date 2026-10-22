import { globSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import type { Plugin } from 'vite'

// Matches the resolved id of PrimeFlex / PrimeIcons stylesheets (ignoring query suffix).
const PRIMEFLEX_CSS = /[\\/]primeflex[\\/]primeflex\.css$/
const PRIMEICONS_CSS = /[\\/]primeicons[\\/]primeicons\.css$/

// Pull class tokens out of a selector, preserving escaped characters (e.g. `.md\:flex` -> `md:flex`).
const selectorClassTokens = (selector: string): string[] =>
  [...selector.matchAll(/\.((?:[A-Za-z0-9_-]|\\.)+)/g)].map((m) => m[1].replace(/\\(.)/g, '$1'))

// Split stylesheet body into `prelude { body }` rules, supporting one level of nesting (e.g. @media).
const splitTopLevelRules = (css: string): { prelude: string; body: string }[] =>
  [...css.matchAll(/([^{}]+)\{((?:[^{}]+|\{[^{}]*\})*)\}/g)].map((m) => ({
    prelude: m[1].trim(),
    body: m[2],
  }))

const keepRule = (selector: string, used: Set<string>): boolean => {
  const tokens = selectorClassTokens(selector)
  return tokens.length === 0 || tokens.some((t) => used.has(t))
}

const buildPrimeflexRule = (rule: { prelude: string; body: string }, used: Set<string>): string => {
  const { prelude, body } = rule
  if (prelude.startsWith('@media') || prelude.startsWith('@supports')) {
    const inner = splitTopLevelRules(body)
      .filter((r) => r.prelude.startsWith('@') || keepRule(r.prelude, used))
      .map((r) => `${r.prelude}{${r.body}}`)
      .join('')
    return inner.trim() ? `${prelude}{${inner}}` : ''
  }
  if (prelude.startsWith('@')) return `${prelude}{${body}}`
  return keepRule(prelude, used) ? `${prelude}{${body}}` : ''
}

const filterPrimeflex = (css: string, used: Set<string>): string =>
  splitTopLevelRules(css.replace(/\/\*[\s\S]*?\*\//g, ''))
    .map((rule) => buildPrimeflexRule(rule, used))
    .join('\n')

const slimPrimeicons = (css: string): string =>
  css.replace(/@font-face\s*\{([^}]*)\}/g, (full, body: string) => {
    const woff2 = body.match(/url\([^)]*\.woff2[^)]*\)/)
    if (!woff2) return full
    const withoutSrc = body.replace(/\s*src:\s*[^;]+;/g, '')
    return `@font-face { src: ${woff2[0]} format('woff2');${withoutSrc}}`
  })

export const primeTrimPlugin = (): Plugin => {
  let usedClasses = new Set<string>()
  let root = ''
  let primeflexId: string | null = null

  const scan = () => {
    const tokens = new Set<string>()
    const files = globSync(['src/**/*.{vue,ts,js}', 'index.html'], { cwd: root })
    for (const file of files) {
      for (const m of readFileSync(resolve(root, file), 'utf8').matchAll(/[A-Za-z0-9_:/-]+/g)) tokens.add(m[0])
    }
    usedClasses = tokens
  }

  return {
    name: 'arcadia-prime-trim',
    enforce: 'pre',
    configResolved(config) {
      root = config.root
      scan()
    },
    handleHotUpdate(ctx) {
      if (!/\.(vue|ts|js|html)$/.test(ctx.file)) return
      scan()
      if (primeflexId) {
        const mod = ctx.server.moduleGraph.getModuleById(primeflexId)
        if (mod) {
          ctx.server.moduleGraph.invalidateModule(mod)
          return [mod]
        }
      }
    },
    load(id) {
      const cleanId = id.split('?')[0]
      if (!PRIMEFLEX_CSS.test(cleanId)) return null
      primeflexId = cleanId
      return filterPrimeflex(readFileSync(cleanId, 'utf8'), usedClasses)
    },
    transform(code, id) {
      const cleanId = id.split('?')[0]
      if (!PRIMEICONS_CSS.test(cleanId)) return null
      return slimPrimeicons(code)
    },
  }
}
