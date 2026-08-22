export class DiagnosticError extends Error {
  constructor(
    readonly file: string,
    readonly line: number,
    message: string,
  ) {
    super(`${file}:${line}: ${message}`);
    this.name = "DiagnosticError";
  }
}

export function lineFor(source: string, search: string): number {
  const index = source.indexOf(search);
  return index < 0 ? 1 : source.slice(0, index).split("\n").length;
}
