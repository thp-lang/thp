import path from "node:path";
import { describe, expect, it } from "vitest";
import { createModel } from "../../src/build.js";
import { renderPage } from "../../src/view/pages.js";

const root = path.resolve(import.meta.dirname, "../..");

describe("0.2.0 contract availability", () => {
  it("maps every stream symbol to its executable availability", async () => {
    const model = await createModel({ root });
    const actual = new Map(
      model.pages
        .filter((page) => page.source.data.id.startsWith("std.streams"))
        .map((page) => [page.source.data.id, page.source.data.availability]),
    );

    expect(actual).toEqual(
      new Map([
        ["std.streams", "partial"],
        ["std.streams.Closeable", "implemented"],
        ["std.streams.Closeable::close", "implemented"],
        ["std.streams.Closeable::isClosed", "implemented"],
        ["std.streams.ClosedStreamException", "implemented"],
        ["std.streams.Files", "partial"],
        ["std.streams.Files::openRead", "implemented"],
        ["std.streams.Files::openReadWrite", "proposed"],
        ["std.streams.Files::openWrite", "proposed"],
        ["std.streams.InvalidStreamUriException", "implemented"],
        ["std.streams.IoException", "implemented"],
        ["std.streams.MemoryStream", "partial"],
        ["std.streams.MemoryStream::open", "implemented"],
        ["std.streams.OpenMode", "partial"],
        ["std.streams.OpenStreamException", "implemented"],
        ["std.streams.OpenStreamException::__construct", "partial"],
        ["std.streams.OpenStreamException::getSystemCode", "implemented"],
        ["std.streams.OpenStreamException::getTarget", "implemented"],
        ["std.streams.ReadWriteFileStream", "proposed"],
        ["std.streams.ReadableFileStream", "partial"],
        ["std.streams.ReadableStream", "partial"],
        ["std.streams.ReadableStream::eof", "implemented"],
        ["std.streams.ReadableStream::read", "implemented"],
        ["std.streams.ReadableStream::readAll", "implemented"],
        ["std.streams.SeekFrom", "proposed"],
        ["std.streams.SeekableStream", "partial"],
        ["std.streams.SeekableStream::seek", "partial"],
        ["std.streams.SeekableStream::tell", "implemented"],
        ["std.streams.Stream", "proposed"],
        ["std.streams.Stream::isReadable", "proposed"],
        ["std.streams.Stream::isSeekable", "proposed"],
        ["std.streams.Stream::isWritable", "proposed"],
        ["std.streams.Streams", "partial"],
        ["std.streams.Streams::open", "partial"],
        ["std.streams.TempStream", "partial"],
        ["std.streams.TempStream::open", "implemented"],
        ["std.streams.UnsupportedStreamOperationException", "implemented"],
        ["std.streams.WritableFileStream", "proposed"],
        ["std.streams.WritableStream", "partial"],
        ["std.streams.WritableStream::flush", "proposed"],
        ["std.streams.WritableStream::write", "proposed"],
        ["std.streams.WritableStream::writeAll", "implemented"],
        ["std.streams.WriteMode", "proposed"],
      ]),
    );
  });

  it("keeps iterator protocols, adapters, and transformations proposed", async () => {
    const model = await createModel({ root });
    const iteratorPages = model.pages.filter(
      (page) =>
        page.route.startsWith("/std/iterators/") ||
        page.source.data.id === "std.baseTypes.Traversable" ||
        page.source.data.id === "std.baseTypes.Iterator" ||
        page.source.data.id.startsWith("std.baseTypes.Iterator::") ||
        page.source.data.id === "std.baseTypes.IteratorAggregate" ||
        page.source.data.id.startsWith("std.baseTypes.IteratorAggregate::"),
    );

    expect(iteratorPages.length).toBeGreaterThan(100);
    expect(
      iteratorPages.every(
        (page) => page.source.data.availability === "proposed",
      ),
    ).toBe(true);

    const status = model.pages.find(
      (page) => page.source.data.id === "guide.implementationStatus",
    )!;
    for (const symbol of [
      "iterator_apply()",
      "iterator_to_vector()",
      "iterator_to_map()",
      "vector_map()",
      "vector_filter()",
      "vector_slice()",
      "vector_concat()",
      "map_transform()",
      "map_filter()",
      "map_merge()",
    ])
      expect(status.source.body).toContain(symbol);
  });

  it("renders a symbol matrix and keeps count separate from iterator_count", async () => {
    const model = await createModel({ root, basePath: "/typed-php/" });
    const status = model.pages.find(
      (page) => page.source.data.id === "guide.implementationStatus",
    )!;
    const html = renderPage(model, status);
    expect(html).toContain("Collection and iterator symbols");
    expect(html).toContain("Stream symbols");
    expect(html).toContain('<div class="table-scroll"><table>');

    expect(status.source.body).toContain(
      "`count(string\\|vector<T>\\|map<K, V>): int`",
    );
    expect(status.source.body).toContain(
      "Reads the value's byte or collection length; it does not consume, move, or create traversal state",
    );
    expect(status.source.body).toContain(
      "`iterator_count<K, V>(Iterator<K, V>): int`",
    );

    const iteratorCount = model.pages.find(
      (page) => page.source.data.id === "std.spl.iterator_count",
    )!;
    expect(iteratorCount.source.data.availability).toBe("proposed");
    if (iteratorCount.source.data.kind !== "function")
      throw new Error("iterator_count must remain a function page");
    expect(iteratorCount.source.data.parameters).toEqual([
      expect.objectContaining({ name: "iterator", type: "Iterator<K, V>" }),
    ]);
    expect(iteratorCount.source.body).toContain("does not call");
    expect(iteratorCount.source.body).toContain("`rewind()` before or after");
    expect(iteratorCount.source.body).toContain(
      "not an alias for, or overload of",
    );
  });
});
