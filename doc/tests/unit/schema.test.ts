import { describe, expect, it } from "vitest";
import { pageSchema } from "../../src/schema.js";

describe("frontmatter schemas", () => {
  it("accepts stable callable IDs", () => {
    const result = pageSchema.safeParse({
      kind: "method",
      id: "std.spl.Queue::take",
      title: "Queue::take",
      summary: "Takes a value.",
      availability: "proposed",
      owner: "std.spl.Queue",
      name: "take",
      order: 1,
      parameters: [],
      returns: { type: "T", description: "A value." },
      errors: [],
      related: [],
      version: "0.1",
    });
    expect(result.success).toBe(true);
  });

  it("rejects unknown fields", () => {
    const result = pageSchema.safeParse({
      kind: "home",
      id: "docs.home",
      title: "Docs",
      summary: "Documentation.",
      availability: "implemented",
      template: "unsafe",
    });
    expect(result.success).toBe(false);
  });

  it("requires implementation availability separately from stability", () => {
    const result = pageSchema.safeParse({
      kind: "home",
      id: "docs.home",
      title: "Docs",
      summary: "Documentation.",
      status: "experimental",
    });
    expect(result.success).toBe(false);
  });

  it("accepts the internals guide section", () => {
    const result = pageSchema.safeParse({
      kind: "guide",
      id: "guide.internalsOverview",
      title: "Overview",
      summary: "Compiler and runtime internals.",
      availability: "implemented",
      nav: { section: "internals", order: 10 },
    });
    expect(result.success).toBe(true);
  });
});
