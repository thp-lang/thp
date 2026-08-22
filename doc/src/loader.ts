import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import matter from "gray-matter";
import { lexer } from "marked";
import { parse as parseYaml } from "yaml";
import { DiagnosticError, lineFor } from "./diagnostic.js";
import type { SourcePage } from "./model.js";
import { pageSchema, siteSchema, type SiteConfig } from "./schema.js";

async function filesBelow(directory: string): Promise<string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map((entry) => {
      const target = path.join(directory, entry.name);
      return entry.isDirectory()
        ? filesBelow(target)
        : Promise.resolve([target]);
    }),
  );
  return nested
    .flat()
    .filter((file) => file.endsWith(".md"))
    .sort();
}

function rawHtmlToken(value: unknown): string | undefined {
  if (!value || typeof value !== "object") return undefined;
  if ("type" in value && value.type === "html" && "raw" in value)
    return String(value.raw);
  for (const nested of Object.values(value))
    if (Array.isArray(nested))
      for (const item of nested) {
        const found = rawHtmlToken(item);
        if (found) return found;
      }
  return undefined;
}

export async function loadContent(root: string): Promise<SourcePage[]> {
  const contentRoot = path.join(root, "content");
  const files = await filesBelow(contentRoot);
  return Promise.all(
    files.map(async (file) => {
      const source = await readFile(file, "utf8");
      if (!source.startsWith("---\n"))
        throw new DiagnosticError(
          path.relative(root, file),
          1,
          "missing YAML frontmatter",
        );
      const parsed = matter(source);
      const result = pageSchema.safeParse(parsed.data);
      if (!result.success) {
        const issue = result.error.issues[0]!;
        const key = String(issue.path.at(-1) ?? "frontmatter");
        throw new DiagnosticError(
          path.relative(root, file),
          lineFor(source, `${key}:`),
          `${issue.path.join(".") || "frontmatter"}: ${issue.message}`,
        );
      }
      const rawHtml = rawHtmlToken(lexer(parsed.content));
      if (rawHtml)
        throw new DiagnosticError(
          path.relative(root, file),
          lineFor(source, rawHtml),
          "raw HTML is not permitted in Markdown",
        );
      return {
        data: result.data,
        body: parsed.content.trim(),
        file: path.relative(root, file),
      };
    }),
  );
}

export async function loadSiteConfig(root: string): Promise<SiteConfig> {
  const file = path.join(root, "site.yaml");
  const source = await readFile(file, "utf8");
  const result = siteSchema.safeParse(parseYaml(source));
  if (!result.success) {
    const issue = result.error.issues[0]!;
    throw new DiagnosticError(
      "site.yaml",
      lineFor(source, `${String(issue.path.at(-1))}:`),
      issue.message,
    );
  }
  return result.data;
}
