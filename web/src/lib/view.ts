import { pageOf, type Page } from "./paging";
import { sortBy, type Order, type SortValue } from "./sort";

/** A column of a table: its name, and the value of each row for the sort and for downloads. */
export interface Column<T> {
  readonly id: string;
  readonly label: string;
  readonly value: (row: T) => SortValue;
  /** True for a column of numbers. */
  readonly numeric?: boolean;
}

/** How a person arranges a table: the sort, the page, and the columns to show. */
export interface TableView {
  readonly sort: string | null;
  readonly order: Order;
  readonly page: number;
  readonly size: number;
  /** The columns to show, in order. Null shows the default columns. */
  readonly columns: readonly string[] | null;
}

export const PAGE_SIZES: readonly number[] = [25, 50, 100, 500];

export const DEFAULT_VIEW: TableView = {
  sort: null,
  order: "ascending",
  page: 1,
  size: 50,
  columns: null,
};

/** Reads a view from the parameters of an address: `sort`, `order`, `page`, `size`, and `columns`. */
export function readView(params: URLSearchParams, known: readonly string[]): TableView {
  const sort = params.get("sort");
  const sorted = sort !== null && known.includes(sort);
  const page = Number(params.get("page"));
  const size = Number(params.get("size"));
  const columns = [...new Set((params.get("columns") ?? "").split(","))].filter((id) =>
    known.includes(id),
  );
  return {
    sort: sorted ? sort : null,
    order: sorted && params.get("order") === "desc" ? "descending" : "ascending",
    page: Number.isInteger(page) && page >= 1 ? page : 1,
    size: PAGE_SIZES.includes(size) ? size : DEFAULT_VIEW.size,
    columns: columns.length > 0 ? columns : null,
  };
}

export function writeView(params: URLSearchParams, view: TableView): void {
  if (view.sort !== null) {
    params.set("sort", view.sort);
    if (view.order === "descending") params.set("order", "desc");
  }
  if (view.page > 1) params.set("page", String(view.page));
  if (view.size !== DEFAULT_VIEW.size) params.set("size", String(view.size));
  if (view.columns !== null) params.set("columns", view.columns.join(","));
}

/** The view after a person selects the head of a column: the table sorts by it, or turns its order around. */
export function sortedBy(view: TableView, column: string): TableView {
  if (view.sort === column) {
    return { ...view, order: view.order === "ascending" ? "descending" : "ascending", page: 1 };
  }
  return { ...view, sort: column, order: "ascending", page: 1 };
}

/** All rows, in the order of the view. */
export function ordered<T>(
  rows: readonly T[],
  columns: readonly Column<T>[],
  view: TableView,
): readonly T[] {
  const column = columns.find((candidate) => candidate.id === view.sort);
  return column ? sortBy(rows, column.value, view.order) : rows;
}

/** The rows of the page of the view, in the order of the view. */
export function arrange<T>(
  rows: readonly T[],
  columns: readonly Column<T>[],
  view: TableView,
): Page<T> {
  return pageOf(ordered(rows, columns, view), view.page, view.size);
}

/** The columns that the table shows: those of the view, or the default columns. */
export function shownColumns<T>(
  columns: readonly Column<T>[],
  view: TableView,
  defaults: readonly string[],
): Column<T>[] {
  return (view.columns ?? defaults).flatMap((id) => columns.filter((column) => column.id === id));
}
