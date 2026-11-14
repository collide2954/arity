<script lang="ts">
  import * as pdfjs from "pdfjs-dist";
  import { SvelteSet } from "svelte/reactivity";
  import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";

  import { anchor, type Match } from "../lib/anchor";
  import type { Highlight } from "../lib/highlight";

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

  let pages: HTMLDivElement | undefined = $state();
  let status = $state("arity is loading the PDF.");
  let tip: { x: number; y: number; text: string } | null = $state(null);

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
    button.setAttribute("aria-label", highlight.label);
    button.style.left = `${String(rect.x - 1)}px`;
    button.style.top = `${String(rect.y - 1)}px`;
    button.style.width = `${String(rect.width + 2)}px`;
    button.style.height = `${String(rect.height + 2)}px`;
    return button;
  }

  $effect(() => {
    const target = pages;
    if (!target) return;
    const wanted = highlights;
    let cancelled = false;
    const task = pdfjs.getDocument({ url });
    const located = new SvelteSet<number>();

    async function draw(container: HTMLDivElement): Promise<void> {
      const pdf = await task.promise;
      for (let number = 1; number <= pdf.numPages; number += 1) {
        if (cancelled) return;
        const page = await pdf.getPage(number);
        const viewport = page.getViewport({ scale: SCALE });
        const ratio = window.devicePixelRatio || 1;
        const wrapper = document.createElement("div");
        wrapper.className = "page";
        wrapper.dataset.page = String(number);
        wrapper.style.width = `${String(viewport.width)}px`;
        wrapper.style.height = `${String(viewport.height)}px`;
        wrapper.style.setProperty("--scale-factor", String(viewport.scale));
        wrapper.style.setProperty("--total-scale-factor", String(viewport.scale));
        wrapper.style.setProperty("--user-unit", "1");
        const canvas = document.createElement("canvas");
        canvas.width = Math.floor(viewport.width * ratio);
        canvas.height = Math.floor(viewport.height * ratio);
        canvas.style.width = `${String(viewport.width)}px`;
        canvas.style.height = `${String(viewport.height)}px`;
        const text = document.createElement("div");
        text.className = "textLayer";
        const marks = document.createElement("div");
        marks.className = "marks";
        wrapper.append(canvas, text, marks);
        container.append(wrapper);
        await page.render({
          canvas,
          viewport,
          ...(ratio === 1 ? {} : { transform: [ratio, 0, 0, ratio, 0, 0] }),
        }).promise;
        const layer = new pdfjs.TextLayer({
          textContentSource: await page.getTextContent(),
          container: text,
          viewport,
        });
        await layer.render();
        const items = layer.textContentItemsStr;
        for (const highlight of wanted) {
          for (const match of anchor(items, highlight.quote)) {
            for (const rect of rectangles(layer.textDivs, match, wrapper)) {
              marks.append(mark(rect, highlight));
              located.add(highlight.report);
            }
          }
        }
      }
    }

    draw(target)
      .then(() => {
        if (cancelled) return;
        status = "";
        onlocate(located);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          status = `The PDF cannot show: ${error instanceof Error ? error.message : String(error)}`;
        }
      });

    return () => {
      cancelled = true;
      void task.destroy();
      target.replaceChildren();
    };
  });

  $effect(() => {
    const target = pages;
    if (!target) return;
    const current = selected === null ? null : String(selected);
    for (const element of target.querySelectorAll<HTMLElement>(".mark")) {
      element.classList.toggle("selected", element.dataset.report === current);
    }
    if (current !== null) {
      target
        .querySelector<HTMLElement>(`.mark[data-report="${current}"]`)
        ?.scrollIntoView({ behavior: "smooth", block: "center" });
    }
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

<div class="viewer">
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
