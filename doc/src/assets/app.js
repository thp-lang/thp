const base = window.THP_DOCS_BASE || "/";
const icons = {
  copy: '<svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="8" y="8" width="11" height="11" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/></svg>',
  check:
    '<svg aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m5 12 4 4L19 6"/></svg>',
};

function setupTheme() {
  const theme =
    localStorage.getItem("thp-docs-theme") ||
    (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
  document.documentElement.dataset.theme = theme;
  document.querySelectorAll("[data-theme-toggle]").forEach((button) => {
    const update = () =>
      button.setAttribute(
        "aria-label",
        document.documentElement.dataset.theme === "dark"
          ? "Use light theme"
          : "Use dark theme",
      );
    update();
    button.addEventListener("click", () => {
      const next =
        document.documentElement.dataset.theme === "dark" ? "light" : "dark";
      document.documentElement.dataset.theme = next;
      localStorage.setItem("thp-docs-theme", next);
      update();
    });
  });
}

function setupCopyButtons() {
  document.querySelectorAll("[data-copy-target]").forEach((button) => {
    button.innerHTML = `${icons.copy}<span>Copy</span>`;
    button.addEventListener("click", async () => {
      const target = document.getElementById(button.dataset.copyTarget);
      if (!target) return;
      try {
        await navigator.clipboard.writeText(target.innerText);
      } catch {
        const range = document.createRange();
        range.selectNodeContents(target);
        const selection = getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
        document.execCommand("copy");
        selection.removeAllRanges();
      }
      button.innerHTML = `${icons.check}<span>Copied</span>`;
      setTimeout(
        () => (button.innerHTML = `${icons.copy}<span>Copy</span>`),
        1500,
      );
    });
  });
}

function setupCodeTabs() {
  document.querySelectorAll("[data-code-tabs]").forEach((group) => {
    group.querySelectorAll("[role=tab]").forEach((tab) => {
      tab.addEventListener("click", () => {
        group
          .querySelectorAll("[role=tab]")
          .forEach((item) =>
            item.setAttribute("aria-selected", String(item === tab)),
          );
        group
          .querySelectorAll("[role=tabpanel]")
          .forEach(
            (panel) =>
              (panel.hidden = panel.id !== tab.getAttribute("aria-controls")),
          );
      });
    });
  });
}

function setupMethodFilters() {
  const list = document.querySelector("[data-api-list]");
  const input = document.querySelector("[data-method-search]");
  if (!list || !input) return;
  let kind = "all";
  const rows = [...list.querySelectorAll("[data-api-row]")];
  const update = () => {
    const query = input.value.trim().toLowerCase();
    rows.forEach(
      (row) =>
        (row.hidden =
          (kind !== "all" && row.dataset.kind !== kind) ||
          !row.textContent.toLowerCase().includes(query)),
    );
  };
  document.querySelectorAll("[data-method-filter]").forEach((button) =>
    button.addEventListener("click", () => {
      kind = button.dataset.methodFilter;
      document
        .querySelectorAll("[data-method-filter]")
        .forEach((item) =>
          item.setAttribute("aria-pressed", String(item === button)),
        );
      update();
    }),
  );
  input.addEventListener("input", update);
}

function setupMobileNavigation() {
  const sidebar = document.querySelector("[data-docs-sidebar]");
  const button = document.querySelector("[data-mobile-menu]");
  const backdrop = document.querySelector("[data-sidebar-backdrop]");
  if (!sidebar || !button || !backdrop) return;
  const close = () => {
    sidebar.classList.remove("open");
    backdrop.hidden = true;
    button.setAttribute("aria-expanded", "false");
    button.focus();
  };
  button.addEventListener("click", () => {
    const open = sidebar.classList.toggle("open");
    backdrop.hidden = !open;
    button.setAttribute("aria-expanded", String(open));
    if (open) sidebar.querySelector("a")?.focus();
  });
  backdrop.addEventListener("click", close);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && sidebar.classList.contains("open")) close();
  });
}

function setupToc() {
  const links = [...document.querySelectorAll(".toc a[href^='#']")];
  if (!links.length || !("IntersectionObserver" in window)) return;
  const observer = new IntersectionObserver(
    (entries) =>
      entries.forEach((entry) => {
        if (entry.isIntersecting)
          links.forEach((link) =>
            link.classList.toggle(
              "active",
              link.hash === `#${entry.target.id}`,
            ),
          );
      }),
    { rootMargin: "-18% 0px -72% 0px" },
  );
  links
    .map((link) => document.querySelector(link.hash))
    .filter(Boolean)
    .forEach((item) => observer.observe(item));
}

function appendSearchExcerpt(target, excerpt) {
  const template = document.createElement("template");
  template.innerHTML = String(excerpt);
  const append = (parent, node) => {
    if (node.nodeType === Node.TEXT_NODE) {
      parent.append(node.textContent || "");
      return;
    }
    if (!(node instanceof Element)) return;
    const destination =
      node.tagName === "MARK" ? document.createElement("mark") : parent;
    [...node.childNodes].forEach((child) => append(destination, child));
    if (destination !== parent) parent.append(destination);
  };
  [...template.content.childNodes].forEach((node) => append(target, node));
}

function searchResultElement(item) {
  const destination = new URL(String(item.url), location.href);
  if (destination.origin !== location.origin) return undefined;
  const availability = ["implemented", "partial", "proposed"].includes(
    item.meta.availability,
  )
    ? item.meta.availability
    : "proposed";
  const link = document.createElement("a");
  link.className = "search-result";
  link.setAttribute(
    "href",
    `${destination.pathname}${destination.search}${destination.hash}`,
  );
  const content = document.createElement("span");
  const title = document.createElement("strong");
  title.textContent = String(item.meta.title);
  const excerpt = document.createElement("small");
  appendSearchExcerpt(excerpt, item.excerpt);
  content.append(title, excerpt);
  const badge = document.createElement("span");
  badge.className = `availability-badge availability-${availability}`;
  badge.textContent = availability;
  link.append(content, badge);
  return link;
}

async function setupSearch() {
  const dialog = document.querySelector("[data-search-dialog]");
  if (!dialog) return;
  const input = dialog.querySelector("[data-search-input]");
  const results = dialog.querySelector("[data-search-results]");
  const kind = dialog.querySelector("[data-search-kind]");
  const module = dialog.querySelector("[data-search-module]");
  const availability = dialog.querySelector("[data-search-availability]");
  const status = dialog.querySelector("[data-search-status]");
  let pagefind;
  let active = 0;
  const open = () => {
    dialog.showModal();
    requestAnimationFrame(() => input.focus());
  };
  const close = () => dialog.close();
  const search = async () => {
    const query = input.value.trim();
    if (!query) {
      results.innerHTML =
        '<p class="search-empty">Start typing to search the documentation.</p>';
      return;
    }
    const request = ++active;
    try {
      pagefind ??= await import(`${base}pagefind/pagefind.js`);
      await pagefind.options({ baseUrl: base });
      const filters = Object.fromEntries(
        [
          ["kind", kind.value],
          ["module", module.value],
          ["availability", availability.value],
          ["status", status.value],
        ].filter(([, value]) => value),
      );
      const searchResult = await pagefind.search(query, { filters });
      const availabilityRank = { implemented: 0, partial: 1, proposed: 2 };
      const data = (
        await Promise.all(
          searchResult.results.slice(0, 36).map((result) => result.data()),
        )
      )
        .sort(
          (left, right) =>
            (availabilityRank[left.meta.availability] ?? 3) -
            (availabilityRank[right.meta.availability] ?? 3),
        )
        .slice(0, 12);
      if (request !== active) return;
      const rendered = data.map(searchResultElement).filter(Boolean);
      if (rendered.length) results.replaceChildren(...rendered);
      else
        results.innerHTML =
          '<p class="search-empty">No matching documentation.</p>';
    } catch {
      results.innerHTML =
        '<p class="search-empty">Search index is available after <code>pnpm build</code>.</p>';
    }
  };
  document
    .querySelectorAll("[data-search-open]")
    .forEach((button) => button.addEventListener("click", open));
  dialog.querySelector("[data-search-close]").addEventListener("click", close);
  dialog.addEventListener("click", (event) => {
    if (event.target === dialog) close();
  });
  [input, kind, module, availability, status].forEach((control) =>
    control.addEventListener("input", search),
  );
  document.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      if (dialog.open) close();
      else open();
    } else if (
      event.key === "/" &&
      !dialog.open &&
      !["INPUT", "TEXTAREA"].includes(document.activeElement?.tagName)
    ) {
      event.preventDefault();
      open();
    }
  });
}

setupTheme();
setupCopyButtons();
setupCodeTabs();
setupMethodFilters();
setupMobileNavigation();
setupToc();
setupSearch();
if (location.hostname === "localhost") {
  new EventSource("/__reload").addEventListener("message", () =>
    location.reload(),
  );
}
