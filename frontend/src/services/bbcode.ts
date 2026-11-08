import bbobHTML from '@bbob/html'
import presetHTML5 from '@bbob/preset-html5'
import DOMPurify from 'dompurify'
import { isEOL } from '@bbob/plugin-helper'
import { referrerPolicy } from '@/services/helpers'
import { URL_ATTRIBUTES } from '@/services/referrerPolicy'

// v-html content is not compiled, so it misses the referrerpolicy the templates get at build time.
// DOMPurify strips any referrerpolicy from the input, this one is the only one the element gets.
DOMPurify.addHook('afterSanitizeAttributes', (node) => {
  const urlAttribute = URL_ATTRIBUTES[node.nodeName.toLowerCase()]
  if (!urlAttribute) return
  const policy = referrerPolicy(node.getAttribute(urlAttribute))
  if (policy) node.setAttribute('referrerpolicy', policy)
})

export function parseBBCode(bbcode: string): string {
  if (!bbcode) return ''

  const html = bbobHTML(bbcode, [presetHTML5(), lineBreakPlugin()])

  // sanitize html to prevent xss attacks
  return DOMPurify.sanitize(html)
}

//TODO: properly type this function
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const walk = (t: any) => {
  const tree = t

  if (Array.isArray(tree)) {
    for (let idx = 0; idx < tree.length; idx++) {
      const child = walk(tree[idx])
      if (Array.isArray(child)) {
        tree.splice(idx, 1, ...child)
        idx += child.length - 1
      } else {
        tree[idx] = child
      }
    }
  } else if (tree && typeof tree === 'object' && tree.content) {
    if (tree.disableLineBreakConversion) {
      // stop walk. children won't be parsed to have <br>
      return tree.tag ? tree : tree.content
    }
    walk(tree.content)
  }

  if (isEOL(tree)) {
    return [{ tag: 'br', content: null }, '\n']
  }

  return tree
}

export const lineBreakPlugin = () => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return (tree: any) => walk(tree)
}
