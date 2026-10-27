import path from "node:path";
import { beforeAll, describe, expect, it } from "vitest";
import { createModel } from "../../src/build.js";
import type { SiteModel } from "../../src/model.js";
import { printType } from "../../src/type-expression.js";
import { renderPage } from "../../src/view/pages.js";

const root = path.resolve(import.meta.dirname, "../..");
const heap = [
  "__construct",
  "insert",
  "top",
  "extract",
  "count",
  "isEmpty",
  "getIterator",
];
const methods: Record<string, string[]> = {
  FixedSequence: [
    "__construct",
    "getSize",
    "count",
    "setSize",
    "offsetExists",
    "offsetGet",
    "offsetSet",
    "offsetUnset",
    "getIterator",
  ],
  LinkedList: [
    "__construct",
    "push",
    "unshift",
    "pop",
    "shift",
    "top",
    "bottom",
    "count",
    "isEmpty",
    "toVector",
    "getIterator",
  ],
  Queue: ["enqueue", "dequeue"],
  MaxHeap: heap,
  MinHeap: heap,
  PriorityQueue: heap,
  TypedMap: [
    "__construct",
    "set",
    "get",
    "contains",
    "remove",
    "count",
    "toMap",
    "getIterator",
  ],
  ObjectStorage: [
    "__construct",
    "attach",
    "get",
    "contains",
    "detach",
    "count",
    "getIterator",
  ],
};

describe("native collection and proposed iterator references", () => {
  let model: SiteModel;
  beforeAll(async () => {
    model = await createModel({ root, basePath: "/typed-php/" });
  });
  const page = (id: string) =>
    model.pages.find((item) => item.source.data.id === id)!;
  const type = (name: string) => model.types.get(`std.dataStructures.${name}`)!;
  const member = (owner: string, name: string) =>
    type(owner).declared.find((item) => item.source.data.name === name)!;

  it("documents all 58 native methods with unique routes, canonical owners and implemented availability", () => {
    const routes: string[] = [];
    for (const [name, names] of Object.entries(methods)) {
      const owner = `std.dataStructures.${name}`;
      expect(type(name).source.data.availability).toBe("implemented");
      expect(type(name).declared.map((item) => item.source.data.name)).toEqual(
        names,
      );
      const html = renderPage(model, page(owner));
      for (const method of type(name).declared) {
        const id = `${owner}::${method.source.data.name}`;
        expect(method.source.data.id).toBe(id);
        expect(method.source.data.owner).toBe(owner);
        expect(method.declaringType).toBe(type(name));
        expect(method.inherited).toBe(false);
        expect(method.source.data.status).toBe("experimental");
        expect(method.source.data.availability).toBe("implemented");
        expect(page(id).route).toBe(method.route);
        expect(html).toContain(`href="/typed-php${method.route}"`);
        routes.push(method.route);
      }
    }
    expect(routes).toHaveLength(58);
    expect(new Set(routes).size).toBe(58);
    expect(new Set(model.pages.map((item) => item.route)).size).toBe(
      model.pages.length,
    );
  });

  it("preserves native signatures, nullable results, defaults and errors", () => {
    for (const name of Object.keys(methods).filter(
      (name) => name !== "Queue",
    )) {
      const ctor = member(name, "__construct");
      expect(printType(ctor.parsedReturn)).toBe("void");
      expect(
        ctor.parameters.map((param) => [
          param.name,
          printType(param.parsedType),
          param.default,
        ]),
      ).toEqual(name === "FixedSequence" ? [["size", "int", "0"]] : []);
    }
    for (const [owner, name, params, result] of [
      ["FixedSequence", "offsetSet", ["?int", "?T"], "void"],
      ["FixedSequence", "offsetGet", ["int"], "?T"],
      ["FixedSequence", "getIterator", [], "Traversable<int, ?T>"],
      ["LinkedList", "push", ["T"], "void"],
      ["LinkedList", "toVector", [], "vector<T>"],
      ["Queue", "enqueue", ["T"], "void"],
      ["Queue", "dequeue", [], "T"],
      ["MaxHeap", "insert", ["T"], "void"],
      ["MinHeap", "insert", ["T"], "void"],
      ["PriorityQueue", "insert", ["T", "int"], "void"],
      ["TypedMap", "set", ["K", "V"], "void"],
      ["TypedMap", "get", ["K"], "?V"],
      ["TypedMap", "toMap", [], "map<K, V>"],
      ["TypedMap", "getIterator", [], "Traversable<K, V>"],
      ["ObjectStorage", "attach", ["mixed", "T"], "void"],
      ["ObjectStorage", "get", ["mixed"], "?T"],
      ["ObjectStorage", "getIterator", [], "Traversable<int, mixed>"],
    ] as const) {
      const method = member(owner, name);
      expect(
        method.parameters.map((param) => printType(param.parsedType)),
      ).toEqual(params);
      expect(printType(method.parsedReturn)).toBe(result);
    }
    for (const [owner, names, error] of [
      ["FixedSequence", ["__construct", "setSize", "offsetSet"], "ValueError"],
      [
        "FixedSequence",
        ["offsetGet", "offsetSet", "offsetUnset"],
        "OutOfBoundsException",
      ],
      ["LinkedList", ["pop", "shift", "top", "bottom"], "UnderflowException"],
      ["Queue", ["dequeue"], "UnderflowException"],
      ["MaxHeap", ["insert"], "ValueError"],
      ["MinHeap", ["insert"], "ValueError"],
      ["PriorityQueue", ["insert"], "OverflowException"],
      ["ObjectStorage", ["attach", "get", "contains", "detach"], "TypeError"],
    ] as const)
      for (const name of names)
        expect(
          member(owner, name).source.data.errors.map((item) => item.type),
        ).toContain(error);
    for (const owner of ["MaxHeap", "MinHeap", "PriorityQueue"])
      for (const name of ["top", "extract"])
        expect(member(owner, name).source.data.errors[0]?.type).toBe(
          "UnderflowException",
        );
  });

  it("renders native defaults and specialized proposed synopses under the base path", () => {
    const fixed = renderPage(model, page("std.dataStructures.FixedSequence"));
    expect(fixed).toContain(
      '<a href="/typed-php/std/data-structures/fixed-sequence/construct/">__construct</a>(<span class="type">int</span> <span class="var">$size</span> = 0);',
    );
    expect(fixed).toContain(
      '<a href="/typed-php/std/data-structures/fixed-sequence/offset-set/">offsetSet</a>(<span class="type">?int</span> <span class="var">$offset</span>, <span class="type">?T</span> <span class="var">$value</span>): <span class="type">void</span>;',
    );
    const recursive = renderPage(
      model,
      page("std.spl.RecursiveCollectionIterator"),
    );
    expect(recursive).toContain(
      '<a href="/typed-php/std/base-types/iterator/value/">value</a>(): <span class="type">RecursiveEntry&lt;mixed, mixed&gt;</span>;',
    );
    const adapter = renderPage(model, page("std.spl.NoRewindIterator"));
    expect(adapter).toContain(
      '<a href="/typed-php/std/iterators/no-rewind-iterator/rewind/">rewind</a>(): <span class="type">void</span>;',
    );
    expect(adapter).toContain(
      '<a href="/typed-php/std/iterators/iterator-iterator/get-inner-iterator/">getInnerIterator</a>(): <span class="type">Iterator&lt;K, V&gt;</span>;',
    );
  });

  it("keeps Stack and Queue inherited references on LinkedList", () => {
    for (const name of ["Stack", "Queue"]) {
      const inherited = type(name).members.filter((item) => item.inherited);
      expect(inherited).toHaveLength(11);
      for (const method of inherited) {
        expect(method.declaringType.source.data.id).toBe(
          "std.dataStructures.LinkedList",
        );
        expect(method.route).toMatch(/^\/std\/data-structures\/linked-list\//);
      }
      expect(
        printType(
          inherited.find((item) => item.source.data.name === "pop")!
            .parsedReturn,
        ),
      ).toBe("T");
      expect(
        printType(
          inherited.find((item) => item.source.data.name === "getIterator")!
            .parsedReturn,
        ),
      ).toBe("Traversable<int, T>");
      expect(renderPage(model, page(`std.dataStructures.${name}`))).toContain(
        'href="/typed-php/std/data-structures/linked-list/pop/" data-inherited="true"',
      );
      expect(
        model.pages.some((item) => item.route === type(name).route + "pop/"),
      ).toBe(false);
    }
    expect(
      page("std.dataStructures.LinkedList::getIterator").source.body,
    ).toContain("top-to-bottom LIFO");
  });

  it("resolves proposed adapters, generic substitution and inherited cursor methods", () => {
    const noRewind = model.types.get("std.spl.NoRewindIterator")!;
    expect(noRewind.source.data.parent).toEqual({
      id: "std.spl.IteratorIterator",
      arguments: ["K", "V"],
    });
    expect(noRewind.declared.map((item) => item.source.data.name)).toEqual([
      "__construct",
      "rewind",
    ]);
    expect(noRewind.members).toHaveLength(7);
    expect(noRewind.declared[0]!.parameters[0]!.type).toBe("Iterator<K, V>");
    const inner = noRewind.members.find(
      (item) => item.source.data.name === "getInnerIterator",
    )!;
    expect(inner.route).toBe(
      "/std/iterators/iterator-iterator/get-inner-iterator/",
    );
    expect(printType(inner.parsedReturn)).toBe("Iterator<K, V>");
    expect(
      noRewind.members.find((item) => item.source.data.name === "rewind")!
        .inherited,
    ).toBe(false);
    const recursive = model.types.get("std.spl.RecursiveCollectionIterator")!;
    expect(recursive.source.data.typeParameters).toEqual([]);
    expect(recursive.source.data.interfaces).toEqual([
      { id: "std.spl.RecursiveIterator", arguments: ["mixed", "mixed"] },
    ]);
    expect(recursive.declared.map((item) => item.source.data.name)).toEqual([
      "__construct",
    ]);
    expect(recursive.declared[0]!.parameters[0]!.type).toBe("mixed");
    expect(recursive.declared[0]!.source.data.errors[0]?.type).toBe(
      "TypeError",
    );
    expect(recursive.members).toHaveLength(6);
    for (const [name, result] of Object.entries({
      rewind: "void",
      valid: "bool",
      key: "mixed",
      value: "RecursiveEntry<mixed, mixed>",
      advance: "void",
    })) {
      const method = recursive.members.find(
        (item) => item.source.data.name === name,
      )!;
      expect(method.inherited).toBe(true);
      expect(method.declaringType.source.data.id).toBe(
        "std.baseTypes.Iterator",
      );
      expect(method.route).toBe(`/std/base-types/iterator/${name}/`);
      expect(printType(method.parsedReturn)).toBe(result);
    }
    for (const adapter of [noRewind, recursive]) {
      for (const source of [
        adapter.source,
        ...adapter.declared.map((method) => method.source),
      ]) {
        expect(source.data.status).toBe("experimental");
        expect(source.data.availability).toBe("proposed");
        expect(renderPage(model, page(source.data.id))).toContain(
          "unavailable at runtime",
        );
      }
      const html = renderPage(model, page(adapter.source.data.id));
      for (const method of adapter.members)
        expect(html).toContain(`href="/typed-php${method.route}"`);
      expect(renderPage(model, page("std.iterators"))).toContain(
        `href="/typed-php${adapter.route}"`,
      );
      expect(page("guide.implementationStatus").source.body).toContain(
        `thp:${adapter.source.data.id}`,
      );
    }
    expect(renderPage(model, page(recursive.source.data.id))).toContain(
      "RecursiveEntry&lt;mixed, mixed&gt;",
    );
    expect(renderPage(model, page("std.spl.RecursiveArrayIterator"))).toContain(
      'href="/typed-php/std/iterators/recursive-collection-iterator/"',
    );
    expect(page("std.baseTypes.Iterator::rewind").source.body).toContain(
      "thp:std.spl.NoRewindIterator",
    );
  });

  it("preserves all five SPL interfaces and their canonical methods", () => {
    for (const [name, names] of [
      ["OuterIterator", ["getInnerIterator"]],
      ["SeekableIterator", ["seek"]],
      ["RecursiveIterator", []],
      ["SplObserver", ["update"]],
      ["SplSubject", ["attach", "detach", "notify"]],
    ] as const) {
      const contract = model.types.get(`std.spl.${name}`)!;
      expect(contract.source.data.kind).toBe("interface");
      expect(contract.declared.map((item) => item.source.data.name)).toEqual(
        names,
      );
      for (const method of contract.declared)
        expect(page(method.source.data.id).route).toBe(method.route);
    }
  });
});
