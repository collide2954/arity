import type { Index, Report, WorkPage } from "./types";

const cache = new Map<string, Promise<unknown>>();

/** The address of a file of the exported data. */
export function dataUrl(path: string): string {
  return `${import.meta.env.BASE_URL}data/${path}`;
}

async function read(path: string): Promise<unknown> {
  const response = await fetch(dataUrl(path));
  if (!response.ok) {
    throw new Error(`Cannot read ${path}: the server gave the status ${String(response.status)}.`);
  }
  return response.json();
}

function load<T>(path: string): Promise<T> {
  let request = cache.get(path);
  if (!request) {
    request = read(path);
    cache.set(path, request);
  }
  return request as Promise<T>;
}

export function loadIndex(): Promise<Index> {
  return load<Index>("index.json");
}

export function loadReports(): Promise<Report[]> {
  return load<Report[]>("reports.json");
}

export function loadWork(id: number): Promise<WorkPage> {
  return load<WorkPage>(`works/${String(id)}.json`);
}
