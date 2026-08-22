import type {
  ResolvedMember,
  ResolvedPage,
  ResolvedType,
  SiteModel,
} from "../model.js";
import {
  parseType,
  printType,
  substituteType,
  type TypeExpression,
} from "../type-expression.js";
import { withBase } from "../route.js";
import { guideSections } from "../guide-section.js";
import { escapeHtml, trusted, type TrustedHtml } from "./html.js";
import {
  baseMarkdown,
  guideSidebar,
  layout,
  pageNavigation,
  sidebar,
} from "./layout.js";

function pageFor(model: SiteModel, id: string): ResolvedPage {
  const page = model.pages.find((item) => item.source.data.id === id);
  if (!page) throw new Error(`Missing resolved page ${id}`);
  return page;
}

function notice(
  status: string | undefined,
  message?: string,
  title = "Experimental API",
): string {
  if (status !== "experimental") return "";
  return `<div class="experimental-note"><strong>${escapeHtml(title)}</strong><p>${escapeHtml(message ?? "This contract is under active design and may change.")}</p></div>`;
}

function availabilityBadge(
  availability: "implemented" | "partial" | "proposed",
): string {
  return `<span class="availability-badge availability-${availability}">${availability}</span>`;
}

function identifierWithBreaks(value: string): string {
  const breakMarker = "\u200b";
  return escapeHtml(
    value
      .replaceAll("::", `::${breakMarker}`)
      .replaceAll("_", `_${breakMarker}`)
      .replace(/([a-z0-9])([A-Z])/g, `$1${breakMarker}$2`)
      .replace(/([A-Z])([A-Z][a-z])/g, `$1${breakMarker}$2`),
  ).replaceAll(breakMarker, "<wbr>");
}

function breadcrumbs(
  model: SiteModel,
  items: Array<[string, string?]>,
): string {
  return `<nav class="breadcrumbs" aria-label="Breadcrumb">${items
    .map(([label, route], index) =>
      route
        ? `<a href="${withBase(model.basePath, route)}">${escapeHtml(label)}</a><span aria-hidden="true">/</span>`
        : `<span aria-current="page">${escapeHtml(label)}</span>${index < items.length - 1 ? '<span aria-hidden="true">/</span>' : ""}`,
    )
    .join("")}</nav>`;
}

function articleBody(model: SiteModel, page: ResolvedPage): TrustedHtml {
  return baseMarkdown(model, page.renderedBody);
}

export function renderHome(model: SiteModel, page: ResolvedPage): string {
  const body =
    trusted(`<div class="announcement"><span class="status-dot"></span> THP 0.1 is an experimental CLI release. <a href="${withBase(model.basePath, "/learn/implementation-status/")}">See what is implemented.</a></div>
  <section class="hero">
    <div class="hero-grid"><div class="hero-copy"><p class="eyebrow">Static types · Verified bytecode · Standalone runtime</p><h1>A typed language with <span>its own runtime.</span></h1><p class="hero-lead">${escapeHtml(page.source.data.summary)}</p>
      <div class="hero-actions"><a class="button button-primary" href="${withBase(model.basePath, "/learn/getting-started/")}">Start with THP →</a><a class="button" href="${withBase(model.basePath, "/learn/implementation-status/")}">See what works today</a></div>
    </div><div class="code-window"><div class="code-window-bar"><span>hello.thp</span><span>THP</span></div>
      <pre><code><span class="comment">&lt;?thp</span>

<span class="kw">function</span> greet(<span class="type">string</span> <span class="var">$name</span>): <span class="type">string</span> {
  <span class="kw">return</span> <span class="string">"Hello, "</span> . <span class="var">$name</span> . <span class="string">"!\\n"</span>;
}

<span class="kw">echo</span> greet(<span class="string">"world"</span>);</code></pre>
      <div class="code-window-command"><code><span>$ thp run hello.thp</span><strong>Hello, world!</strong></code></div>
    </div></div>
  </section>
  <section class="section prose home-prose" data-pagefind-body>${articleBody(model, page)}</section>`);
  return layout(model, {
    title: page.source.data.title,
    description: page.source.data.summary,
    route: page.route,
    kind: "home",
    status: page.source.data.status,
    availability: page.source.data.availability,
    body,
  });
}

export function renderGuide(model: SiteModel, page: ResolvedPage): string {
  const data = page.source.data;
  if (data.kind !== "guide") throw new Error("Expected guide");
  const section = guideSections[data.nav.section];
  const body =
    trusted(`<div class="docs-content-grid"><article class="article prose" data-pagefind-body>
    ${breadcrumbs(model, [["Docs", "/"], [section.breadcrumb, section.root], [data.title]])}
    <header class="article-header"><p class="content-kicker">${escapeHtml(data.nav.section)} guide</p>${availabilityBadge(data.availability)}<h1 data-pagefind-meta="title">${escapeHtml(data.title)}</h1><p class="article-lead">${escapeHtml(data.summary)}</p></header>
    ${notice(data.status, data.notice, section.notice)}
    ${articleBody(model, page)}
  </article></div>`);
  return layout(model, {
    title: data.title,
    description: data.summary,
    route: page.route,
    section: section.navigation,
    docs: true,
    sidebar: guideSidebar(model, data.id, data.nav.section),
    body,
    kind: "guide",
    status: data.status,
    availability: data.availability,
  });
}

export function renderModule(model: SiteModel, page: ResolvedPage): string {
  const data = page.source.data;
  if (data.kind !== "module") throw new Error("Expected module");
  const types = [...model.types.values()].filter(
    (type) => type.source.data.module === data.module,
  );
  const functions = model.pages.filter(
    (item) =>
      item.source.data.kind === "function" &&
      item.source.data.module === data.module,
  );
  const cards: Array<[string, string, string]> = [
    ...types.map((type): [string, string, string] => [
      type.source.data.name,
      type.source.data.summary,
      type.route,
    ]),
    ...functions.map((item): [string, string, string] => [
      item.source.data.title,
      item.source.data.summary,
      item.route,
    ]),
  ];
  const cardHtml = cards
    .map(
      ([name, summary, route]) =>
        `<a class="symbol-card" href="${withBase(model.basePath, route)}"><strong>${escapeHtml(name)}</strong><span>${escapeHtml(summary)}</span></a>`,
    )
    .join("");
  const body =
    trusted(`<div class="docs-content-grid"><article class="article prose" data-pagefind-body>
    ${breadcrumbs(model, data.module === "standard-library" ? [["Docs", "/"], [data.title]] : [["Docs", "/"], ["Standard library", "/std/"], [data.title]])}
    <header class="article-header"><p class="content-kicker">Standard-library module</p>${availabilityBadge(data.availability)}<h1 data-pagefind-meta="title">${escapeHtml(data.title)}</h1><p class="article-lead">${escapeHtml(data.summary)}</p></header>
    ${notice(data.status, data.notice)}${articleBody(model, page)}${cardHtml ? `<section><h2 id="symbols">Symbols</h2><div class="symbol-grid">${cardHtml}</div></section>` : ""}
  </article></div>`);
  return layout(model, {
    title: data.title,
    description: data.summary,
    route: page.route,
    section: "Runtime/API",
    docs: true,
    sidebar: sidebar(model, data.id, undefined, data.module),
    body,
    kind: "module",
    module: data.module,
    status: data.status,
    availability: data.availability,
  });
}

function signature(
  model: SiteModel,
  member: ResolvedMember,
  options: { synopsis?: boolean } = {},
): string {
  const data = member.source.data;
  const modifiers = data.modifiers.length ? `${data.modifiers.join(" ")} ` : "";
  const typeParameters = data.typeParameters.length
    ? `&lt;${data.typeParameters.map((parameter) => escapeHtml(parameter.name)).join(", ")}&gt;`
    : "";
  const parameters = member.parameters
    .map((parameter) => {
      const suffix =
        parameter.default === undefined
          ? ""
          : ` = ${escapeHtml(parameter.default)}`;
      const variadic = parameter.variadic ? "..." : "";
      return `<span class="type">${escapeHtml(printType(parameter.parsedType))}</span> ${variadic}<span class="var">$${escapeHtml(parameter.name)}</span>${suffix}`;
    })
    .join(", ");
  const returnType =
    options.synopsis &&
    data.name === "__construct" &&
    printType(member.parsedReturn) === "void"
      ? ""
      : `: <span class="type">${escapeHtml(printType(member.parsedReturn))}</span>`;
  return `<span class="kw">${escapeHtml(data.visibility)} ${escapeHtml(modifiers)}function</span> <a href="${withBase(model.basePath, member.route)}">${escapeHtml(data.name)}</a>${typeParameters}(${parameters})${returnType}`;
}

function relatedTypeArguments(
  type: ResolvedType,
  related: ResolvedType,
): string[] {
  if (type.source.data.parent?.id === related.source.data.id)
    return type.source.data.parent.arguments ?? [];
  const implemented = type.source.data.interfaces.find(
    (item) => item.id === related.source.data.id,
  );
  if (implemented) return implemented.arguments ?? [];
  return related.source.data.typeParameters.map((parameter) => parameter.name);
}

function typeName(name: string, arguments_: string[]): string {
  const argumentsHtml = arguments_.length
    ? `&lt;${arguments_.map(escapeHtml).join(", ")}&gt;`
    : "";
  return `${escapeHtml(name)}${argumentsHtml}`;
}

interface ImplementedInterface {
  type: ResolvedType;
  arguments: TypeExpression[];
}

function implementedInterfaces(type: ResolvedType): ImplementedInterface[] {
  const collect = (
    current: ResolvedType,
    substitutions: ReadonlyMap<string, TypeExpression>,
  ): ImplementedInterface[] => {
    const direct = current.interfaces.map((implemented, index) => {
      const relation = current.source.data.interfaces[index]!;
      return {
        type: implemented,
        arguments: (relation.arguments ?? []).map((argument) =>
          substituteType(parseType(argument), substitutions),
        ),
      };
    });
    if (!current.parent || !current.source.data.parent) return direct;
    const parentSubstitutions = new Map<string, TypeExpression>();
    current.parent.source.data.typeParameters.forEach((parameter, index) => {
      const argument = current.source.data.parent?.arguments?.[index];
      if (argument)
        parentSubstitutions.set(
          parameter.name,
          substituteType(parseType(argument), substitutions),
        );
    });
    return [...direct, ...collect(current.parent, parentSubstitutions)];
  };

  const unique = new Map<string, ImplementedInterface>();
  for (const implemented of collect(type, new Map()))
    unique.set(implemented.type.source.data.id, implemented);
  return [...unique.values()];
}

export function renderType(model: SiteModel, type: ResolvedType): string {
  const data = type.source.data;
  const page = pageFor(model, data.id);
  const inherited = type.members.filter((member) => member.inherited);
  const parent = type.parent
    ? `<a class="inheritance-node" href="${withBase(model.basePath, type.parent.route)}">${typeName(type.parent.source.data.name, relatedTypeArguments(type, type.parent))}</a><span aria-hidden="true">→</span>`
    : "";
  const synopsisGroups: string[] = [];
  if (data.kind === "enum" && data.cases.length) {
    synopsisGroups.push(
      data.cases
        .map(
          (name) =>
            `<span class="member-signature" data-member-origin="declared"><span class="kw">case</span> ${escapeHtml(name)};</span>`,
        )
        .join("\n    "),
    );
  }
  if (type.declared.length) {
    synopsisGroups.push(
      type.declared
        .map(
          (member) =>
            `<span class="member-signature" data-member-origin="declared">${signature(model, member, { synopsis: true })};</span>`,
        )
        .join("\n    "),
    );
  }
  const inheritedGroups = new Map<
    string,
    { owner: ResolvedType; members: ResolvedMember[] }
  >();
  for (const member of inherited) {
    const id = member.declaringType.source.data.id;
    const group = inheritedGroups.get(id) ?? {
      owner: member.declaringType,
      members: [],
    };
    group.members.push(member);
    inheritedGroups.set(id, group);
  }
  for (const group of inheritedGroups.values()) {
    const relationship =
      data.kind !== "interface" && group.owner.source.data.kind === "interface"
        ? "Implementing"
        : "Inherited from";
    synopsisGroups.push(
      [
        `<span class="comment">// ${relationship} ${typeName(group.owner.source.data.name, relatedTypeArguments(type, group.owner))}</span>`,
        ...group.members.map(
          (member) =>
            `<span class="member-signature inherited-member" data-member-origin="inherited">${signature(model, member, { synopsis: true })};</span>`,
        ),
      ].join("\n    "),
    );
  }
  const synopsis = synopsisGroups.join("\n\n    ");
  const parentDeclaration = type.parent
    ? `\n    <span class="kw">extends</span> <a href="${withBase(model.basePath, type.parent.route)}"><span class="type">${typeName(type.parent.source.data.name, relatedTypeArguments(type, type.parent))}</span></a>`
    : "";
  const interfaces = implementedInterfaces(type);
  const interfaceDeclaration = interfaces.length
    ? `\n    <span class="kw">${data.kind === "interface" ? "extends" : "implements"}</span> ${interfaces
        .map(
          (implemented) =>
            `<a href="${withBase(model.basePath, implemented.type.route)}"><span class="type">${typeName(
              implemented.type.source.data.name,
              implemented.arguments.map(printType),
            )}</span></a>`,
        )
        .join(", ")}`
    : "";
  const rows = type.members
    .map(
      (member) =>
        `<div class="api-row" data-api-row data-kind="${member.inherited ? "inherited" : "declared"}"><div class="api-name"><a href="${withBase(model.basePath, member.route)}">${escapeHtml(member.source.data.name)}()</a></div><p>${escapeHtml(member.source.data.summary)}</p><span class="declared-by">${member.inherited ? `Inherited from ${escapeHtml(member.declaringType.source.data.name)}` : "Declared here"}</span></div>`,
    )
    .join("");
  const typeParams = data.typeParameters.length
    ? `<section class="article-section" id="type-parameters"><h2>Type parameters</h2><table class="api-table"><thead><tr><th>Name</th><th>Description</th></tr></thead><tbody>${data.typeParameters.map((item) => `<tr><td><code>${escapeHtml(item.name)}</code></td><td>${escapeHtml(item.description)}</td></tr>`).join("")}</tbody></table></section>`
    : "";
  const factsTable = (
    heading: string,
    facts: typeof data.constants | typeof data.properties,
  ): string =>
    facts.length
      ? `<section class="article-section" id="${heading.toLowerCase()}"><h2>${heading}</h2><table class="api-table"><thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead><tbody>${facts.map((item) => `<tr><td><code>${escapeHtml(item.name)}</code></td><td><code>${escapeHtml(item.type)}</code></td><td>${escapeHtml(item.description)}</td></tr>`).join("")}</tbody></table></section>`
      : "";
  const constants = factsTable("Constants", data.constants);
  const properties = factsTable("Properties", data.properties);
  const navigation = pageNavigation(
    model,
    [
      ["Overview", "#overview"],
      ["Class synopsis", "#synopsis"],
      ...(data.typeParameters.length
        ? ([["Type parameters", "#type-parameters"]] as const)
        : []),
      ...(data.constants.length
        ? ([["Constants", "#constants"]] as const)
        : []),
      ...(data.properties.length
        ? ([["Properties", "#properties"]] as const)
        : []),
      ["Methods", "#methods"],
    ],
    type,
    data.id,
  );
  const body =
    trusted(`<div class="docs-content-grid"><article class="article" data-pagefind-body>
    ${breadcrumbs(model, [["Docs", "/"], ["Standard library", "/std/"], [model.config.modules[data.module]?.name ?? data.module, `/std/${data.module}/`], [data.name]])}
    <header class="article-header"><p class="content-kicker">${escapeHtml(data.module)} · ${escapeHtml(data.kind)} reference</p>${availabilityBadge(data.availability)}<div class="article-title-row"><h1><code data-pagefind-meta="title">${identifierWithBreaks(data.name)}</code></h1><span class="kind-badge">${escapeHtml(data.kind)}</span></div><p class="article-lead">${escapeHtml(data.summary)}</p><div class="meta-row"><span>${type.members.length} methods</span><span>THP ${escapeHtml(data.version)}</span></div></header>
    ${notice(data.status, data.notice)}
    <section class="article-section prose" id="overview"><h2>Overview</h2>${articleBody(model, page)}
      <div class="inheritance-path">${parent}<span class="inheritance-node current">${escapeHtml(data.name)}${data.typeParameters.length ? `&lt;${data.typeParameters.map((item) => escapeHtml(item.name)).join(", ")}&gt;` : ""}</span></div>
    </section>
    <section class="article-section" id="synopsis"><h2>Class synopsis</h2><p>The complete public surface. Declared methods are listed first, followed by methods grouped under their declaring type.</p>
      <div class="synopsis"><div class="synopsis-toolbar"><span class="synopsis-title">${escapeHtml(data.name)}.thp</span><button class="copy-button" type="button" data-copy-target="class-synopsis">Copy</button></div><pre id="class-synopsis"><code><span class="kw">${escapeHtml(data.kind)}</span> <span class="type">${escapeHtml(data.name)}${data.typeParameters.length ? `&lt;${data.typeParameters.map((item) => escapeHtml(item.name)).join(", ")}&gt;` : ""}</span>${parentDeclaration}${interfaceDeclaration} {

    ${synopsis}
}</code></pre></div>
    </section>${typeParams}${constants}${properties}
    <section class="article-section" id="methods"><h2>Methods</h2><div class="filter-bar"><input type="search" data-method-search aria-label="Filter methods" placeholder="Filter methods…"><button data-method-filter="all" aria-pressed="true">All ${type.members.length}</button><button data-method-filter="declared" aria-pressed="false">Declared ${type.declared.length}</button><button data-method-filter="inherited" aria-pressed="false">Inherited ${inherited.length}</button></div><div class="api-list" data-api-list>${rows}</div></section>
  </article>${navigation}</div>`);
  return layout(model, {
    title: data.title,
    description: data.summary,
    route: type.route,
    section: "Runtime/API",
    docs: true,
    sidebar: sidebar(model, data.id, type, data.module),
    body,
    kind: data.kind,
    module: data.module,
    status: data.status,
    availability: data.availability,
  });
}

function callableSections(
  model: SiteModel,
  page: ResolvedPage,
  member?: ResolvedMember,
): string {
  const data = page.source.data;
  if (data.kind !== "method" && data.kind !== "function")
    throw new Error("Expected callable");
  const parameters = data.parameters.length
    ? `<table class="api-table"><thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead><tbody>${data.parameters.map((item) => `<tr><td><code>$${escapeHtml(item.name)}</code></td><td><code>${escapeHtml(item.type)}</code></td><td>${escapeHtml(item.description)}</td></tr>`).join("")}</tbody></table>`
    : "<p>This callable has no parameters.</p>";
  const errors = data.errors.length
    ? data.errors
        .map(
          (item) =>
            `<div class="error-card">${item.type ? `<code>${escapeHtml(item.type)}</code> — ` : ""}${escapeHtml(item.description)}</div>`,
        )
        .join("")
    : "<p>This callable has no specified errors.</p>";
  const signatureHtml = member
    ? signature(model, member)
    : `<span class="kw">function</span> ${escapeHtml(data.name)}${data.typeParameters.length ? `&lt;${data.typeParameters.map((parameter) => escapeHtml(parameter.name)).join(", ")}&gt;` : ""}(${data.parameters
        .map((item) => {
          const defaultValue =
            item.default === undefined ? "" : ` = ${escapeHtml(item.default)}`;
          return `<span class="type">${escapeHtml(item.type)}</span> ${item.variadic ? "..." : ""}<span class="var">$${escapeHtml(item.name)}</span>${defaultValue}`;
        })
        .join(
          ", ",
        )}): <span class="type">${escapeHtml(data.returns.type)}</span>`;
  const related = data.related
    .map((id) => {
      const relatedPage = model.pages.find(
        (item) => item.source.data.id === id,
      );
      return relatedPage
        ? `<a class="symbol-card" href="${withBase(model.basePath, relatedPage.route)}"><strong>${escapeHtml(relatedPage.source.data.title)}</strong><span>${escapeHtml(relatedPage.source.data.summary)}</span></a>`
        : "";
    })
    .join("");
  return `<section class="article-section" id="signature"><h2>Signature</h2><div class="signature-block"><div class="synopsis-toolbar"><span>THP</span><button class="copy-button" data-copy-target="callable-signature">Copy</button></div><pre id="callable-signature"><code>${signatureHtml}</code></pre></div></section>
    <section class="article-section" id="parameters"><h2>Parameters</h2>${parameters}</section>
    <section class="article-section" id="returns"><h2>Return value</h2><div class="return-card"><code>${escapeHtml(data.returns.type)}</code><p>${escapeHtml(data.returns.description)}</p></div></section>
    <section class="article-section" id="errors"><h2>Errors</h2>${errors}</section>
    <section class="article-section prose" id="behavior"><h2>Behavior and examples</h2>${articleBody(model, page)}</section>
    ${related ? `<section class="article-section" id="related"><h2>Related symbols</h2><div class="symbol-grid">${related}</div></section>` : ""}`;
}

export function renderCallable(model: SiteModel, page: ResolvedPage): string {
  const data = page.source.data;
  if (data.kind !== "method" && data.kind !== "function")
    throw new Error("Expected callable");
  const owner =
    data.kind === "method" ? model.types.get(data.owner) : undefined;
  const member = owner?.declared.find(
    (item) => item.source.data.id === data.id,
  );
  const qualified = owner
    ? `${owner.source.data.name}::${data.name}`
    : data.name;
  const moduleName =
    data.kind === "function" ? data.module : owner!.source.data.module;
  const navigation = pageNavigation(
    model,
    [
      ["Signature", "#signature"],
      ["Parameters", "#parameters"],
      ["Return value", "#returns"],
      ["Errors", "#errors"],
      ["Behavior", "#behavior"],
    ],
    owner,
    data.id,
  );
  const callableTitle = owner
    ? `<p class="article-owner"><a href="${withBase(model.basePath, owner.route)}">${identifierWithBreaks(owner.source.data.name)}</a><span aria-hidden="true">::</span></p><div class="article-title-row"><h1 aria-label="${escapeHtml(qualified)}" data-pagefind-meta="title[aria-label]"><code>${identifierWithBreaks(data.name)}</code></h1></div>`
    : `<div class="article-title-row"><h1><code data-pagefind-meta="title">${identifierWithBreaks(data.name)}</code></h1><span class="kind-badge">${data.kind}</span></div>`;
  const body =
    trusted(`<div class="docs-content-grid"><article class="article" data-pagefind-body>
    ${breadcrumbs(model, [["Docs", "/"], ["Standard library", "/std/"], [model.config.modules[moduleName]?.name ?? moduleName, `/std/${moduleName}/`], ...(owner ? [[owner.source.data.name, owner.route] as [string, string]] : []), [data.name]])}
    <header class="article-header"><p class="content-kicker">${data.kind} reference</p>${availabilityBadge(data.availability)}${callableTitle}<p class="article-lead">${escapeHtml(data.summary)}</p><div class="meta-row"><span>THP ${escapeHtml(data.version)}</span></div></header>
    ${notice(data.status, data.notice)}${callableSections(model, page, member)}
  </article>${navigation}</div>`);
  return layout(model, {
    title: qualified,
    description: data.summary,
    route: page.route,
    section: "Runtime/API",
    docs: true,
    sidebar: sidebar(model, data.id, owner, moduleName),
    body,
    kind: data.kind,
    module: moduleName,
    status: data.status,
    availability: data.availability,
  });
}

export function renderPage(model: SiteModel, page: ResolvedPage): string {
  switch (page.source.data.kind) {
    case "home":
      return renderHome(model, page);
    case "guide":
      return renderGuide(model, page);
    case "module":
      return renderModule(model, page);
    case "class":
    case "interface":
    case "trait":
    case "enum":
      return renderType(model, model.types.get(page.source.data.id)!);
    case "method":
    case "function":
      return renderCallable(model, page);
  }
}
