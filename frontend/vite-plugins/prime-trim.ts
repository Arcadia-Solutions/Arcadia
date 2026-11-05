// Vite plugin that shrinks two global stylesheets we import whole:
// - PrimeFlex ships thousands of utility classes but we only use a handful. Every class-like token
//   found in src/ and index.html is collected, and PrimeFlex rules whose selectors reference none of
//   them are dropped (rules without class selectors are always kept). On HMR, only the changed file
//   is rescanned and its tokens are added to the set (removed tokens stay until the server restarts).
// - PrimeIcons declares its font in several formats (eot, ttf, svg, woff, woff2). Every browser we
//   support reads woff2, so the @font-face is rewritten to that single source.
// The token scan is a superset match (any word in the code counts as "used"), so it can keep
// unused rules but never removes a class that is referenced, as long as it is not built from a
// runtime string concatenation.

import { globSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import type { Plugin } from 'vite'

// Matches the resolved id of PrimeFlex / PrimeIcons stylesheets (ignoring query suffix).
const PRIMEFLEX_CSS = /[\\/]primeflex[\\/]primeflex\.css$/
const PRIMEICONS_CSS = /[\\/]primeicons[\\/]primeicons\.css$/

// Pull class tokens out of a selector, preserving escaped characters (e.g. `.md\:flex` -> `md:flex`).
const selectorClassTokens = (selector: string): string[] => [...selector.matchAll(/\.((?:[A-Za-z0-9_-]|\\.)+)/g)].map((m) => m[1].replace(/\\(.)/g, '$1'))

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

  const addTokens = (code: string) => {
    for (const m of code.matchAll(/[A-Za-z0-9_:/-]+/g)) usedClasses.add(m[0])
  }

  const scan = () => {
    usedClasses = new Set<string>()
    for (const file of globSync(['src/**/*.{vue,ts,js}', 'index.html'], { cwd: root })) addTokens(readFileSync(resolve(root, file), 'utf8'))
  }

  return {
    name: 'arcadia-prime-trim',
    enforce: 'pre',
    configResolved(config) {
      root = config.root
      scan()
    },
    async handleHotUpdate(ctx) {
      if (!primeflexId || !/\.(vue|ts|js|html)$/.test(ctx.file)) return
      const before = usedClasses.size
      addTokens(await ctx.read())
      if (usedClasses.size === before) return
      const mod = ctx.server.moduleGraph.getModuleById(primeflexId)
      if (!mod) return
      ctx.server.moduleGraph.invalidateModule(mod)
      // the returned list replaces the modules Vite updates, so the edited file's own modules must stay in it
      return [...ctx.modules, mod]
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
