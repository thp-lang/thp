import type { GuideSection } from "./schema.js";

interface GuideSectionDetails {
  breadcrumb: string;
  navigation: string;
  notice: string;
  root: string;
  sidebar: string;
}

export const guideSections: Record<GuideSection, GuideSectionDetails> = {
  learn: {
    breadcrumb: "Learn",
    navigation: "Learn",
    notice: "Experimental language",
    root: "/learn/getting-started/",
    sidebar: "Learn THP",
  },
  language: {
    breadcrumb: "Language",
    navigation: "Implemented Language",
    notice: "Experimental language",
    root: "/language/overview/",
    sidebar: "Language reference",
  },
  internals: {
    breadcrumb: "Internals",
    navigation: "Internals",
    notice: "Experimental implementation",
    root: "/internals/overview/",
    sidebar: "Internals",
  },
};
