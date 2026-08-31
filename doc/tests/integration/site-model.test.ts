import { readFile } from "node:fs/promises";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { build, createModel } from "../../src/build.js";
import { renderPage } from "../../src/view/pages.js";

const root = path.resolve(import.meta.dirname, "../..");

describe("documentation site model", () => {
  it("marks every Throwable class hierarchy as an exception", async () => {
    const model = await createModel({ root, basePath: "/typed-php/" });
    const page = model.pages.find(
      (item) => item.source.data.id === "std.baseTypes.Throwable",
    )!;
    const html = renderPage(model, page);
    const sidebarLink = (route: string, name: string, kind: string) =>
      `<a class="symbol-link" href="/typed-php${route}"><span class="symbol-kind-icon symbol-kind-${kind}" data-symbol-kind="${kind}" aria-hidden="true">${kind === "exception" ? "X" : "C"}</span><span class="sidebar-symbol-name">${name}</span></a>`;

    expect(html).toContain(
      sidebarLink("/std/base-types/exception/", "Exception", "exception"),
    );
    expect(html).toContain(
      sidebarLink("/std/base-types/error/", "Error", "exception"),
    );
    expect(html).toContain(
      sidebarLink(
        "/std/base-types/unhandled-match-error/",
        "UnhandledMatchError",
        "exception",
      ),
    );
    expect(html).toContain(
      sidebarLink("/std/base-types/trace-line/", "TraceLine", "class"),
    );
  });

  it("resolves the complete SplQueue synopsis to canonical owners", async () => {
    const model = await createModel({ root });
    const queue = model.types.get("std.spl.SplQueue")!;
    expect(queue.members).toHaveLength(20);
    expect(
      queue.members.map((member) => [
        member.source.data.name,
        member.inherited ? member.declaringType.source.data.name : "SplQueue",
      ]),
    ).toMatchInlineSnapshot(`
      [
        [
          "__construct",
          "SplQueue",
        ],
        [
          "enqueue",
          "SplQueue",
        ],
        [
          "dequeue",
          "SplQueue",
        ],
        [
          "add",
          "SplDoublyLinkedList",
        ],
        [
          "pop",
          "SplDoublyLinkedList",
        ],
        [
          "shift",
          "SplDoublyLinkedList",
        ],
        [
          "push",
          "SplDoublyLinkedList",
        ],
        [
          "unshift",
          "SplDoublyLinkedList",
        ],
        [
          "top",
          "SplDoublyLinkedList",
        ],
        [
          "bottom",
          "SplDoublyLinkedList",
        ],
        [
          "count",
          "SplDoublyLinkedList",
        ],
        [
          "isEmpty",
          "SplDoublyLinkedList",
        ],
        [
          "setIteratorMode",
          "SplDoublyLinkedList",
        ],
        [
          "getIteratorMode",
          "SplDoublyLinkedList",
        ],
        [
          "offsetExists",
          "SplDoublyLinkedList",
        ],
        [
          "offsetGet",
          "SplDoublyLinkedList",
        ],
        [
          "offsetSet",
          "SplDoublyLinkedList",
        ],
        [
          "offsetUnset",
          "SplDoublyLinkedList",
        ],
        [
          "toVector",
          "SplDoublyLinkedList",
        ],
        [
          "getIterator",
          "SplDoublyLinkedList",
        ],
      ]
    `);
    expect(
      queue.members.find((member) => member.source.data.name === "pop")?.route,
    ).toBe("/std/data-structures/spl-doubly-linked-list/pop/");
    expect(
      model.pages.some(
        (page) => page.route === "/std/data-structures/spl-queue/pop/",
      ),
    ).toBe(false);
  });

  it("renders every callable once and resolves every synopsis link", async () => {
    const model = await createModel({ root });
    const callablePages = model.pages.filter(
      (page) =>
        page.source.data.kind === "method" ||
        page.source.data.kind === "function",
    );
    expect(new Set(callablePages.map((page) => page.route)).size).toBe(
      callablePages.length,
    );
    for (const type of model.types.values())
      for (const member of type.members)
        expect(model.pages.some((page) => page.route === member.route)).toBe(
          true,
        );
  });

  it("contains the migrated standard-library corpus and module navigation", async () => {
    const model = await createModel({ root, basePath: "/typed-php/" });
    const standardLibraryPages = model.pages.filter((page) =>
      page.route.startsWith("/std/"),
    );
    const byKind = (kind: string) =>
      standardLibraryPages.filter((page) => page.source.data.kind === kind);

    expect(standardLibraryPages).toHaveLength(456);
    expect(byKind("module")).toHaveLength(9);
    expect(byKind("class").length + byKind("interface").length).toBe(91);
    expect(byKind("enum")).toHaveLength(3);
    expect(byKind("method")).toHaveLength(330);
    expect(byKind("function")).toHaveLength(23);
    expect(
      byKind("module").map((page) => [page.source.data.id, page.route]),
    ).toEqual([
      ["std.async", "/std/async/"],
      ["std.baseTypes", "/std/base-types/"],
      ["std.dataStructures", "/std/data-structures/"],
      ["std.exceptions", "/std/exceptions/"],
      ["std.extensions", "/std/extensions/"],
      ["std.filesystem", "/std/filesystem/"],
      ["std.index", "/std/"],
      ["std.iterators", "/std/iterators/"],
      ["std.streams", "/std/streams/"],
    ]);
    expect(model.config.navigation).toContainEqual({
      label: "Runtime/API",
      href: "/std/",
    });
    const overview = model.pages.find(
      (page) => page.source.data.id === "std.index",
    )!;
    expect(renderPage(model, overview)).toContain(
      '<a href="/typed-php/std/" aria-current="page">Runtime/API</a>',
    );

    const categories = [
      ["std.exceptions", "/std/exceptions/"],
      ["std.iterators", "/std/iterators/"],
      ["std.dataStructures", "/std/data-structures/"],
      ["std.filesystem", "/std/filesystem/"],
    ] as const;
    for (const [id, route] of categories) {
      const category = model.pages.find((page) => page.source.data.id === id)!;
      expect(category.route).toBe(route);
      expect(renderPage(model, category)).toContain(
        `href="/typed-php${route}" aria-current="page"`,
      );
    }
    expect(model.types.get("std.spl.LogicException")?.source.data.module).toBe(
      "exceptions",
    );
    expect(model.types.get("std.spl.CachingIterator")?.source.data.module).toBe(
      "iterators",
    );
    for (const name of [
      "FileLockResult",
      "SplFileInfo",
      "SplFileObject",
      "SplTempFileObject",
    ])
      expect(model.types.get(`std.spl.${name}`)?.source.data.module).toBe(
        "filesystem",
      );
    expect(model.types.get("std.spl.SplQueue")?.source.data.module).toBe(
      "data-structures",
    );

    const streams = model.pages.find(
      (page) => page.source.data.id === "std.streams",
    )!;
    const streamsHtml = renderPage(model, streams);
    expect(streams.route).toBe("/std/streams/");
    expect(streamsHtml).toContain(
      'href="/typed-php/std/streams/" aria-current="page">Streams</a>',
    );
    expect(streamsHtml).toContain(
      '<div class="experimental-note"><strong>Experimental API</strong>',
    );
    expect(streamsHtml).toContain('<div class="table-scroll"><table>');
    expect(streamsHtml).toContain("Native handle ownership");

    const memoryStream = model.types.get("std.streams.MemoryStream")!;
    expect(memoryStream.members).toHaveLength(14);
    expect(
      memoryStream.declared.map((member) => member.source.data.name),
    ).toEqual(["open"]);
    expect(
      memoryStream.members.map((member) => member.source.data.name),
    ).toEqual(
      expect.arrayContaining([
        "close",
        "isClosed",
        "read",
        "readAll",
        "eof",
        "write",
        "writeAll",
        "flush",
        "seek",
        "tell",
      ]),
    );
    const memoryPage = model.pages.find(
      (page) => page.source.data.id === "std.streams.MemoryStream",
    )!;
    const memoryHtml = renderPage(model, memoryPage);
    expect(memoryHtml).toContain(
      '<span class="kw">public static function</span> <a href="/typed-php/std/streams/memory-stream/open/">open</a>',
    );
    expect(memoryHtml).toContain("// Implementing ReadableStream");
    expect(memoryHtml).not.toContain("// Inherited from ReadableStream");
    for (const kind of ["class", "interface", "enum", "exception"])
      expect(memoryHtml).toContain(`data-symbol-kind="${kind}"`);

    const readableStreamPage = model.pages.find(
      (page) => page.source.data.id === "std.streams.ReadableStream",
    )!;
    const readableStreamHtml = renderPage(model, readableStreamPage);
    expect(readableStreamHtml).toContain(
      '<div class="toc-section toc-members"><strong>On this interface</strong>',
    );
    expect(readableStreamHtml).toContain(
      'class="method-link" href="/typed-php/std/streams/readable-stream/read/">read()</a>',
    );
    expect(readableStreamHtml).not.toContain(
      '<p class="sidebar-label">On this interface</p>',
    );

    const openMode = model.pages.find(
      (page) => page.source.data.id === "std.streams.OpenMode",
    )!;
    expect(renderPage(model, openMode)).toContain(
      '<span class="kw">case</span> ReadWriteAppend;',
    );

    const asyncFunction = model.pages.find(
      (page) => page.source.data.id === "std.async.async",
    )!;
    const asyncHtml = renderPage(model, asyncFunction);
    expect(asyncHtml).toContain(
      "schedules a callable to run in a new coroutine",
    );
    expect(asyncHtml).toContain(
      '<span class="kw">function</span> async&lt;T&gt;',
    );
    expect(asyncHtml).toContain(
      '<span class="type">mixed</span> ...<span class="var">$arguments</span>',
    );
    expect(asyncHtml).toContain('data-symbol-kind="function"');
    expect(asyncHtml).toContain(
      'href="/typed-php/std/async/functions/async/" aria-current="page"',
    );

    const extensions = model.pages.find(
      (page) => page.source.data.id === "std.extensions",
    )!;
    const extensionsHtml = renderPage(model, extensions);
    expect(extensionsHtml).toContain("PDO PostgreSQL driver");
    expect(extensionsHtml).toContain('<div class="table-scroll"><table>');
    expect(extensionsHtml).toContain(
      '<div class="experimental-note"><strong>Experimental API</strong>',
    );

    const caching = model.types.get("std.spl.CachingIterator")!;
    expect(
      caching.source.data.constants.map((constant) => constant.name),
    ).toContain("FULL_CACHE");
    const cachingPage = model.pages.find(
      (page) => page.source.data.id === "std.spl.CachingIterator",
    )!;
    expect(renderPage(model, cachingPage)).toContain(
      '<section class="article-section" id="constants">',
    );

    const traceLine = model.types.get("std.baseTypes.TraceLine")!;
    expect(traceLine.source.data.properties).toHaveLength(7);
    const traceLinePage = model.pages.find(
      (page) => page.source.data.id === "std.baseTypes.TraceLine",
    )!;
    expect(renderPage(model, traceLinePage)).toContain(
      '<section class="article-section" id="properties">',
    );
  });

  it("renders valid root and GitHub Pages base-path output", async () => {
    const rootModel = await build({
      root,
      basePath: "/",
      siteUrl: "https://example.github.io",
    });
    expect(rootModel.pages).toHaveLength(493);
    const subpathModel = await build({
      root,
      basePath: "/typed-php/",
      siteUrl: "https://example.github.io",
    });
    const queuePage = subpathModel.pages.find(
      (page) => page.source.data.id === "std.spl.SplQueue",
    )!;
    const html = renderPage(subpathModel, queuePage);
    expect(html).toContain(
      'href="/typed-php/std/data-structures/spl-doubly-linked-list/pop/"',
    );
    expect(html).toContain(
      'href="https://example.github.io/typed-php/std/data-structures/spl-queue/"',
    );
    expect(html).toContain(
      '<span class="type">SplDoublyLinkedList&lt;T&gt;</span></a>\n    <span class="kw">implements</span>',
    );
    expect(html).toContain(
      '<span class="kw">implements</span> <a href="/typed-php/std/base-types/iterator-aggregate/"><span class="type">IteratorAggregate&lt;int, T&gt;</span></a>, <a href="/typed-php/std/base-types/countable/"><span class="type">Countable</span></a>, <a href="/typed-php/std/base-types/map-access/"><span class="type">MapAccess&lt;int, T&gt;</span></a>',
    );
    expect(html).not.toContain("// Declared by SplQueue");
    expect(html).toContain(
      '<span class="comment">// Inherited from SplDoublyLinkedList&lt;T&gt;</span>',
    );
    expect(html).toMatch(
      /data-member-origin="declared">[^]*?dequeue[^]*?;<\/span>\n\n {4}<span class="comment">\/\/ Inherited from/,
    );
    expect(html).toContain(
      '<a href="/typed-php/std/data-structures/spl-queue/construct/">__construct</a>();',
    );
    expect(html).not.toContain(
      '<a href="/typed-php/std/data-structures/spl-queue/construct/">__construct</a>():',
    );
    expect(html).toContain(
      '<p class="sidebar-label">Standard library <span class="sidebar-badge">THP</span></p>',
    );
    expect(html).toContain('href="/typed-php/std/async/">Async</a>');
    expect(html).toContain('href="/typed-php/std/exceptions/">Exceptions</a>');
    expect(html).toContain('href="/typed-php/std/iterators/">Iterators</a>');
    expect(html).toContain(
      'href="/typed-php/std/data-structures/" aria-current="page">Data structures</a>',
    );
    expect(html).toContain(
      '<p class="sidebar-label">Data structures Types</p>',
    );
    expect(html).not.toContain('<p class="sidebar-label">Exceptions</p>');
    expect(html).not.toContain('<p class="sidebar-label">Iterators</p>');
    expect(html).toContain(
      'class="symbol-link" href="/typed-php/std/data-structures/spl-doubly-linked-list/"',
    );
    expect(html).toContain(
      '<span class="sidebar-symbol-name">SplDoublyLinkedList</span>',
    );
    expect(html).not.toContain('<p class="sidebar-label">On this class</p>');
    expect(html).toContain(
      '<div class="toc-section toc-members"><strong>On this class</strong>',
    );
    expect(html).toContain(
      'href="/typed-php/std/data-structures/spl-queue/enqueue/">enqueue()</a>',
    );
    expect(html).toContain(
      'href="/typed-php/std/data-structures/spl-doubly-linked-list/pop/" data-inherited="true">pop()</a>',
    );
    expect(html).toContain(
      'href="/typed-php/std/data-structures/spl-doubly-linked-list/get-iterator/" data-inherited="true">getIterator()</a>',
    );
    expect(html).not.toContain(">inherited <span");
    expect(html).not.toContain(
      'class="method-link" href="/typed-php/std/data-structures/spl-queue/construct/"',
    );
    expect(html).not.toContain('href="/std/spl/');

    const dequeuePage = subpathModel.pages.find(
      (page) => page.source.data.id === "std.spl.SplQueue::dequeue",
    )!;
    const dequeueHtml = renderPage(subpathModel, dequeuePage);
    expect(dequeueHtml).toContain(
      'class="method-link" href="/typed-php/std/data-structures/spl-queue/dequeue/" aria-current="page">dequeue()</a>',
    );
    expect(dequeueHtml).not.toContain(
      '<p class="sidebar-label">On this class</p>',
    );

    const legacyQueue = await readFile(
      path.join(root, "dist/std/spl/spl-queue/index.html"),
      "utf8",
    );
    expect(legacyQueue).toContain(
      'content="0; url=/typed-php/std/data-structures/spl-queue/"',
    );

    const movedFileObject = await readFile(
      path.join(root, "dist/std/iterators/spl-file-object/index.html"),
      "utf8",
    );
    expect(movedFileObject).toContain(
      'content="0; url=/typed-php/std/filesystem/spl-file-object/"',
    );
    const movedFileMethod = await readFile(
      path.join(
        root,
        "dist/std/iterators/spl-file-info/get-filename/index.html",
      ),
      "utf8",
    );
    expect(movedFileMethod).toContain(
      'content="0; url=/typed-php/std/filesystem/spl-file-info/get-filename/"',
    );
  });

  it("renders the complete language reference with ordered navigation", async () => {
    const model = await createModel({ root, basePath: "/typed-php/" });
    const languagePages = model.pages
      .filter(
        (page) =>
          page.source.data.kind === "guide" &&
          page.source.data.nav.section === "language",
      )
      .sort((a, b) => {
        if (a.source.data.kind !== "guide" || b.source.data.kind !== "guide")
          return 0;
        return a.source.data.nav.order - b.source.data.nav.order;
      });

    expect(languagePages).toHaveLength(19);
    expect(languagePages.map((page) => page.source.data.title)).toEqual([
      "Overview",
      "Basic syntax",
      "Types",
      "Variables",
      "Constants",
      "Expressions",
      "Operators",
      "Control structures",
      "Functions",
      "Classes and objects",
      "Namespaces",
      "Enumerations",
      "Errors",
      "Exceptions",
      "Generators",
      "Attributes",
      "References",
      "Predefined variables",
      "Resources and streams",
    ]);

    const resources = languagePages.at(-1)!;
    const html = renderPage(model, resources);
    expect(resources.route).toBe("/language/resources-and-streams/");
    expect(html).toContain("Language reference");
    expect(html).toContain(
      'href="/typed-php/language/resources-and-streams/" aria-current="page"',
    );
    expect(html).toContain('href="/typed-php/language/types/">Types</a>');
    expect(html).toContain(
      '<div class="experimental-note"><strong>Experimental language</strong>',
    );
    expect(html).toContain(
      "Memory, temporary, URI, and read-only file streams now execute",
    );
    expect(html).toContain('<div class="table-scroll"><table>');
    expect(html).not.toContain("<blockquote>");
    expect(html).not.toContain("Standard library <span");
  });

  it("renders the complete internals section with ordered navigation", async () => {
    const model = await createModel({ root, basePath: "/typed-php/" });
    const internalsPages = model.pages
      .filter(
        (page) =>
          page.source.data.kind === "guide" &&
          page.source.data.nav.section === "internals",
      )
      .sort((a, b) => {
        if (a.source.data.kind !== "guide" || b.source.data.kind !== "guide")
          return 0;
        return a.source.data.nav.order - b.source.data.nav.order;
      });

    expect(internalsPages.map((page) => page.source.data.title)).toEqual([
      "Overview",
      "Source loading and project discovery",
      "Lexing",
      "Parsing and AST",
      "Modules and name resolution",
      "Type analysis and HIR",
      "Control-flow lowering and MIR",
      "Bytecode generation and linking",
      "Bytecode verification",
      "Bytecode interpreter",
      "OPcache and JIT",
      "Runtime design",
    ]);

    const overview = internalsPages[0]!;
    const html = renderPage(model, overview);
    expect(overview.route).toBe("/internals/overview/");
    expect(html).toContain(
      '<p class="sidebar-label">Internals <span class="sidebar-badge">THP</span></p>',
    );
    expect(html).toContain(
      'href="/typed-php/internals/overview/" aria-current="page"',
    );
    expect(html).toContain(
      'href="/typed-php/internals/runtime-design/">Runtime design</a>',
    );
    expect(html).toContain(
      '<div class="experimental-note"><strong>Experimental implementation</strong>',
    );
    expect(html).toContain(
      '<a href="/typed-php/internals/overview/" aria-current="page">Internals</a>',
    );
    expect(model.config.navigation).toContainEqual({
      label: "Internals",
      href: "/internals/overview/",
    });
    expect(
      model.config.navigation.some((item) => item.label === "Project"),
    ).toBe(false);

    const home = model.pages.find(
      (page) => page.source.data.id === "docs.home",
    )!;
    const homeHtml = renderPage(model, home);
    expect(homeHtml).toContain(
      '<h2 id="choose-your-path">Choose your path</h2>',
    );
    expect(homeHtml).toContain('href="/typed-php/internals/overview/"');
    expect(homeHtml).not.toContain('href="/typed-php/#project"');
  });
});
