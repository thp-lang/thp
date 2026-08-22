import { z } from "zod";

const id = z
  .string()
  .regex(
    /^[a-z][a-z0-9]*(?:\.[A-Za-z0-9_]+)*(?:::[A-Za-z0-9_]+)?$/,
    "invalid stable ID",
  );
const nonempty = z.string().min(1);
const status = z.enum(["experimental", "stable", "deprecated"]);
const availability = z.enum(["implemented", "partial", "proposed"]);
const version = z.string().regex(/^\d+\.\d+(?:\.\d+)?$/);
const typeParameter = z
  .object({
    name: z.string().regex(/^[A-Z][A-Za-z0-9_]*$/),
    description: nonempty,
  })
  .strict();
export const guideSectionSchema = z.enum(["learn", "language", "internals"]);
export type GuideSection = z.infer<typeof guideSectionSchema>;

const nav = z
  .object({
    section: guideSectionSchema,
    order: z.number().int().nonnegative(),
  })
  .strict();
const value = z
  .object({
    type: nonempty,
    description: nonempty,
  })
  .strict();
const parameter = z
  .object({
    name: z.string().regex(/^[a-z_][A-Za-z0-9_]*$/),
    type: nonempty,
    description: nonempty,
    default: z.string().optional(),
    variadic: z.boolean().optional(),
  })
  .strict();
const error = z
  .object({
    type: nonempty.optional(),
    description: nonempty,
  })
  .strict();
const relation = z
  .object({
    id,
    arguments: z.array(nonempty).optional(),
  })
  .strict();
const memberFact = z
  .object({
    name: nonempty,
    type: nonempty,
    description: nonempty,
  })
  .strict();
const base = {
  id,
  title: nonempty,
  summary: nonempty,
  availability,
  status: status.optional(),
  notice: nonempty.optional(),
};

export const homeSchema = z
  .object({
    kind: z.literal("home"),
    ...base,
  })
  .strict();
export const guideSchema = z
  .object({
    kind: z.literal("guide"),
    ...base,
    nav,
  })
  .strict();
export const moduleSchema = z
  .object({
    kind: z.literal("module"),
    ...base,
    module: z.string().regex(/^[a-z][a-z0-9-]*$/),
    order: z.number().int().nonnegative(),
  })
  .strict();
const typeBase = {
  ...base,
  name: nonempty,
  module: z.string().regex(/^[a-z][a-z0-9-]*$/),
  typeParameters: z.array(typeParameter).default([]),
  parent: relation.optional(),
  interfaces: z.array(relation).default([]),
  constants: z.array(memberFact).default([]),
  properties: z.array(memberFact).default([]),
  version,
};
export const classSchema = z
  .object({ kind: z.literal("class"), ...typeBase })
  .strict();
export const interfaceSchema = z
  .object({
    kind: z.literal("interface"),
    ...typeBase,
    parent: z.never().optional(),
  })
  .strict();
export const traitSchema = z
  .object({
    kind: z.literal("trait"),
    ...typeBase,
    parent: z.never().optional(),
  })
  .strict();
export const enumSchema = z
  .object({
    kind: z.literal("enum"),
    ...typeBase,
    cases: z.array(nonempty).default([]),
  })
  .strict();
const callableBase = {
  ...base,
  name: nonempty,
  order: z.number().int().nonnegative(),
  typeParameters: z.array(typeParameter).default([]),
  parameters: z.array(parameter).default([]),
  returns: value,
  errors: z.array(error).default([]),
  related: z.array(id).default([]),
  version,
};
export const methodSchema = z
  .object({
    kind: z.literal("method"),
    ...callableBase,
    owner: id,
    visibility: z.enum(["public", "protected", "private"]).default("public"),
    modifiers: z.array(z.enum(["static", "final", "abstract"])).default([]),
  })
  .strict();
export const functionSchema = z
  .object({
    kind: z.literal("function"),
    ...callableBase,
    module: z.string().regex(/^[a-z][a-z0-9-]*$/),
  })
  .strict();

export const pageSchema = z.discriminatedUnion("kind", [
  homeSchema,
  guideSchema,
  moduleSchema,
  classSchema,
  interfaceSchema,
  traitSchema,
  enumSchema,
  methodSchema,
  functionSchema,
]);

export const siteSchema = z
  .object({
    title: nonempty,
    description: nonempty,
    github: z.url(),
    navigation: z.array(z.object({ label: nonempty, href: nonempty }).strict()),
    modules: z.record(
      z.string(),
      z
        .object({
          name: nonempty,
          summary: nonempty,
          order: z.number().int().nonnegative().default(0),
          typeGroup: nonempty.default("Types"),
        })
        .strict(),
    ),
  })
  .strict();

export type PageData = z.infer<typeof pageSchema>;
export type TypePage = Extract<
  PageData,
  { kind: "class" | "interface" | "trait" | "enum" }
>;
export type CallablePage = Extract<PageData, { kind: "method" | "function" }>;
export type MethodPage = Extract<PageData, { kind: "method" }>;
export type SiteConfig = z.infer<typeof siteSchema>;
