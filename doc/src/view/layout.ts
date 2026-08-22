import type { ResolvedType, SiteModel } from "../model.js";
import { withBase } from "../route.js";
import type { GuideSection } from "../schema.js";
import { guideSections } from "../guide-section.js";
import { escapeHtml, trusted, type TrustedHtml } from "./html.js";

function icon(name: "menu" | "search" | "theme" | "github"): string {
  const paths = {
    menu: '<path d="M4 7h16M4 12h16M4 17h16"/>',
    search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/>',
    theme:
      '<path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6 7 7M17 17l1.4 1.4M18.4 5.6 17 7M7 17l-1.4 1.4"/><circle cx="12" cy="12" r="4"/>',
    github:
      '<path d="M15 22v-4c0-1.4-.5-2.5-1-3.5 3.3-.4 6.8-1.6 6.8-7.4A5.8 5.8 0 0 0 19.2 3 5.4 5.4 0 0 0 19 0s-1.3-.4-4 1.6a13.8 13.8 0 0 0-7 0C5.3-.4 4 0 4 0a5.4 5.4 0 0 0-.2 3 5.8 5.8 0 0 0-1.6 4.1c0 5.8 3.5 7 6.8 7.4A4.8 4.8 0 0 0 8 18v4"/><path d="M8 19c-3 .9-3-1.5-4-2"/>',
  };
  return `<svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">${paths[name]}</svg>`;
}

function header(model: SiteModel, section?: string, docs = false): string {
  return `<header class="site-header">
    <div class="header-inner">
      ${docs ? `<button class="icon-button mobile-menu-button" type="button" aria-label="Open documentation navigation" aria-expanded="false" data-mobile-menu>${icon("menu")}</button>` : ""}
      <a class="brand" href="${withBase(model.basePath, "/")}" aria-label="THP documentation home">
        <span class="brand-mark" aria-hidden="true">th</span><span>THP</span><span class="brand-version">Docs</span>
      </a>
      <nav class="header-nav" aria-label="Primary navigation">
        ${model.config.navigation
          .map(
            (item) =>
              `<a href="${withBase(model.basePath, item.href)}"${section === item.label ? ' aria-current="page"' : ""}>${escapeHtml(item.label)}</a>`,
          )
          .join("")}
      </nav>
      <div class="header-tools">
        <button class="search-trigger" type="button" data-search-open>${icon("search")}<span>Search documentation</span><kbd class="keycap">⌘ K</kbd></button>
        <button class="icon-button" type="button" data-theme-toggle aria-label="Use dark theme">${icon("theme")}</button>
        <a class="icon-button" href="${escapeHtml(model.config.github)}" aria-label="THP on GitHub">${icon("github")}</a>
      </div>
    </div>
  </header>`;
}

function searchDialog(model: SiteModel): string {
  const moduleOptions = Object.entries(model.config.modules)
    .filter(([id]) => id !== "standard-library")
    .sort(([, left], [, right]) => left.order - right.order)
    .map(
      ([id, item]) =>
        `<option value="${escapeHtml(id)}">${escapeHtml(item.name)}</option>`,
    )
    .join("");
  return `<dialog class="search-dialog" data-search-dialog aria-labelledby="search-title">
    <div class="search-box">
      <div class="search-heading"><strong id="search-title">Search THP documentation</strong><button type="button" data-search-close aria-label="Close search">Esc</button></div>
      <label class="search-input-wrap">${icon("search")}<span class="sr-only">Search</span><input type="search" data-search-input placeholder="Class, method, function, or topic…" autocomplete="off"></label>
      <div class="search-filters" aria-label="Search filters">
        <select data-search-kind aria-label="Filter by kind"><option value="">All kinds</option><option>class</option><option>method</option><option>function</option><option>guide</option><option>module</option></select>
        <select data-search-module aria-label="Filter by module"><option value="">All modules</option>${moduleOptions}</select>
        <select data-search-availability aria-label="Filter by availability"><option value="">All availability</option><option>implemented</option><option>partial</option><option>proposed</option></select>
        <select data-search-status aria-label="Filter by status"><option value="">All statuses</option><option>experimental</option><option>stable</option><option>deprecated</option></select>
      </div>
      <div class="search-results" data-search-results aria-live="polite"><p class="search-empty">Start typing to search the documentation.</p></div>
    </div>
  </dialog>`;
}

function typeLink(
  model: SiteModel,
  item: ResolvedType,
  current?: string,
): string {
  const kind = sidebarTypeKind(item);
  return `<li><a class="symbol-link" href="${withBase(model.basePath, item.route)}"${current === item.source.data.id ? ' aria-current="page"' : ""}>${symbolKindIcon(kind)}<span class="sidebar-symbol-name">${escapeHtml(item.source.data.name)}</span></a></li>`;
}

type SidebarSymbolKind =
  "class" | "interface" | "trait" | "enum" | "exception" | "function";

function sidebarTypeKind(item: ResolvedType): SidebarSymbolKind {
  if (
    item.source.data.kind === "class" &&
    (item.source.data.id === "std.baseTypes.Exception" ||
      (item.parent && sidebarTypeKind(item.parent) === "exception"))
  )
    return "exception";
  return item.source.data.kind;
}

function symbolKindIcon(kind: SidebarSymbolKind): string {
  const glyphs: Record<SidebarSymbolKind, string> = {
    class: "C",
    interface: "I",
    trait: "T",
    enum: "E",
    exception: "X",
    function: "ƒ",
  };
  return `<span class="symbol-kind-icon symbol-kind-${kind}" data-symbol-kind="${kind}" aria-hidden="true">${glyphs[kind]}</span>`;
}

export function sidebar(
  model: SiteModel,
  current?: string,
  type?: ResolvedType,
  module?: string,
): string {
  const activeModule = module ?? type?.source.data.module;
  const modulePages = new Map(
    model.pages.flatMap((page) =>
      page.source.data.kind === "module"
        ? ([[page.source.data.module, page]] as const)
        : [],
    ),
  );
  const standardLibraryLinks = Object.entries(model.config.modules)
    .filter(([id]) => id !== "standard-library")
    .sort(([, a], [, b]) => a.order - b.order || a.name.localeCompare(b.name))
    .map(([id, item]) => {
      const page = modulePages.get(id);
      if (!page)
        return `<li><span class="sidebar-item" aria-disabled="true">${escapeHtml(item.name)}</span></li>`;
      const isCurrent = activeModule === id || current === page.source.data.id;
      return `<li><a href="${withBase(model.basePath, page.route)}"${isCurrent ? ' aria-current="page"' : ""}>${escapeHtml(item.name)}</a></li>`;
    })
    .join("");
  const standardOverview = modulePages.get("standard-library");
  const overview = standardOverview
    ? `<li><a href="${withBase(model.basePath, standardOverview.route)}"${current === standardOverview.source.data.id ? ' aria-current="page"' : ""}>Overview</a></li>`
    : "";
  const typeLinks = [...model.types.values()]
    .filter((item) => item.source.data.module === activeModule)
    .sort((a, b) => a.source.data.name.localeCompare(b.source.data.name));
  const typeSection =
    activeModule && typeLinks.length
      ? `<div class="sidebar-section"><p class="sidebar-label">${escapeHtml(model.config.modules[activeModule]?.name ?? activeModule)} ${escapeHtml(model.config.modules[activeModule]?.typeGroup ?? "types")}</p><ul class="sidebar-list">${typeLinks.map((item) => typeLink(model, item, current)).join("")}</ul></div>`
      : "";
  const functionLinks = model.pages
    .filter(
      (item) =>
        item.source.data.kind === "function" &&
        item.source.data.module === activeModule,
    )
    .sort((a, b) => a.source.data.title.localeCompare(b.source.data.title));
  const functionSection =
    activeModule && functionLinks.length
      ? `<div class="sidebar-section"><p class="sidebar-label">${escapeHtml(model.config.modules[activeModule]?.name ?? activeModule)} functions</p><ul class="sidebar-list">${functionLinks
          .map(
            (item) =>
              `<li><a class="symbol-link" href="${withBase(model.basePath, item.route)}"${current === item.source.data.id ? ' aria-current="page"' : ""}>${symbolKindIcon("function")}<span class="sidebar-symbol-name">${escapeHtml(item.source.data.title)}</span></a></li>`,
          )
          .join("")}</ul></div>`
      : "";
  return `<aside class="docs-sidebar" aria-label="Documentation navigation" data-docs-sidebar>
    <div class="sidebar-section"><p class="sidebar-label">Standard library <span class="sidebar-badge">THP</span></p><ul class="sidebar-list">
      ${overview}${standardLibraryLinks}
    </ul></div>${typeSection}${functionSection}
  </aside><button class="mobile-sidebar-backdrop" type="button" aria-label="Close documentation navigation" data-sidebar-backdrop hidden></button>`;
}

export function pageNavigation(
  model: SiteModel,
  links: ReadonlyArray<readonly [label: string, href: string]>,
  type?: ResolvedType,
  current?: string,
): string {
  const pageLinks = links
    .map(
      ([label, href]) =>
        `<a href="${escapeHtml(href)}">${escapeHtml(label)}</a>`,
    )
    .join("");
  const visibleMethods =
    type?.members.filter(
      (member) => member.source.data.name !== "__construct",
    ) ?? [];
  const methods = type
    ? `<div class="toc-section toc-members"><strong>On this ${escapeHtml(type.source.data.kind)}</strong>${visibleMethods
        .map(
          (member) =>
            `<a class="method-link" href="${withBase(model.basePath, member.route)}"${member.inherited ? ' data-inherited="true"' : ""}${current === member.source.data.id ? ' aria-current="page"' : ""}>${escapeHtml(member.source.data.name)}()</a>`,
        )
        .join("")}</div>`
    : "";
  return `<aside class="toc" aria-label="Page navigation"><div class="toc-section"><strong>On this page</strong>${pageLinks}</div>${methods}</aside>`;
}

export function guideSidebar(
  model: SiteModel,
  current: string,
  section: GuideSection,
): string {
  const links = model.pages
    .filter(
      (page) =>
        page.source.data.kind === "guide" &&
        page.source.data.nav.section === section,
    )
    .sort(
      (a, b) =>
        (a.source.data.kind === "guide" ? a.source.data.nav.order : 0) -
          (b.source.data.kind === "guide" ? b.source.data.nav.order : 0) ||
        a.source.data.title.localeCompare(b.source.data.title),
    )
    .map(
      (page) =>
        `<li><a href="${withBase(model.basePath, page.route)}"${current === page.source.data.id ? ' aria-current="page"' : ""}>${escapeHtml(page.source.data.title)}</a></li>`,
    )
    .join("");
  const label = guideSections[section].sidebar;
  return `<aside class="docs-sidebar" aria-label="Documentation navigation" data-docs-sidebar>
    <div class="sidebar-section"><p class="sidebar-label">${label} <span class="sidebar-badge">THP</span></p><ul class="sidebar-list">
      ${links}
    </ul></div>
  </aside><button class="mobile-sidebar-backdrop" type="button" aria-label="Close documentation navigation" data-sidebar-backdrop hidden></button>`;
}

export interface LayoutOptions {
  title: string;
  description: string;
  route: string;
  section?: string | undefined;
  docs?: boolean | undefined;
  sidebar?: string | undefined;
  body: TrustedHtml;
  kind: string;
  module?: string | undefined;
  status?: string | undefined;
  availability: "implemented" | "partial" | "proposed";
}

export function layout(model: SiteModel, options: LayoutOptions): string {
  const canonical = `${model.siteUrl.replace(/\/$/, "")}${withBase(model.basePath, options.route)}`;
  return `<!doctype html>
<html lang="en"><head>
  <meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="description" content="${escapeHtml(options.description)}"><meta name="generator" content="THP documentation compiler">
  <link rel="canonical" href="${escapeHtml(canonical)}"><title>${escapeHtml(options.title)} — THP Documentation</title>
  <link rel="icon" href="${withBase(model.basePath, "/assets/favicon.svg")}"><link rel="stylesheet" href="${withBase(model.basePath, "/assets/styles.css")}">
  <script>try{document.documentElement.dataset.theme=localStorage.getItem("thp-docs-theme")||(matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light")}catch{}</script>
</head><body>
  <a class="skip-link" href="#main">Skip to content</a>${header(model, options.section, options.docs)}
  ${options.docs ? `<div class="docs-shell">${options.sidebar ?? sidebar(model)}<main class="docs-main" id="main">${options.body}</main></div>` : `<main id="main">${options.body}</main>`}
  ${searchDialog(model)}
  <script>window.THP_DOCS_BASE=${JSON.stringify(model.basePath)};</script><script type="module" src="${withBase(model.basePath, "/assets/app.js")}"></script>
  <span hidden data-pagefind-filter="kind">${escapeHtml(options.kind)}</span>
  ${options.module ? `<span hidden data-pagefind-filter="module">${escapeHtml(options.module)}</span>` : ""}
  ${options.status ? `<span hidden data-pagefind-filter="status">${escapeHtml(options.status)}</span>` : ""}
  <span hidden data-pagefind-filter="availability" data-pagefind-meta="availability">${escapeHtml(options.availability)}</span>
</body></html>`;
}

export function baseMarkdown(model: SiteModel, html: string): TrustedHtml {
  return trusted(
    html.replaceAll('href="/', `href="${model.basePath}`.replace("//", "/")),
  );
}
