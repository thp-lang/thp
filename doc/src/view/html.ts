const brand = Symbol("TrustedHtml");
export type TrustedHtml = string & { readonly [brand]: true };

export function escapeHtml(value: unknown): string {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function trusted(value: string): TrustedHtml {
  return value as TrustedHtml;
}

export function joinHtml(values: Array<string | TrustedHtml>): TrustedHtml {
  return trusted(values.join(""));
}
