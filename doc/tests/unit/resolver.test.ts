import { describe, expect, it } from "vitest";
import type { SourcePage } from "../../src/model.js";
import { resolveSite } from "../../src/resolver.js";
import { pageSchema, siteSchema } from "../../src/schema.js";
import { printType } from "../../src/type-expression.js";

const config = siteSchema.parse({
  title: "Test",
  description: "Test documentation.",
  github: "https://example.com",
  navigation: [],
  modules: { test: { name: "Test", summary: "Test module." } },
});

function source(data: Record<string, unknown>, body = ""): SourcePage {
  const parsed = pageSchema.parse({ availability: "proposed", ...data });
  return { data: parsed, body, file: `${parsed.id}.md` };
}

function type(
  id: string,
  name: string,
  extra: Record<string, unknown> = {},
): SourcePage {
  return source({
    kind: "class",
    id,
    title: name,
    summary: `${name} summary.`,
    name,
    module: "test",
    typeParameters: [],
    interfaces: [],
    constants: [],
    properties: [],
    version: "0.1",
    ...extra,
  });
}

function method(owner: string, name: string, returnType = "void"): SourcePage {
  return source({
    kind: "method",
    id: `${owner}::${name}`,
    title: `${owner}::${name}`,
    summary: `${name} summary.`,
    owner,
    name,
    order: 1,
    parameters: [],
    returns: { type: returnType, description: "A value." },
    errors: [],
    related: [],
    version: "0.1",
  });
}

describe("symbol graph resolver", () => {
  it("substitutes generic parameters across inheritance edges", async () => {
    const parent = type("std.test.Parent", "Parent", {
      typeParameters: [{ name: "T", description: "Value type." }],
    });
    const child = type("std.test.Child", "Child", {
      parent: { id: "std.test.Parent", arguments: ["string"] },
    });
    const model = await resolveSite(
      [parent, child, method("std.test.Parent", "value", "T")],
      config,
      "/",
      "https://example.com",
    );
    const inherited = model.types.get("std.test.Child")!.members[0]!;
    expect(printType(inherited.parsedReturn)).toBe("string");
    expect(inherited.route).toBe("/std/test/parent/value/");
  });

  it("uses a child override instead of the inherited member", async () => {
    const parent = type("std.test.Parent", "Parent");
    const child = type("std.test.Child", "Child", {
      parent: { id: "std.test.Parent", arguments: [] },
    });
    const model = await resolveSite(
      [
        parent,
        child,
        method("std.test.Parent", "run"),
        method("std.test.Child", "run"),
      ],
      config,
      "/",
      "https://example.com",
    );
    const members = model.types.get("std.test.Child")!.members;
    expect(members).toHaveLength(1);
    expect(members[0]?.declaringType.source.data.id).toBe("std.test.Child");
    expect(members[0]?.inherited).toBe(false);
  });

  it("rejects inheritance cycles", async () => {
    const first = type("std.test.First", "First", {
      parent: { id: "std.test.Second", arguments: [] },
    });
    const second = type("std.test.Second", "Second", {
      parent: { id: "std.test.First", arguments: [] },
    });
    await expect(
      resolveSite([first, second], config, "/", "https://example.com"),
    ).rejects.toThrow("inheritance cycle");
  });

  it("rejects invalid type argument counts", async () => {
    const parent = type("std.test.Parent", "Parent", {
      typeParameters: [{ name: "T", description: "Value type." }],
    });
    const child = type("std.test.Child", "Child", {
      parent: { id: "std.test.Parent", arguments: [] },
    });
    await expect(
      resolveSite([parent, child], config, "/", "https://example.com"),
    ).rejects.toThrow("expects 1 argument(s), got 0");
  });

  it("rejects equally near ambiguous interface members", async () => {
    const first = type("std.test.First", "First");
    first.data = pageSchema.parse({ ...first.data, kind: "interface" });
    const second = type("std.test.Second", "Second");
    second.data = pageSchema.parse({ ...second.data, kind: "interface" });
    const child = type("std.test.Child", "Child", {
      interfaces: [
        { id: "std.test.First", arguments: [] },
        { id: "std.test.Second", arguments: [] },
      ],
    });
    await expect(
      resolveSite(
        [
          first,
          second,
          child,
          method("std.test.First", "run"),
          method("std.test.Second", "run"),
        ],
        config,
        "/",
        "https://example.com",
      ),
    ).rejects.toThrow('ambiguous inherited member "run"');
  });

  it("rejects unresolved thp symbol links", async () => {
    const page = source(
      {
        kind: "home",
        id: "docs.home",
        title: "Home",
        summary: "Home page.",
      },
      "[missing](thp:std.test.Missing)",
    );
    await expect(
      resolveSite([page], config, "/", "https://example.com"),
    ).rejects.toThrow('unresolved symbol reference "std.test.Missing"');
  });
});
