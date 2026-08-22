import { Marked } from "marked";
import { createHighlighter, type Highlighter } from "shiki";
import { slug } from "./route.js";
import { escapeHtml, trusted, type TrustedHtml } from "./view/html.js";

let highlighter: Promise<Highlighter> | undefined;

function getHighlighter(): Promise<Highlighter> {
  highlighter ??= createHighlighter({
    themes: ["github-dark"],
    langs: ["typescript", "php", "markdown"],
  });
  return highlighter;
}

function languageFor(name: string): "typescript" | "php" | "markdown" {
  if (name === "thp") return "typescript";
  if (name === "php") return "php";
  return "markdown";
}

export async function renderMarkdown(source: string): Promise<TrustedHtml> {
  const syntax = await getHighlighter();
  const headingIds = new Map<string, number>();
  const marked = new Marked({
    gfm: true,
    async: false,
    renderer: {
      html(token): string {
        return escapeHtml(token.raw);
      },
      code({ text, lang }): string {
        const language = languageFor(lang ?? "");
        return syntax.codeToHtml(text, {
          lang: language,
          theme: "github-dark",
        });
      },
      heading({ text, tokens, depth }): string {
        const baseId = slug(text);
        const occurrence = headingIds.get(baseId) ?? 0;
        headingIds.set(baseId, occurrence + 1);
        const id = occurrence === 0 ? baseId : `${baseId}-${occurrence + 1}`;
        return `<h${depth} id="${escapeHtml(id)}">${this.parser.parseInline(tokens)}</h${depth}>`;
      },
    },
  });
  const rendered = marked.parse(source) as string;
  return trusted(
    rendered
      .replaceAll("<table>", '<div class="table-scroll"><table>')
      .replaceAll("</table>", "</table></div>"),
  );
}
