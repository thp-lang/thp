import type {
  CallablePage,
  MethodPage,
  PageData,
  SiteConfig,
  TypePage,
} from "./schema.js";
import type { TypeExpression } from "./type-expression.js";

export interface SourcePage {
  data: PageData;
  body: string;
  file: string;
}

export interface ResolvedMember {
  source: SourcePage & { data: MethodPage };
  declaringType: ResolvedType;
  viewedType: ResolvedType;
  inherited: boolean;
  parameters: Array<
    MethodPage["parameters"][number] & { parsedType: TypeExpression }
  >;
  parsedReturn: TypeExpression;
  route: string;
}

export interface ResolvedType {
  source: SourcePage & { data: TypePage };
  route: string;
  parent?: ResolvedType;
  interfaces: ResolvedType[];
  declared: ResolvedMember[];
  members: ResolvedMember[];
}

export interface ResolvedPage {
  source: SourcePage;
  route: string;
  renderedBody: string;
}

export interface SiteModel {
  config: SiteConfig;
  pages: ResolvedPage[];
  types: Map<string, ResolvedType>;
  callables: Map<string, SourcePage & { data: CallablePage }>;
  routes: Map<string, ResolvedPage | ResolvedType | ResolvedMember>;
  basePath: string;
  siteUrl: string;
}
