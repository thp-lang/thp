import type { PageData } from "./schema.js";

export function slug(value: string): string {
  return value
    .replace(/::/g, "-")
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/[^A-Za-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
    .toLowerCase();
}

export function routeFor(data: PageData, owner?: PageData): string {
  switch (data.kind) {
    case "home":
      return "/";
    case "guide":
      return `/${data.nav.section}/${slug(data.title)}/`;
    case "module":
      return data.module === "standard-library"
        ? "/std/"
        : `/std/${data.module}/`;
    case "class":
    case "interface":
    case "trait":
    case "enum":
      return `/std/${data.module}/${slug(data.name)}/`;
    case "method": {
      if (!owner || !("module" in owner) || !("name" in owner))
        throw new Error(
          `Cannot route method ${data.id}: owner was not resolved`,
        );
      return `/std/${owner.module}/${slug(owner.name)}/${slug(data.name.replace(/^__/, ""))}/`;
    }
    case "function":
      return `/std/${data.module}/functions/${slug(data.name)}/`;
  }
}

export function normalizeBasePath(value: string | undefined): string {
  if (!value || value === "/") return "/";
  return `/${value.replace(/^\/+|\/+$/g, "")}/`;
}

export function withBase(basePath: string, route: string): string {
  if (/^(?:https?:|mailto:|#)/.test(route)) return route;
  const normalized = route.startsWith("/") ? route.slice(1) : route;
  return basePath === "/" ? `/${normalized}` : `${basePath}${normalized}`;
}

export function outputPath(route: string): string {
  return route === "/" ? "index.html" : `${route.slice(1)}index.html`;
}
