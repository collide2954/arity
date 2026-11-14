<script lang="ts">
  import * as pdfjs from "pdfjs-dist";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";

  import { finder, type Match } from "../lib/anchor";
  import {
    labelsOf,
    mainMark,
    place,
    quotedParts,
    reveal,
    type Highlight,
    type Mark,
  } from "../lib/highlight";
  import type { Block } from "../lib/types";

  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

  let {
    url,
    blocks,
    highlights,
    selected,
    onselect,
    onlocate,
  }: {
    url: string;
    blocks: readonly Block[];
    highlights: readonly Highlight[];
    selected: number | null;
    onselect: (report: number) => void;
    onlocate: (located: ReadonlySet<number>) => void;
  } = $props();

  const SCALE = 1.4;

  type TextContent = Awaited<ReturnType<pdfjs.PDFPageProxy["getTextContent"]>>;

  interface Page {
    readonly page: pdfjs.PDFPageProxy;
    readonly viewport: pdfjs.PageViewport;
    readonly wrapper: HTMLDivElement;
    readonly canvas: HTMLCanvasElement;
    /** True when the page has come near the view. */
    near: boolean;
    /** The text layer and the marks of the page, when arity adds them. */
    annotated: Promise<HTMLElement | null> | null;
  }

  interface Shown {
    /** The page of the main mark of each report that the PDF shows. */
    readonly pages: ReadonlyMap<number, number>;
    /** Adds the text layer and the marks of a page one time, and gives the page. */
    readonly show: (page: number) => Promise<HTMLElement | null>;
  }

  let viewer: HTMLDivElement | undefined = $state();
  let pages: HTMLDivElement | undefined = $state();
  let status = $state("arity is loading the PDF.");
  let shown: Shown | null = $state.raw(null);
  let tip: { x: number; y: number; text: string } | null = $state(null);

  function sized(element: HTMLElement, viewport: pdfjs.PageViewport): void {
    element.style.width = `${String(viewport.width)}px`;
    element.style.height = `${String(viewport.height)}px`;
  }

  function lay(page: pdfjs.PDFPageProxy, number: number, container: HTMLDivElement): Page {
    const viewport = page.getViewport({ scale: SCALE });
    const wrapper = document.createElement("div");
    wrapper.className = "page";
    wrapper.dataset.page = String(number);
    sized(wrapper, viewport);
    wrapper.style.setProperty("--scale-factor", String(viewport.scale));
    wrapper.style.setProperty("--total-scale-factor", String(viewport.scale));
    wrapper.style.setProperty("--user-unit", "1");
    const canvas = document.createElement("canvas");
    sized(canvas, viewport);
    wrapper.append(canvas);
    container.append(wrapper);
    return { page, viewport, wrapper, canvas, near: false, annotated: null };
  }

  async function paint(entry: Page): Promise<void> {
    const ratio = window.devicePixelRatio || 1;
    entry.canvas.width = Math.floor(entry.viewport.width * ratio);
    entry.canvas.height = Math.floor(entry.viewport.height * ratio);
    await entry.page.render({
      canvas: entry.canvas,
      viewport: entry.viewport,
      ...(ratio === 1 ? {} : { transform: [ratio, 0, 0, ratio, 0, 0] }),
    }).promise;
  }

  const labels = $derived(labelsOf(highlights));

  function rectangles(divs: readonly HTMLElement[], match: Match, origin: DOMRect): DOMRect[] {
    const first = divs[match.start.item]?.firstChild;
    const last = divs[match.end.item]?.firstChild;
    if (!(first instanceof Text) || !(last instanceof Text)) return [];
    const range = document.createRange();
    range.setStart(first, Math.min(match.start.offset, first.length));
    range.setEnd(last, Math.min(match.end.offset, last.length));
    return Array.from(range.getClientRects())
      .filter((rect) => rect.width > 0.5 && rect.height > 0.5)
      .map(
        (rect) =>
          new DOMRect(rect.left - origin.left, rect.top - origin.top, rect.width, rect.height),
      );
  }

  function describe(reports: readonly number[]): string {
    const first = reports[0] === undefined ? "" : (labels.get(reports[0]) ?? "");
    return reports.length > 1 ? `${first} (and ${String(reports.length - 1)} more)` : first;
  }

  function button(box: DOMRect, mark: Mark): HTMLButtonElement {
    const element = document.createElement("button");
    element.type = "button";
    element.className = "mark";
    element.classList.toggle("selected", selected !== null && mark.reports.includes(selected));
    element.dataset.reports = mark.reports.join(" ");
    element.dataset.main = mark.main.join(" ");
    element.setAttribute("aria-label", describe(mark.reports));
    element.style.left = `${String(box.x - 1)}px`;
    element.style.top = `${String(box.y - 1)}px`;
    element.style.width = `${String(box.width + 2)}px`;
    element.style.height = `${String(box.height + 2)}px`;
    return element;
  }

  /** Adds the text layer of a page, and then its marks in one step. */
  async function annotate(
    entry: Page,
    content: TextContent,
    marks: readonly Mark[],
  ): Promise<void> {
    const text = document.createElement("div");
    text.className = "textLayer";
    const layer = document.createElement("div");
    layer.className = "marks";
    entry.wrapper.append(text, layer);
    const textLayer = new pdfjs.TextLayer({
      textContentSource: content,
      container: text,
      viewport: entry.viewport,
    });
    await textLayer.render();
    const origin = entry.wrapper.getBoundingClientRect();
    layer.append(
      ...marks.flatMap((mark) =>
        rectangles(textLayer.textDivs, mark.match, origin).map((box) => button(box, mark)),
      ),
    );
  }

  /** The text of each item that the text layer shows, in the order of its text divs. */
  function itemsOf(content: TextContent): string[] {
    return content.items.flatMap((item) => ("str" in item ? [item.str] : []));
  }

  $effect(() => {
    const target = pages;
    const scroller = viewer;
    if (!target || !scroller) return;
    const parts = quotedParts(blocks, highlights);
    let cancelled = false;
    const task = pdfjs.getDocument({ url });
    const laid: Page[] = [];
    /** Adds the text layer and the marks of a page, when arity has found the quotes. */
    let build: ((page: number) => Promise<HTMLElement | null>) | null = null;

    function fail(error: unknown): void {
      if (!cancelled) {
        status = `The PDF cannot show: ${error instanceof Error ? error.message : String(error)}`;
      }
    }

    const observer = new IntersectionObserver(
      (changes) => {
        for (const change of changes) {
          const index = laid.findIndex((entry) => entry.wrapper === change.target);
          const entry = laid[index];
          if (!change.isIntersecting || !entry) continue;
          observer.unobserve(entry.wrapper);
          entry.near = true;
          paint(entry).catch(fail);
          void build?.(index);
        }
      },
      { root: scroller, rootMargin: "1200px 0px" },
    );

    async function draw(container: HTMLDivElement): Promise<void> {
      const pdf = await task.promise;
      for (let number = 1; number <= pdf.numPages; number += 1) {
        if (cancelled) return;
        laid.push(lay(await pdf.getPage(number), number, container));
      }
      for (const entry of laid) observer.observe(entry.wrapper);
      const contents = await Promise.all(laid.map((entry) => entry.page.getTextContent()));
      if (cancelled) return;
      const placed = place(parts, finder(contents.map(itemsOf)));
      const show = (index: number): Promise<HTMLElement | null> => {
        const entry = laid[index];
        const content = contents[index];
        if (!entry || !content) return Promise.resolve(null);
        entry.annotated ??= annotate(entry, content, placed.marks.get(index) ?? [])
          .then(() => entry.wrapper)
          .catch((error: unknown) => {
            fail(error);
            return null;
          });
        return entry.annotated;
      };
      build = show;
      laid.forEach((entry, index) => {
        if (entry.near) void show(index);
      });
      shown = { pages: placed.pages, show };
      status = "";
      onlocate(new Set(placed.pages.keys()));
    }

    draw(target).catch(fail);

    return () => {
      cancelled = true;
      observer.disconnect();
      void task.destroy();
      target.replaceChildren();
      shown = null;
    };
  });

  $effect(() => {
    const target = pages;
    const scroller = viewer;
    const current = shown;
    const report = selected;
    if (!target || !scroller || !current) return;
    for (const element of target.querySelectorAll<HTMLElement>(".mark.selected")) {
      element.classList.remove("selected");
    }
    if (report === null) return;
    for (const element of target.querySelectorAll<HTMLElement>(
      `[data-reports~="${String(report)}"]`,
    )) {
      element.classList.add("selected");
    }
    const page = current.pages.get(report);
    if (page === undefined) return;
    let stale = false;
    void current.show(page).then((wrapper) => {
      const main = wrapper && mainMark(wrapper, report);
      if (!stale && main) reveal(scroller, main);
    });
    return () => {
      stale = true;
    };
  });

  /** The reports of the mark under the pointer, with its main reports first. */
  function reportsOf(event: Event): number[] {
    const element =
      event.target instanceof HTMLElement ? event.target.closest<HTMLElement>(".mark") : null;
    if (!element) return [];
    const words = (value: string | undefined) =>
      (value ?? "").split(" ").filter(Boolean).map(Number);
    const main = words(element.dataset.main);
    return [...main, ...words(element.dataset.reports).filter((report) => !main.includes(report))];
  }

  function hover(event: PointerEvent): void {
    const reports = reportsOf(event);
    tip =
      reports.length > 0
        ? { x: event.clientX + 14, y: event.clientY + 14, text: describe(reports) }
        : null;
  }

  function choose(event: MouseEvent): void {
    const [report] = reportsOf(event);
    if (report !== undefined) onselect(report);
  }
</script>

<div class="viewer" bind:this={viewer}>
  {#if status}<p class="muted status">{status}</p>{/if}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div
    class="pages"
    bind:this={pages}
    onpointermove={hover}
    onpointerleave={() => (tip = null)}
    onclick={choose}
  ></div>
  {#if tip}
    <div class="tip" role="tooltip" style="left: {tip.x}px; top: {tip.y}px">{tip.text}</div>
  {/if}
</div>

<style>
  .viewer {
    position: relative;
    overflow: auto;
    max-height: 80vh;
    background: var(--line);
    border-radius: 8px;
    padding: 12px;
  }

  .status {
    margin: 0 0 0.5rem;
  }

  .pages {
    display: grid;
    gap: 12px;
    justify-content: center;
  }

  .pages :global(.page) {
    position: relative;
    background: #ffffff;
    box-shadow: 0 1px 3px rgb(0 0 0 / 0.2);
  }

  .pages :global(canvas) {
    display: block;
  }

  .pages :global(.textLayer) {
    position: absolute;
    inset: 0;
    overflow: clip;
    line-height: 1;
    text-align: initial;
    transform-origin: 0 0;
    --min-font-size: 1;
    --text-scale-factor: calc(var(--total-scale-factor) * var(--min-font-size));
    --min-font-size-inv: calc(1 / var(--min-font-size));
  }

  .pages :global(.textLayer span),
  .pages :global(.textLayer br) {
    color: transparent;
    position: absolute;
    white-space: pre;
    cursor: text;
    transform-origin: 0% 0%;
  }

  .pages :global(.textLayer > :not(.markedContent)),
  .pages :global(.textLayer .markedContent span:not(.markedContent)) {
    --font-height: 0;
    font-size: calc(var(--text-scale-factor) * var(--font-height));
    --scale-x: 1;
    --rotate: 0deg;
    transform: rotate(var(--rotate)) scaleX(var(--scale-x)) scale(var(--min-font-size-inv));
  }

  .pages :global(.textLayer .markedContent) {
    display: contents;
  }

  .pages :global(.marks) {
    position: absolute;
    inset: 0;
    pointer-events: none;
  }

  .pages :global(.mark) {
    position: absolute;
    pointer-events: auto;
    border: 0;
    padding: 0;
    border-radius: 2px;
    background: var(--mark);
    mix-blend-mode: multiply;
    cursor: pointer;
  }

  .pages :global(.mark:hover),
  .pages :global(.mark:focus-visible),
  .pages :global(.mark.selected) {
    background: var(--mark-strong);
    outline: 2px solid var(--accent);
  }

  .tip {
    position: fixed;
    z-index: 20;
    max-width: 22rem;
    background: var(--surface);
    color: var(--ink);
    box-shadow:
      0 0 0 1px var(--ring),
      0 4px 12px rgb(0 0 0 / 0.15);
    border-radius: 6px;
    padding: 0.4rem 0.6rem;
    font-size: 0.85rem;
    pointer-events: none;
  }
</style>
