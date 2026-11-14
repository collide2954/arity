/** Writes rows as CSV text, with quotes where a field needs them (RFC 4180). */
export function toCsv(header: readonly string[], rows: readonly (readonly unknown[])[]): string {
  const field = (value: unknown): string => {
    let text: string;
    if (value === null || value === undefined) text = "";
    else if (typeof value === "string") text = value;
    else if (typeof value === "number" || typeof value === "boolean") text = String(value);
    else text = JSON.stringify(value);
    return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
  };
  return [header, ...rows].map((row) => row.map(field).join(",")).join("\r\n") + "\r\n";
}

/** Gives the browser a CSV file to save. */
export function download(name: string, text: string): void {
  const url = URL.createObjectURL(new Blob([text], { type: "text/csv;charset=utf-8" }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = name;
  anchor.click();
  URL.revokeObjectURL(url);
}
