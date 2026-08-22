export type TypeExpression =
  | { kind: "name"; name: string; arguments: TypeExpression[] }
  | { kind: "nullable"; inner: TypeExpression }
  | { kind: "union"; members: TypeExpression[] };

class Parser {
  private index = 0;
  constructor(private readonly input: string) {}

  parse(): TypeExpression {
    const result = this.union();
    this.space();
    if (this.index !== this.input.length) this.fail("unexpected token");
    return result;
  }

  private union(): TypeExpression {
    const members = [this.nullable()];
    this.space();
    while (this.input[this.index] === "|") {
      this.index++;
      members.push(this.nullable());
      this.space();
    }
    return members.length === 1 ? members[0]! : { kind: "union", members };
  }

  private nullable(): TypeExpression {
    this.space();
    if (this.input[this.index] === "?") {
      this.index++;
      return { kind: "nullable", inner: this.nullable() };
    }
    const name = this.readName();
    const args: TypeExpression[] = [];
    this.space();
    if (this.input[this.index] === "<") {
      this.index++;
      let moreArguments = true;
      while (moreArguments) {
        args.push(this.union());
        this.space();
        if (this.input[this.index] === ",") this.index++;
        else moreArguments = false;
      }
      if (this.input[this.index] !== ">") this.fail("expected '>'");
      this.index++;
    }
    return { kind: "name", name, arguments: args };
  }

  private readName(): string {
    this.space();
    const start = this.index;
    while (/[A-Za-z0-9_\\]/.test(this.input[this.index] ?? "")) this.index++;
    if (start === this.index) this.fail("expected a type name");
    return this.input.slice(start, this.index);
  }

  private space(): void {
    while (/\s/.test(this.input[this.index] ?? "")) this.index++;
  }

  private fail(message: string): never {
    throw new Error(
      `${message} at column ${this.index + 1} in "${this.input}"`,
    );
  }
}

export function parseType(input: string): TypeExpression {
  return new Parser(input).parse();
}

export function printType(type: TypeExpression): string {
  if (type.kind === "nullable") return `?${printType(type.inner)}`;
  if (type.kind === "union") return type.members.map(printType).join("|");
  return type.arguments.length
    ? `${type.name}<${type.arguments.map(printType).join(", ")}>`
    : type.name;
}

export function substituteType(
  type: TypeExpression,
  substitutions: ReadonlyMap<string, TypeExpression>,
): TypeExpression {
  if (type.kind === "nullable")
    return {
      kind: "nullable",
      inner: substituteType(type.inner, substitutions),
    };
  if (type.kind === "union")
    return {
      kind: "union",
      members: type.members.map((item) => substituteType(item, substitutions)),
    };
  if (!type.arguments.length && substitutions.has(type.name))
    return substitutions.get(type.name)!;
  return {
    ...type,
    arguments: type.arguments.map((item) =>
      substituteType(item, substitutions),
    ),
  };
}

export function referencedNames(type: TypeExpression): string[] {
  if (type.kind === "nullable") return referencedNames(type.inner);
  if (type.kind === "union") return type.members.flatMap(referencedNames);
  return [type.name, ...type.arguments.flatMap(referencedNames)];
}
