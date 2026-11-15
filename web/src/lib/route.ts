/**
 * The pages of the website, as hash routes, for example `#/works/12?report=40`.
 * The pages with filters keep them in `params`, the query of the address.
 */

export type Route =
  | { page: "home" }
  | { page: "works"; params?: string }
  | { page: "work"; id: number; report: number | null }
  | { page: "reports"; params?: string }
  | { page: "report"; id: number }
  | { page: "properties"; params?: string }
  | { page: "property"; key: string; params?: string }
  | { page: "entities"; params?: string }
  | { page: "entity"; id: number }
  | { page: "effects"; params?: string }
  | { page: "compare"; params?: string }
  | { page: "about" }
  | { page: "missing"; path: string };

function integer(text: string | undefined): number | null {
  if (text === undefined || !/^[1-9]\d*$/.test(text)) return null;
  return Number(text);
}

/** The route with the parameters of the query, when there are some. */
function withParams<R extends Route>(route: R, query: URLSearchParams): R {
  const params = query.toString();
  return params === "" ? route : { ...route, params };
}

export function parseRoute(hash: string): Route {
  const address = hash.replace(/^#/, "");
  const mark = address.indexOf("?");
  const path = mark < 0 ? address : address.slice(0, mark);
  const query = new URLSearchParams(mark < 0 ? "" : address.slice(mark + 1));
  const parts = path.split("/").filter((part) => part.length > 0);
  const [first, second, ...rest] = parts;
  const missing: Route = { page: "missing", path };
  if (rest.length > 0) return missing;
  switch (first) {
    case undefined:
      return { page: "home" };
    case "works": {
      if (second === undefined) return withParams({ page: "works" }, query);
      const id = integer(second);
      return id === null
        ? missing
        : { page: "work", id, report: integer(query.get("report") ?? undefined) };
    }
    case "reports": {
      if (second === undefined) return withParams({ page: "reports" }, query);
      const id = integer(second);
      return id === null ? missing : { page: "report", id };
    }
    case "properties":
      return second === undefined
        ? withParams({ page: "properties" }, query)
        : withParams({ page: "property", key: decodeURIComponent(second) }, query);
    case "entities": {
      if (second === undefined) return withParams({ page: "entities" }, query);
      const id = integer(second);
      return id === null ? missing : { page: "entity", id };
    }
    case "effects":
      return second === undefined ? withParams({ page: "effects" }, query) : missing;
    case "compare":
      return second === undefined ? withParams({ page: "compare" }, query) : missing;
    case "about":
      return second === undefined ? { page: "about" } : missing;
    default:
      return missing;
  }
}

function querySuffix(params: string | undefined): string {
  return params === undefined || params === "" ? "" : `?${params}`;
}

export function link(route: Route): string {
  switch (route.page) {
    case "home":
      return "#/";
    case "works":
      return `#/works${querySuffix(route.params)}`;
    case "work":
      return route.report === null
        ? `#/works/${String(route.id)}`
        : `#/works/${String(route.id)}?report=${String(route.report)}`;
    case "reports":
      return `#/reports${querySuffix(route.params)}`;
    case "report":
      return `#/reports/${String(route.id)}`;
    case "properties":
      return `#/properties${querySuffix(route.params)}`;
    case "property":
      return `#/properties/${encodeURIComponent(route.key)}${querySuffix(route.params)}`;
    case "entities":
      return `#/entities${querySuffix(route.params)}`;
    case "entity":
      return `#/entities/${String(route.id)}`;
    case "effects":
      return `#/effects${querySuffix(route.params)}`;
    case "compare":
      return `#/compare${querySuffix(route.params)}`;
    case "about":
      return "#/about";
    case "missing":
      return `#/${route.path}`;
  }
}
