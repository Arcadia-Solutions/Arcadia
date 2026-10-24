import { fileURLToPath, URL } from 'node:url'
import { existsSync, readFileSync, readdirSync, rmSync } from 'node:fs'
import { resolve } from 'node:path'

import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import { parse } from 'yaml'
import sirv from 'sirv'
// import vueDevTools from 'vite-plugin-vue-devtools'

// The whole project is configured by a single config.yml at the root of the repository.
// Its `frontend` section is inlined in the bundle at build time, which means the frontend
// has to be rebuilt after changing it.
const configurationPath = fileURLToPath(new URL('../config.yml', import.meta.url))
const configuration = parse(readFileSync(configurationPath, 'utf8'))

if (!configuration?.frontend) {
  throw new Error(`no 'frontend' section found in ${configurationPath}`)
}

const customIconsPlugin = (): Plugin => {
  const virtualModuleId = 'virtual:custom-icons.css'
  const customIconsDir = resolve(__dirname, 'src/assets/custom-icons')

  const generateCss = (): string => {
    if (!existsSync(customIconsDir)) return ''
    const files = readdirSync(customIconsDir).filter((f) => f.endsWith('.svg'))
    return files
      .map((file) => {
        const name = file.replace(/\.svg$/, '')
        const svgContent = readFileSync(resolve(customIconsDir, file), 'utf8')
        const dataUri = `data:image/svg+xml;utf8,${encodeURIComponent(svgContent)}`
        return `
.pi.pi-${name}:before,
.pi-${name}:before {
  content: "" !important;
  display: inline-block !important;
  width: 1em !important;
  height: 1em !important;
  background-color: currentColor !important;
  -webkit-mask: url("${dataUri}") no-repeat center / contain !important;
  mask: url("${dataUri}") no-repeat center / contain !important;
  vertical-align: -0.125em !important;
}`
      })
      .join('\n')
  }

  return {
    name: 'arcadia-custom-icons',
    enforce: 'pre',
    resolveId(id) {
      if (id === virtualModuleId) return virtualModuleId
    },
    load(id) {
      if (id === virtualModuleId) return generateCss() || '/* no custom icons */'
    },
    configureServer(server) {
      server.watcher.add(customIconsDir)
      server.watcher.on('all', (_event, path) => {
        if (path.startsWith(customIconsDir)) {
          const mod = server.moduleGraph.getModuleById(virtualModuleId)
          if (mod) server.moduleGraph.invalidateModule(mod)
        }
      })
    },
  }
}

const logo = existsSync(new URL('./src/assets/logo.svg', import.meta.url)) ? 'logo.svg' : 'logo.example.svg'

const kiwiDevServer = (): Plugin => ({
  name: 'kiwiirc-dev-server',
  configureServer(server) {
    const kiwiDist = resolve(__dirname, '../kiwiirc/dist')
    if (!existsSync(kiwiDist)) {
      server.middlewares.use('/kiwiirc', (_req, res) => {
        res.setHeader('Content-Type', 'text/html; charset=utf-8')
        res.end(
          '<p style="font-family:system-ui,sans-serif;padding:40px;text-align:center;color:#888;background:#1a1a1a;">' +
            '<strong style="color:#eee;">KiwiIRC Dev Mode:</strong> <code>kiwiirc/dist</code> not found.<br>' +
            'Run <code>npm run kiwi:setup</code> to build & extract KiwiIRC locally.</p>',
        )
      })
      return
    }

    // Live reload workspace config and plugin during development
    const kiwiConfig = existsSync(resolve(__dirname, '../kiwiirc/config.json')) ? 'config.json' : 'config.json.example'

    server.middlewares.use('/kiwiirc', (req, res, next) => {
      if (req.url === '/static/config.json') {
        res.setHeader('Content-Type', 'application/json')
        return res.end(readFileSync(resolve(__dirname, `../kiwiirc/${kiwiConfig}`)))
      }
      if (req.url === '/static/plugins/arcadia-plugin.js') {
        res.setHeader('Content-Type', 'application/javascript')
        return res.end(readFileSync(resolve(__dirname, '../kiwiirc/arcadia-plugin.js')))
      }
      next()
    })

    server.middlewares.use('/kiwiirc', sirv(kiwiDist, { dev: true, single: true }))
  },
})

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueJsx(),
    kiwiDevServer(),
    customIconsPlugin(),
    // vueDevTools(),
    {
      name: 'arcadia-site-name',
      transformIndexHtml: (html) =>
        html.replace(
          '%SITE_NAME%',
          String(configuration.frontend.site_name).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[c]!),
        ),
    },
    {
      // the web server serves the custom front page whenever the file exists
      name: 'arcadia-custom-front-page',
      apply: 'build',
      closeBundle() {
        if (!configuration.frontend.enable_custom_front_page) {
          rmSync(resolve(__dirname, 'dist/home'), { recursive: true, force: true })
        }
      },
    },
    {
      // same as the Caddyfile: logged out visitors get the custom front page at /
      name: 'arcadia-custom-front-page-dev',
      apply: 'serve',
      configureServer(server) {
        const frontPage = resolve(__dirname, 'public/home/index.html')
        server.middlewares.use((req, res, next) => {
          if (
            !configuration.frontend.enable_custom_front_page ||
            req.url?.split('?')[0] !== '/' ||
            /(^|;\s*)arcadia_logged_in=1(;|$)/.test(req.headers.cookie ?? '') ||
            !existsSync(frontPage)
          ) {
            return next()
          }
          res.setHeader('Content-Type', 'text/html; charset=utf-8')
          res.end(readFileSync(frontPage))
        })
      },
    },
  ],
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
      '/webirc/websocket': {
        target: 'ws://localhost:8097',
        ws: true,
        rewrite: (path) => path.replace(/^\/webirc\/websocket/, ''),
        configure: (proxy) => {
          proxy.on('error', (_err, _req, socket) => {
            // Silently close socket if Ergo is offline to prevent crashing Vite
            if (socket && 'destroyed' in socket && !socket.destroyed) {
              socket.destroy()
            }
          })
        },
      },
    },
  },
  define: {
    __ARCADIA_CONFIG__: JSON.stringify(configuration.frontend),
  },
  resolve: {
    alias: {
      '@/assets/logo.svg': fileURLToPath(new URL(`./src/assets/${logo}`, import.meta.url)),
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
})
