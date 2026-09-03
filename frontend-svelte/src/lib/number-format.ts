const compactFormatter = new Intl.NumberFormat("es-AR", {
  notation: "compact",
  compactDisplay: "short",
  maximumFractionDigits: 1,
});

const standardFormatter = new Intl.NumberFormat("es-AR");

export function formatNumber(value: number): string {
  return standardFormatter.format(value);
}

export function formatCompact(value: number): string {
  return compactFormatter.format(value);
}
