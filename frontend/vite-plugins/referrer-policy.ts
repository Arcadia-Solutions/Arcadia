// Gives a referrerpolicy to every element loading a url from `frontend.referrer_allowed_domains`,
// so those domains get the origin while the rest of the site keeps the no-referrer policy.
// - In the templates, a static url is checked at build time and a bound one at runtime through the
//   `$referrerPolicy` global property (main.ts). Elements setting their own referrerpolicy are left alone.
// - The custom front page (public/home/*.html) is static, it gets the no-referrer meta and the
//   attributes written in its files.

import { readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { NodeTypes, createCompoundExpression, createSimpleExpression, parse } from '@vue/compiler-dom'
import type { AttributeNode, DirectiveNode, ElementNode, NodeTransform, TemplateChildNode } from '@vue/compiler-dom'
import { URL_ATTRIBUTES, referrerPolicyFor } from '../src/services/referrerPolicy'

const isBound = (prop: DirectiveNode, name: string) => prop.arg?.type === NodeTypes.SIMPLE_EXPRESSION && prop.arg.content === name

const isReferrerPolicyProp = (prop: AttributeNode | DirectiveNode) =>
  prop.type === NodeTypes.ATTRIBUTE ? prop.name === 'referrerpolicy' : isBound(prop, 'referrerpolicy')

export const referrerPolicyTransform =
  (allowedDomains: string[]): NodeTransform =>
  (node) => {
    if (allowedDomains.length === 0 || node.type !== NodeTypes.ELEMENT) return
    const urlAttribute = URL_ATTRIBUTES[node.tag]
    if (!urlAttribute || node.props.some(isReferrerPolicyProp)) return
    for (const prop of node.props) {
      if (prop.type === NodeTypes.ATTRIBUTE && prop.name === urlAttribute) {
        const policy = referrerPolicyFor(prop.value?.content, allowedDomains)
        if (policy) {
          node.props.push({
            type: NodeTypes.ATTRIBUTE,
            name: 'referrerpolicy',
            nameLoc: prop.loc,
            value: { type: NodeTypes.TEXT, content: policy, loc: prop.loc },
            loc: prop.loc,
          })
        }
        return
      }
      if (prop.type === NodeTypes.DIRECTIVE && prop.name === 'bind' && isBound(prop, urlAttribute) && prop.exp) {
        // user transforms run after the identifiers were prefixed, hence the explicit _ctx
        node.props.push({
          type: NodeTypes.DIRECTIVE,
          name: 'bind',
          rawName: ':referrerpolicy',
          arg: createSimpleExpression('referrerpolicy', true),
          exp: createCompoundExpression(['_ctx.$referrerPolicy(', prop.exp, ')']),
          modifiers: [],
          loc: prop.loc,
        })
        return
      }
    }
  }

const elements = (nodes: TemplateChildNode[]): ElementNode[] =>
  nodes.flatMap((node) => (node.type === NodeTypes.ELEMENT ? [node, ...elements(node.children)] : []))

export const addReferrerPolicies = (html: string, allowedDomains: string[]): string => {
  const insertions: { offset: number; text: string }[] = []
  for (const element of elements(parse(html, { parseMode: 'html' }).children)) {
    const tagEnd = element.loc.start.offset + 1 + element.tag.length
    if (element.tag === 'head' && !/<meta\s[^>]*name=["']?referrer/i.test(html)) {
      insertions.push({ offset: html.indexOf('>', tagEnd) + 1, text: '<meta name="referrer" content="no-referrer" />' })
    }
    const urlAttribute = URL_ATTRIBUTES[element.tag]
    if (!urlAttribute || element.props.some(isReferrerPolicyProp)) continue
    const url = element.props.find((prop): prop is AttributeNode => prop.type === NodeTypes.ATTRIBUTE && prop.name === urlAttribute)
    const policy = referrerPolicyFor(url?.value?.content, allowedDomains)
    if (policy) insertions.push({ offset: tagEnd, text: ` referrerpolicy="${policy}"` })
  }
  return insertions.sort((a, b) => b.offset - a.offset).reduce((result, { offset, text }) => result.slice(0, offset) + text + result.slice(offset), html)
}

// rewrites the html files of a built custom front page directory
export const addReferrerPoliciesToDirectory = (directory: string, allowedDomains: string[]) => {
  for (const file of readdirSync(directory, { recursive: true, encoding: 'utf8' })) {
    if (!file.endsWith('.html')) continue
    const path = resolve(directory, file)
    writeFileSync(path, addReferrerPolicies(readFileSync(path, 'utf8'), allowedDomains))
  }
}
