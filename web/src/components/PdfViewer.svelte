<script lang="ts">
  import * as pdfjs from "pdfjs-dist";
  import { SvelteSet } from "svelte/reactivity";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";

  import { anchor, type Match } from "../lib/anchor";
  import { mainMark, reveal, type Highlight } from "../lib/highlight";

  pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

  let {
    url,
    highlights,
    selected,
    onselect,
    onlocate,
  }: {
    url: string;
    highlights: readonly Highlight[];
    selected: number | null;
    onselect: (report: number) => void;
    onlocate: (located: ReadonlySet<number>) => void;
  } = $props();

  const SCALE = 1.4;

  interface Page {
    readonly page: pdfjs.PDFPageProxy;
    readonly viewport: pdfjs.PageViewport;
    readonly wrapper: HTMLDivElement;
    readonly canvas: HTMLCanvasElement;
  }

  let viewer: HTMLDivElement | undefined = $state();
  let pages: HTMLDivElement | undefined = $state();
  let status = $state("arity is loading the PDF.");
  let drawn = $state(0);
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
    return { page, viewport, wrapper, canvas };
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

  function rectangles(divs: readonly HTMLElement[], match: Match, page: HTMLElement): DOMRect[] {
    const first = divs[match.start.item]?.firstChild;
    const last = divs[match.end.item]?.firstChild;
    if (!(first instanceof Text) || !(last instanceof Text)) return [];
    const range = document.createRange();
    range.setStart(first, Math.min(match.start.offset, first.length));
    range.setEnd(last, Math.min(match.end.offset, last.length));
    const origin = page.getBoundingClientRect();
    return Array.from(range.getClientRects())
      .filter((rect) => rect.width > 0.5 && rect.height > 0.5)
      .map(
        (rect) =>
          new DOMRect(rect.left - origin.left, rect.top - origin.top, rect.width, rect.height),
      );
  }

  function mark(rect: DOMRect, highlight: Highlight): HTMLButtonElement {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "mark";
    button.dataset.report = String(highlight.report);
    button.dataset.evidence = String(highlight.index);
    button.setAttribute("aria-label", highlight.label);
    button.style.left = `${String(rect.x - 1)}px`;
    button.style.top = `${String(rect.y - 1)}px`;
    button.style.width = `${String(rect.width + 2)}px`;
    button.style.height = `${String(rect.height + 2)}px`;
    return button;
  }

  /** Builds the text layer of a page and marks each quote on it. */
  async function annotate(
    entry: Page,
    wanted: readonly Highlight[],
    located: Set<number>,
  ): Promise<void> {
    const text = document.createElement("div");
    text.className = "textLayer";
    const marks = document.createElement("div");
    marks.className = "marks";
    entry.wrapper.append(text, marks);
    const layer = new pdfjs.TextLayer({
      textContentSource: await entry.page.getTextContent(),
      container: text,
      viewport: entry.viewport,
    });
    await layer.render();
    const items = layer.textContentItemsStr;
    for (const highlight of wanted) {
      for (const match of anchor(items, highlight.quote)) {
        for (const rect of rectangles(layer.textDivs, match, entry.wrapper)) {
          marks.append(mark(rect, highlight));
          located.add(highlight.report);
        }
      }
    }
  }

  $effect(() => {
    const target = pages;
    const scroller = viewer;
    if (!target || !scroller) return;
    const wanted = highlights;
    let cancelled = false;
    const task = pdfjs.getDocument({ url });
    const located = new SvelteSet<number>();
    const painted = new WeakSet<Element>();
    let entries: Page[] = [];
    const observer = new IntersectionObserver(
      (changes) => {
        for (const change of changes) {
          const entry = entries.find((candidate) => candidate.wrapper === change.target);
          if (!change.isIntersecting || !entry || painted.has(entry.wrapper)) continue;
          painted.add(entry.wrapper);
          observer.unobserve(entry.wrapper);
          paint(entry).catch(() => {
            painted.delete(entry.wrapper);
          });
        }
      },
      { root: scroller, rootMargin: "1200px 0px" },
    );

    async function draw(container: HTMLDivElement): Promise<void> {
      const pdf = await task.promise;
      const laid: Page[] = [];
      for (let number = 1; number <= pdf.numPages; number += 1) {
        if (cancelled) return;
        laid.push(lay(await pdf.getPage(number), number, container));
      }
      entries = laid;
      for (const entry of laid) observer.observe(entry.wrapper);
      for (const entry of laid) {
        if (cancelled) return;
        await annotate(entry, wanted, located);
      }
    }

    draw(target)
      .then(() => {
        if (cancelled) return;
        status = "";
        drawn += 1;
        onlocate(located);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          status = `The PDF cannot show: ${error instanceof Error ? error.message : String(error)}`;
        }
      });

    return () => {
      cancelled = true;
      observer.disconnect();
      void task.destroy();
      target.replaceChildren();
    };
  });

  $effect(() => {
    const target = pages;
    if (!target || drawn === 0) return;
    const current = selected === null ? null : String(selected);
    for (const element of target.querySelectorAll<HTMLElement>(".mark")) {
      element.classList.toggle("selected", element.dataset.report === current);
    }
    const main = selected === null ? null : mainMark(target, selected);
    if (main && viewer) reveal(viewer, main);
  });

  function reportOf(event: Event): number | null {
    const element =
      event.target instanceof HTMLElement ? event.target.closest<HTMLElement>(".mark") : null;
    const value = element?.dataset.report;
    return value === undefined ? null : Number(value);
  }

  function hover(event: PointerEvent): void {
    const report = reportOf(event);
    const highlight = highlights.find((candidate) => candidate.report === report);
    tip = highlight
      ? { x: event.clientX + 14, y: event.clientY + 14, text: highlight.label }
      : null;
  }

  function choose(event: MouseEvent): void {
    const report = reportOf(event);
    if (report !== null) onselect(report);
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
