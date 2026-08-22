import { describe, expect, it } from "vitest";
import {
  parseType,
  printType,
  substituteType,
} from "../../src/type-expression.js";

describe("type expressions", () => {
  it("parses nested generics, nullable types, and unions", () => {
    expect(printType(parseType("Result<vector<?T>, Error>|null"))).toBe(
      "Result<vector<?T>, Error>|null",
    );
  });

  it("substitutes type parameters recursively", () => {
    const type = parseType("Iterator<vector<T>>");
    expect(
      printType(substituteType(type, new Map([["T", parseType("string")]]))),
    ).toBe("Iterator<vector<string>>");
  });

  it("reports the failing column", () => {
    expect(() => parseType("vector<T")).toThrow("expected '>' at column");
  });
});
