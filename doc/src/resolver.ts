import { DiagnosticError, lineFor } from "./diagnostic.js";
import type {
  ResolvedMember,
  ResolvedPage,
  ResolvedType,
  SiteModel,
  SourcePage,
} from "./model.js";
import { renderMarkdown } from "./markdown.js";
import { routeFor } from "./route.js";
import type { MethodPage, SiteConfig, TypePage } from "./schema.js";
import {
  parseType,
  printType,
  referencedNames,
  substituteType,
  type TypeExpression,
} from "./type-expression.js";

const primitives = new Set([
  "array",
  "array-key",
  "bool",
  "callable",
  "class-string",
  "false",
  "float",
  "int",
  "Iterator",
  "iterable",
  "map",
  "mixed",
  "never",
  "null",
  "object",
  "resource",
  "self",
  "static",
  "string",
  "true",
  "void",
  "vector",
]);

function fail(source: SourcePage, message: string, search = ""): never {
  throw new DiagnosticError(source.file, lineFor(source.body, search), message);
}

function parseMember(
  source: SourcePage & { data: MethodPage },
): Omit<
  ResolvedMember,
  "declaringType" | "viewedType" | "inherited" | "route"
> {
  try {
    return {
      source,
      parameters: source.data.parameters.map((parameter) => ({
        ...parameter,
        parsedType: parseType(parameter.type),
      })),
      parsedReturn: parseType(source.data.returns.type),
    };
  } catch (error) {
    fail(source, error instanceof Error ? error.message : String(error));
  }
}

function cloneInherited(
  member: ResolvedMember,
  viewedType: ResolvedType,
  substitutions: ReadonlyMap<string, TypeExpression>,
): ResolvedMember {
  return {
    ...member,
    viewedType,
    inherited: true,
    parameters: member.parameters.map((parameter) => ({
      ...parameter,
      parsedType: substituteType(parameter.parsedType, substitutions),
    })),
    parsedReturn: substituteType(member.parsedReturn, substitutions),
  };
}

export async function resolveSite(
  sources: SourcePage[],
  config: SiteConfig,
  basePath: string,
  siteUrl: string,
): Promise<SiteModel> {
  const ids = new Map<string, SourcePage>();
  for (const source of sources) {
    const duplicate = ids.get(source.data.id);
    if (duplicate)
      fail(
        source,
        `duplicate ID "${source.data.id}" (first declared in ${duplicate.file})`,
      );
    ids.set(source.data.id, source);
  }

  const typeSources = new Map(
    sources
      .filter(
        (source): source is SourcePage & { data: TypePage } =>
          source.data.kind === "class" ||
          source.data.kind === "interface" ||
          source.data.kind === "trait" ||
          source.data.kind === "enum",
      )
      .map((source) => [source.data.id, source]),
  );
  const types = new Map<string, ResolvedType>();
  for (const source of typeSources.values()) {
    types.set(source.data.id, {
      source,
      route: routeFor(source.data),
      interfaces: [],
      declared: [],
      members: [],
    });
  }

  const methodSources = sources.filter(
    (source): source is SourcePage & { data: MethodPage } =>
      source.data.kind === "method",
  );
  const callableSources = sources.filter(
    (
      source,
    ): source is SourcePage & {
      data: Extract<SourcePage["data"], { kind: "method" | "function" }>;
    } => source.data.kind === "method" || source.data.kind === "function",
  );
  const callables = new Map(
    callableSources.map((source) => [source.data.id, source]),
  );

  for (const source of methodSources) {
    const owner = types.get(source.data.owner);
    if (!owner) fail(source, `unknown method owner "${source.data.owner}"`);
    const partial = parseMember(source);
    owner.declared.push({
      ...partial,
      declaringType: owner,
      viewedType: owner,
      inherited: false,
      route: routeFor(source.data, owner.source.data),
    });
  }
  for (const type of types.values())
    type.declared.sort((a, b) => a.source.data.order - b.source.data.order);

  const state = new Map<string, "visiting" | "done">();
  const resolveType = (
    type: ResolvedType,
    chain: string[] = [],
  ): ResolvedType => {
    const id = type.source.data.id;
    if (state.get(id) === "done") return type;
    if (state.get(id) === "visiting")
      fail(type.source, `inheritance cycle: ${[...chain, id].join(" → ")}`);
    state.set(id, "visiting");

    const relations = [
      ...(type.source.data.parent ? [type.source.data.parent] : []),
      ...type.source.data.interfaces,
    ];
    const resolvedRelations = relations.map((relation) => {
      const target = types.get(relation.id);
      if (!target) fail(type.source, `unknown inherited type "${relation.id}"`);
      if (
        (relation.arguments?.length ?? 0) !==
        target.source.data.typeParameters.length
      )
        fail(
          type.source,
          `type "${relation.id}" expects ${target.source.data.typeParameters.length} argument(s), got ${relation.arguments?.length ?? 0}`,
        );
      return resolveType(target, [...chain, id]);
    });
    const parent = type.source.data.parent ? resolvedRelations[0] : undefined;
    if (parent) type.parent = parent;
    type.interfaces = resolvedRelations.slice(parent ? 1 : 0);

    const declaredByName = new Map(
      type.declared.map((member) => [member.source.data.name, member]),
    );
    const inherited = new Map<string, ResolvedMember>();
    const inheritedDistance = new Map<string, number>();
    const candidates = relations.map((relation, index) => {
      const resolved = resolvedRelations[index]!;
      const substitution = new Map<string, TypeExpression>();
      resolved.source.data.typeParameters.forEach(
        (parameter, argumentIndex) => {
          const argument = relation.arguments?.[argumentIndex];
          if (argument) substitution.set(parameter.name, parseType(argument));
        },
      );
      return {
        resolved,
        substitution,
        isParent: index === 0 && Boolean(parent),
      };
    });
    for (const candidate of candidates) {
      for (const member of candidate.resolved.members) {
        if (declaredByName.has(member.source.data.name)) continue;
        const existing = inherited.get(member.source.data.name);
        const distance = candidate.isParent ? 0 : 1;
        if (
          existing &&
          inheritedDistance.get(member.source.data.name) === distance &&
          (existing.declaringType.source.data.id !==
            member.declaringType.source.data.id ||
            printType(existing.parsedReturn) !==
              printType(
                substituteType(member.parsedReturn, candidate.substitution),
              ))
        ) {
          fail(
            type.source,
            `ambiguous inherited member "${member.source.data.name}" from "${existing.declaringType.source.data.id}" and "${member.declaringType.source.data.id}"`,
          );
        }
        if (
          !existing ||
          distance <
            (inheritedDistance.get(member.source.data.name) ?? Infinity)
        ) {
          inherited.set(
            member.source.data.name,
            cloneInherited(member, type, candidate.substitution),
          );
          inheritedDistance.set(member.source.data.name, distance);
        }
      }
    }
    type.members = [...type.declared, ...inherited.values()];
    state.set(id, "done");
    return type;
  };
  for (const type of types.values()) resolveType(type);

  const knownTypeNames = new Set(
    [...types.values()].map((type) => type.source.data.name),
  );
  for (const type of types.values()) {
    const allowed = new Set(
      type.source.data.typeParameters.map((parameter) => parameter.name),
    );
    for (const member of type.members) {
      const memberAllowed = new Set([
        ...allowed,
        ...member.source.data.typeParameters.map((parameter) => parameter.name),
      ]);
      for (const expression of [
        ...member.parameters.map((parameter) => parameter.parsedType),
        member.parsedReturn,
      ]) {
        for (const name of referencedNames(expression)) {
          if (
            !primitives.has(name) &&
            !memberAllowed.has(name) &&
            !knownTypeNames.has(name)
          )
            fail(
              member.source,
              `unknown type "${name}" in ${printType(expression)}`,
            );
        }
      }
    }
  }
  for (const source of sources) {
    if (source.data.kind !== "function") continue;
    const allowed = new Set(
      source.data.typeParameters.map((parameter) => parameter.name),
    );
    for (const value of [
      ...source.data.parameters.map((parameter) => parameter.type),
      source.data.returns.type,
    ]) {
      let expression: TypeExpression;
      try {
        expression = parseType(value);
      } catch (error) {
        fail(source, error instanceof Error ? error.message : String(error));
      }
      for (const name of referencedNames(expression)) {
        if (
          !primitives.has(name) &&
          !allowed.has(name) &&
          !knownTypeNames.has(name)
        )
          fail(source, `unknown type "${name}" in ${printType(expression)}`);
      }
    }
  }

  const routeMap = new Map<
    string,
    ResolvedPage | ResolvedType | ResolvedMember
  >();
  const pages: ResolvedPage[] = [];
  const symbolRoutes = new Map<string, string>();
  for (const type of types.values())
    symbolRoutes.set(type.source.data.id, type.route);
  for (const type of types.values())
    for (const member of type.declared)
      symbolRoutes.set(member.source.data.id, member.route);
  for (const source of sources) {
    if (source.data.kind === "function")
      symbolRoutes.set(source.data.id, routeFor(source.data));
    if (
      source.data.kind === "home" ||
      source.data.kind === "guide" ||
      source.data.kind === "module"
    )
      symbolRoutes.set(source.data.id, routeFor(source.data));
  }

  for (const source of sources) {
    let owner;
    if (source.data.kind === "method")
      owner = types.get(source.data.owner)?.source.data;
    const route = routeFor(source.data, owner);
    if (routeMap.has(route)) fail(source, `duplicate route "${route}"`);
    let body = source.body;
    body = body.replace(/\]\(thp:([^)]+)\)/g, (match, id: string) => {
      const target = symbolRoutes.get(id);
      if (!target) fail(source, `unresolved symbol reference "${id}"`, match);
      return `](${target})`;
    });
    for (const related of "related" in source.data ? source.data.related : []) {
      if (!ids.has(related))
        fail(source, `unresolved related symbol "${related}"`);
    }
    const page: ResolvedPage = {
      source,
      route,
      renderedBody: String(await renderMarkdown(body)),
    };
    pages.push(page);
    routeMap.set(route, page);
  }
  for (const type of types.values()) {
    routeMap.set(type.route, type);
    for (const member of type.declared) routeMap.set(member.route, member);
  }

  return {
    config,
    pages,
    types,
    callables,
    routes: routeMap,
    basePath,
    siteUrl,
  };
}
