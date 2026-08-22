import { cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { loadContent, loadSiteConfig } from "./loader.js";
import type { SiteModel } from "./model.js";
import { normalizeBasePath, outputPath, slug, withBase } from "./route.js";
import { resolveSite } from "./resolver.js";
import { renderPage } from "./view/pages.js";

export interface BuildOptions {
  root: string;
  basePath?: string;
  siteUrl?: string;
  clean?: boolean;
}

export async function createModel(options: BuildOptions): Promise<SiteModel> {
  const basePath = normalizeBasePath(options.basePath ?? process.env.BASE_PATH);
  const siteUrl =
    options.siteUrl ?? process.env.SITE_URL ?? "https://thp-lang.github.io/thp";
  const [sources, config] = await Promise.all([
    loadContent(options.root),
    loadSiteConfig(options.root),
  ]);
  return resolveSite(sources, config, basePath, siteUrl);
}

export async function build(options: BuildOptions): Promise<SiteModel> {
  const model = await createModel(options);
  const dist = path.join(options.root, "dist");
  if (options.clean !== false) await rm(dist, { recursive: true, force: true });
  await mkdir(dist, { recursive: true });
  await cp(path.join(options.root, "src/assets"), path.join(dist, "assets"), {
    recursive: true,
  });
  for (const page of model.pages) {
    const destination = path.join(dist, outputPath(page.route));
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, renderPage(model, page));
  }
  for (const [legacyRoute, targetRoute] of documentationRedirects(model)) {
    const destination = path.join(dist, outputPath(legacyRoute));
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, redirectPage(model, targetRoute));
  }
  await validateOutput(model, dist);
  return model;
}

function documentationRedirects(model: SiteModel): Map<string, string> {
  const redirects = new Map<string, string>([["/std/spl/", "/std/"]]);
  for (const page of model.pages) {
    const data = page.source.data;
    switch (data.kind) {
      case "class":
      case "interface":
      case "trait":
      case "enum": {
        if (data.id.startsWith("std.spl."))
          redirects.set(`/std/spl/${slug(data.name)}/`, page.route);
        if (data.module === "filesystem")
          redirects.set(`/std/iterators/${slug(data.name)}/`, page.route);
        break;
      }
      case "method": {
        const owner = model.types.get(data.owner);
        if (owner && data.id.startsWith("std.spl."))
          redirects.set(
            `/std/spl/${slug(owner.source.data.name)}/${slug(data.name.replace(/^__/, ""))}/`,
            page.route,
          );
        if (owner?.source.data.module === "filesystem")
          redirects.set(
            `/std/iterators/${slug(owner.source.data.name)}/${slug(data.name.replace(/^__/, ""))}/`,
            page.route,
          );
        break;
      }
      case "function":
        if (data.id.startsWith("std.spl."))
          redirects.set(`/std/spl/functions/${slug(data.name)}/`, page.route);
        break;
    }
  }
  return redirects;
}

function redirectPage(model: SiteModel, targetRoute: string): string {
  const target = withBase(model.basePath, targetRoute);
  const canonical = `${model.siteUrl.replace(/\/$/, "")}${target}`;
  return `<!doctype html>
<html lang="en"><head><meta charset="UTF-8">
  <meta http-equiv="refresh" content="0; url=${target}">
  <link rel="canonical" href="${canonical}">
  <title>Documentation moved — THP Documentation</title>
</head><body><p>This documentation moved to <a href="${target}">${target}</a>.</p></body></html>`;
}

async function validateOutput(model: SiteModel, dist: string): Promise<void> {
  const known = new Set(
    model.pages.map((page) => model.basePath + page.route.slice(1)),
  );
  known.add(model.basePath);
  for (const page of model.pages) {
    const html = await readFile(
      path.join(dist, outputPath(page.route)),
      "utf8",
    );
    const canonical = html.match(/<link rel="canonical" href="([^"]+)"/)?.[1];
    if (!canonical?.startsWith(model.siteUrl))
      throw new Error(`${page.route}: invalid canonical URL`);
    for (const match of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
      const target = match[1]!;
      if (/^(?:https?:|mailto:|#)/.test(target)) continue;
      const withoutHash = target.split("#")[0]!;
      if (withoutHash.startsWith(`${model.basePath}assets/`)) continue;
      if (!known.has(withoutHash))
        throw new Error(`${page.route}: unresolved generated link "${target}"`);
    }
    const ids = new Set(
      [...html.matchAll(/\sid="([^"]+)"/g)].map((match) => match[1]),
    );
    for (const match of html.matchAll(/href="#([^"]+)"/g))
      if (!ids.has(match[1]!))
        throw new Error(`${page.route}: unresolved anchor "#${match[1]}"`);
  }
}
