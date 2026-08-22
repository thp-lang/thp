import { describe, expect, it } from "vitest";
import type { PageData } from "../../src/schema.js";
import {
  normalizeBasePath,
  outputPath,
  routeFor,
  slug,
  withBase,
} from "../../src/route.js";

describe("routes", () => {
  it("creates stable kebab-case slugs", () => {
    expect(slug("SplDoublyLinkedList")).toBe("spl-doubly-linked-list");
    expect(slug("__construct")).toBe("construct");
  });

  it("normalizes and applies base paths", () => {
    expect(normalizeBasePath("typed-php")).toBe("/typed-php/");
    expect(withBase("/typed-php/", "/std/data-structures/")).toBe(
      "/typed-php/std/data-structures/",
    );
    expect(withBase("/", "/std/data-structures/")).toBe(
      "/std/data-structures/",
    );
  });

  it("writes route directories as index files", () => {
    expect(outputPath("/")).toBe("index.html");
    expect(outputPath("/std/data-structures/")).toBe(
      "std/data-structures/index.html",
    );
  });

  it("routes the standard-library overview at the section root", () => {
    expect(
      routeFor({
        kind: "module",
        id: "std.index",
        title: "Standard library",
        summary: "Overview.",
        module: "standard-library",
        order: 0,
      } as PageData),
    ).toBe("/std/");
  });

  it("routes internals guides in their own section", () => {
    expect(
      routeFor({
        kind: "guide",
        id: "guide.bytecodeInterpreter",
        title: "Bytecode interpreter",
        summary: "VM internals.",
        nav: { section: "internals", order: 100 },
      } as PageData),
    ).toBe("/internals/bytecode-interpreter/");
  });
});
