import { marked } from 'marked'

/**
 * Render Markdown to HTML.
 * Uses the `marked` library for full CommonMark + GFM support.
 */
export function renderMarkdown(md: string): string {
  return marked.parse(md, { async: false }) as string
}
