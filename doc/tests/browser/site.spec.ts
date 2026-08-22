import { AxeBuilder } from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";

async function expectArticleHeaderToFit(page: Page) {
  const fits = await page
    .locator(".article-header")
    .evaluate((header) => header.scrollWidth <= header.clientWidth);
  expect(fits).toBe(true);
}

test("long API headings fit their article column", async ({ page }) => {
  await page.goto(
    "/std/data-structures/spl-priority-queue/recover-from-corruption/",
  );
  const longMethodHeading = page.getByRole("heading", {
    name: "SplPriorityQueue::recoverFromCorruption",
  });
  await expect(longMethodHeading).toHaveText("recoverFromCorruption");
  await expect(longMethodHeading).toHaveAttribute(
    "data-pagefind-meta",
    "title[aria-label]",
  );
  await expect(page.locator(".article-owner a")).toHaveText("SplPriorityQueue");
  await expect(page.locator(".article-owner a")).toHaveAttribute(
    "href",
    "/std/data-structures/spl-priority-queue/",
  );
  await expect(page.locator(".article-header .kind-badge")).toHaveCount(0);
  await expect(page.locator(".meta-row")).not.toContainText("Declared by");
  await expectArticleHeaderToFit(page);

  for (const route of [
    "/std/iterators/recursive-iterator-iterator/get-recursive-iterator/",
    "/std/streams/unsupported-stream-operation-exception/",
  ]) {
    await page.goto(route);
    await expectArticleHeaderToFit(page);
  }
});

test("search results treat index metadata as untrusted", async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== "desktop", "desktop-only coverage");
  await page.route("**/pagefind/pagefind.js", async (route) => {
    await route.fulfill({
      contentType: "application/javascript",
      body: `
        export async function options() {}
        export async function search() {
          return {
            results: [
              {
                data: async () => ({
                  url: "/std/",
                  meta: {
                    title: "<img src=x onerror=alert(1)>Standard library",
                    availability: "implemented",
                  },
                  excerpt: "A <mark onclick=alert(1)>typed</mark><img src=x onerror=alert(1)> API",
                }),
              },
              {
                data: async () => ({
                  url: "https://example.com/",
                  meta: { title: "External", availability: "implemented" },
                  excerpt: "External result",
                }),
              },
            ],
          };
        }
      `,
    });
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Search documentation" }).click();
  await page.locator("[data-search-input]").fill("typed");
  const result = page.locator(".search-result");
  await expect(result).toHaveCount(1);
  await expect(result).toContainText(
    "<img src=x onerror=alert(1)>Standard library",
  );
  await expect(result.locator("img")).toHaveCount(0);
  await expect(result.locator("mark")).toHaveText("typed");
  await expect(result.locator("mark")).not.toHaveAttribute("onclick");
});

test("desktop class interactions and accessibility", async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== "desktop", "desktop-only coverage");
  await page.goto("/std/");
  await expect(
    page.getByRole("heading", { name: "Standard library" }),
  ).toBeVisible();
  await expect(page.locator(".experimental-note")).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Async" }).first(),
  ).toHaveAttribute("href", "/std/async/");
  await expect(
    page.getByRole("link", { name: "Exceptions" }).first(),
  ).toHaveAttribute("href", "/std/exceptions/");
  await expect(
    page.getByRole("link", { name: "Iterators" }).first(),
  ).toHaveAttribute("href", "/std/iterators/");
  await expect(
    page.getByRole("link", { name: "Data structures" }).first(),
  ).toHaveAttribute("href", "/std/data-structures/");
  await expect(
    page.getByRole("link", { name: "Filesystem" }).first(),
  ).toHaveAttribute("href", "/std/filesystem/");
  await expect(
    page.getByRole("link", { name: "Streams" }).first(),
  ).toHaveAttribute("href", "/std/streams/");
  await expect(
    page.getByRole("link", { name: "Bundled extensions" }).first(),
  ).toHaveAttribute("href", "/std/extensions/");
  await page.screenshot({
    path: testInfo.outputPath("standard-library-overview-desktop.png"),
    fullPage: true,
  });

  await page
    .locator(".header-nav")
    .getByRole("link", { name: "Runtime/API" })
    .click();
  await expect(page).toHaveURL("/std/");
  await expect(
    page.getByRole("heading", { name: "Standard library" }),
  ).toBeVisible();

  await page.goto("/std/iterators/");
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "SplFileObject" }),
  ).toHaveCount(0);

  await page.goto("/std/filesystem/");
  await expect(page.getByRole("heading", { name: "Filesystem" })).toBeVisible();
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "Filesystem" })
      .first(),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "SplFileInfo" }),
  ).toBeVisible();
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "SplFileObject" }),
  ).toBeVisible();
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "SplTempFileObject" }),
  ).toBeVisible();

  await page.goto("/std/streams/");
  await expect(page.getByRole("heading", { name: "Streams" })).toBeVisible();
  await expect(page.locator(".experimental-note")).toBeVisible();
  await expect(page.locator(".table-scroll").first()).toBeVisible();
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "Streams" })
      .first(),
  ).toHaveAttribute("aria-current", "page");

  await page.goto("/std/streams/memory-stream/");
  await expect(
    page.getByRole("heading", { name: "MemoryStream" }),
  ).toBeVisible();
  await expect(page.locator("[data-api-row]")).toHaveCount(14);
  await expect(page.locator("#class-synopsis")).toContainText(
    "implements ReadableStream, WritableStream, SeekableStream",
  );
  await expect(page.locator("#class-synopsis")).toContainText(
    "// Implementing ReadableStream",
  );
  await expect(page.locator("#class-synopsis")).not.toContainText(
    "// Inherited from ReadableStream",
  );
  for (const kind of ["class", "interface", "enum", "exception"])
    await expect(
      page.locator(`[data-docs-sidebar] [data-symbol-kind="${kind}"]`).first(),
    ).toBeVisible();
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByText("Streams Types", { exact: true }),
  ).toBeVisible();

  await page.goto("/std/data-structures/spl-queue/");
  await expect(page.getByRole("heading", { name: "SplQueue" })).toBeVisible();
  await expect(page.locator("[data-api-row]")).toHaveCount(20);
  const sidebar = page.locator("[data-docs-sidebar]");
  await expect(
    sidebar.getByText("Standard library", { exact: false }),
  ).toBeVisible();
  await expect(
    sidebar.getByText("Exceptions", { exact: true }).first(),
  ).toBeVisible();
  await expect(
    sidebar.getByText("Iterators", { exact: true }).first(),
  ).toBeVisible();
  await expect(
    sidebar.getByText("Data structures", { exact: true }).first(),
  ).toBeVisible();
  await expect(
    sidebar.getByText("Data structures Types", { exact: true }),
  ).toBeVisible();
  await expect(
    page.locator(".toc").getByText("On this class", { exact: true }),
  ).toBeVisible();
  await expect(sidebar.getByRole("link", { name: "SplQueue" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  const rightPanel = page.locator(".toc");
  await expect(
    rightPanel.getByRole("link", { name: "enqueue()" }),
  ).toBeVisible();
  await expect(
    rightPanel.getByRole("link", { name: "dequeue()" }),
  ).toBeVisible();
  await expect(
    rightPanel.getByRole("link", { name: "__construct()" }),
  ).toHaveCount(0);
  await expect(sidebar.locator(".method-link")).toHaveCount(0);
  await expect(rightPanel.locator(".method-link")).toHaveCount(19);
  await expect(
    rightPanel.locator('.method-link[data-inherited="true"]'),
  ).toHaveCount(17);
  await expect(rightPanel.getByRole("link", { name: "pop()" })).toHaveAttribute(
    "href",
    "/std/data-structures/spl-doubly-linked-list/pop/",
  );
  await expect(
    rightPanel.getByRole("link", { name: "getIterator()" }),
  ).toBeVisible();
  const typography = await page.evaluate(() => {
    const body = getComputedStyle(document.body);
    const title = getComputedStyle(document.querySelector(".article h1 code")!);
    const method = getComputedStyle(
      document.querySelector(".toc .method-link")!,
    );
    return {
      bodyFamily: body.fontFamily,
      bodySize: body.fontSize,
      bodyLineHeight: body.lineHeight,
      titleFamily: title.fontFamily,
      titleWeight: title.fontWeight,
      methodFamily: method.fontFamily,
    };
  });
  expect(typography.bodyFamily).toContain("Inter");
  expect(typography.bodySize).toBe("16px");
  expect(typography.bodyLineHeight).toBe("26.4px");
  expect(typography.titleFamily).toBe(typography.bodyFamily);
  expect(typography.titleWeight).toBe("760");
  expect(typography.methodFamily).toContain("SFMono-Regular");
  const synopsis = page.locator("#class-synopsis");
  await expect(synopsis).toContainText("extends SplDoublyLinkedList<T>");
  await expect(synopsis).toContainText(
    "implements IteratorAggregate<int, T>, Countable, MapAccess<int, T>",
  );
  await expect(synopsis).not.toContainText("// Declared by SplQueue");
  await expect(synopsis).toContainText(
    "// Inherited from SplDoublyLinkedList<T>",
  );
  await expect(page.locator(".synopsis-legend")).toHaveCount(0);
  const memberTops = await synopsis
    .locator('[data-member-origin="declared"]')
    .evaluateAll((members) =>
      members.map((member) => member.getBoundingClientRect().top),
    );
  expect(memberTops[1]! - memberTops[0]!).toBeLessThan(35);
  expect(memberTops[2]! - memberTops[1]!).toBeLessThan(35);
  const markerContent = await synopsis
    .locator('[data-member-origin="declared"]')
    .first()
    .evaluate((member) => getComputedStyle(member, "::before").content);
  expect(markerContent).toBe("none");

  await page.getByRole("button", { name: "Inherited 17" }).click();
  await expect(
    page.locator('[data-api-row][data-kind="declared"]:visible'),
  ).toHaveCount(0);
  await expect(
    page.locator('[data-api-row][data-kind="inherited"]:visible'),
  ).toHaveCount(17);

  await page.getByRole("button", { name: "Copy" }).first().click();
  await expect(
    page.getByRole("button", { name: "Copied" }).first(),
  ).toBeVisible();

  await page.getByRole("button", { name: "Use dark theme" }).click();
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  await page.screenshot({
    path: testInfo.outputPath("spl-queue-desktop.png"),
    fullPage: true,
  });
  await page.goto("/");
  await expect(
    page.getByRole("heading", {
      name: "A typed language with its own runtime.",
    }),
  ).toBeVisible();
  await expect(page.locator(".code-window")).toContainText("thp run hello.thp");
  await expect(page.locator(".code-window")).toContainText("Hello, world!");
  await expect(
    page.getByRole("heading", { name: "When to choose THP over PHP" }),
  ).toBeVisible();
  const comparison = page.locator(".home-prose .table-scroll").first();
  await expect(comparison).toContainText("vector<T>");
  await expect(comparison).toContainText("map<K, V>");
  await expect(comparison).toContainText("Magic array-key conversion");
  await expect(comparison).toContainText("Loose == type juggling");
  await expect(comparison).toContainText("✅ Yes");
  await expect(comparison).toContainText("❌ No");
  await expect(
    page.getByRole("heading", { name: "More than a syntax experiment" }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("heading", { name: "What runs today" }),
  ).toBeVisible();
  await expect(
    page.locator(".home-prose > ul").first().locator("li"),
  ).toHaveCount(6);
  await expect(
    page.getByRole("link", { name: "See what works today" }),
  ).toHaveAttribute("href", "/learn/implementation-status/");
  await expect(
    page.locator(".header-nav").getByRole("link", { name: "Language" }),
  ).toHaveAttribute("href", "/language/overview/");
  await expect(
    page.locator(".header-nav").getByRole("link", {
      name: "Design Proposals",
    }),
  ).toHaveCount(0);
  await expect(
    page.locator(".header-nav").getByRole("link", { name: "Internals" }),
  ).toHaveAttribute("href", "/internals/overview/");
  const homeAccessibility = await new AxeBuilder({ page }).analyze();
  expect(homeAccessibility.violations).toEqual([]);
  await page.screenshot({
    path: testInfo.outputPath("home-desktop.png"),
    fullPage: true,
  });
  await page.goto("/internals/overview/");
  await expect(page.getByRole("heading", { name: "Overview" })).toBeVisible();
  await expect(
    page.locator(".header-nav").getByRole("link", { name: "Internals" }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "Runtime design" }),
  ).toBeVisible();
  await page.screenshot({
    path: testInfo.outputPath("internals-overview-desktop.png"),
    fullPage: true,
  });
  await page.goto("/language/types/");
  await expect(page.locator(".availability-partial").first()).toHaveText(
    "partial",
  );
  await expect(page.locator(".experimental-note")).toContainText(
    "Experimental language",
  );
  const languageTable = page.locator(".table-scroll").first();
  await expect(languageTable).toBeVisible();
  const tableStyles = await languageTable.evaluate((wrapper) => {
    const heading = wrapper.querySelector("th")!;
    const cell = wrapper.querySelector("td")!;
    return {
      overflow: getComputedStyle(wrapper).overflowX,
      headingBorder: getComputedStyle(heading).borderBottomWidth,
      cellBorder: getComputedStyle(cell).borderRightWidth,
    };
  });
  expect(tableStyles).toEqual({
    overflow: "auto",
    headingBorder: "1px",
    cellBorder: "1px",
  });
  await page.screenshot({
    path: testInfo.outputPath("language-types-desktop.png"),
    fullPage: true,
  });
  await page.goto("/language/attributes/");
  await expect(page.locator(".availability-proposed").first()).toHaveText(
    "proposed",
  );
  await expect(page.locator("[data-search-availability]")).toHaveCount(1);
  await page.goto("/std/data-structures/spl-queue/dequeue/");
  await page.screenshot({
    path: testInfo.outputPath("method-desktop.png"),
    fullPage: true,
  });
  await page.goto("/std/async/functions/async/");
  await expect(page.locator("#callable-signature")).toContainText(
    "function async<T>(callable $function, mixed ...$arguments): Coroutine<T>",
  );
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "async", exact: true }),
  ).toHaveAttribute("aria-current", "page");
  await expect(
    page.locator('[data-docs-sidebar] [data-symbol-kind="function"]').first(),
  ).toBeVisible();
  await page.goto("/std/extensions/");
  await expect(page.locator(".table-scroll")).toContainText(
    "PDO PostgreSQL driver",
  );
});

test("mobile navigation, search, and keyboard access", async ({
  page,
}, testInfo) => {
  test.skip(testInfo.project.name !== "mobile", "mobile-only coverage");
  await page.goto("/std/data-structures/spl-queue/dequeue/");
  await page
    .getByRole("button", { name: "Open documentation navigation" })
    .click();
  await expect(page.locator("[data-docs-sidebar]")).toHaveClass(/open/);
  await page.keyboard.press("Escape");
  await expect(page.locator("[data-docs-sidebar]")).not.toHaveClass(/open/);

  await page.goto("/internals/overview/");
  await page
    .getByRole("button", { name: "Open documentation navigation" })
    .click();
  await expect(
    page
      .locator("[data-docs-sidebar]")
      .getByRole("link", { name: "Bytecode interpreter" }),
  ).toBeVisible();
  await page.keyboard.press("Escape");

  await page.goto("/std/data-structures/spl-queue/dequeue/");
  await page.keyboard.press("Control+k");
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.locator("[data-search-input]").fill("first-in first-out");
  await expect(page.locator("[data-search-results]")).toContainText("SplQueue");
  await page.locator("[data-search-kind]").selectOption("function");
  await page.locator("[data-search-input]").fill("spl_autoload");
  await expect(page.locator("[data-search-results]")).toContainText(
    "spl_autoload",
  );
  await page.locator("[data-search-kind]").selectOption("method");
  await page.locator("[data-search-input]").fill("dequeue");
  await expect(page.locator("[data-search-results]")).toContainText(
    "SplQueue::dequeue",
  );
  await page.screenshot({
    path: testInfo.outputPath("method-mobile.png"),
    fullPage: true,
  });
});
