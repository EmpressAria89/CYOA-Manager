/** Catalog feeds mix bracket groups, comma lists and inconsistent casing. */
export function normalizeTags(values: string[]): string[] {
  const result = new Map<string, string>();
  const aliases: Record<string, string> = {oc:'Original Character',fempov:'PoV: Female',femalepov:'PoV: Female',malepov:'PoV: Male',neutralpov:'PoV: Neutral',eventbased:'Event Based',worldpicker:'World Picker',itempicker:'Item Picker',powerpicker:'Power Picker',sfw:'SFW',nsfw:'NSFW',mod:'MOD'};
  for (const value of values) {
    const expanded = value.replace(/\[([^\]]+)\]/g, '\n$1\n');
    for (const piece of expanded.split(/[\n,;|]+/)) {
      const clean = piece.trim().replace(/\s+/g, ' '); if (!clean) continue;
      const key=clean.toLowerCase().replace(/[\s:_-]/g,'');
      const label=aliases[key] || clean.replace(/\b\p{L}/gu, char=>char.toUpperCase());
      if(!result.has(label.toLowerCase()))result.set(label.toLowerCase(),label);
    }
  }
  return [...result.values()];
}
