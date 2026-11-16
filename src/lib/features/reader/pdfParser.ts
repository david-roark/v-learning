import * as pdfjsLib from "pdfjs-dist";
import workerSrc from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { joinPdfLines } from "./textParser";

pdfjsLib.GlobalWorkerOptions.workerSrc = workerSrc;

export interface BookSection {
  title: string;
  text: string;
}

export type PdfFitMode = "custom" | "fit-width" | "fit-page";

export interface PdfRenderOptions {
  scale: number;
  fitMode: PdfFitMode;
  containerWidth: number;
  containerHeight: number;
}

export interface PdfSession {
  numPages: number;
  getPageText: (pageNum: number) => Promise<string>;
  renderPage: (
    pageNum: number,
    canvas: HTMLCanvasElement,
    options: PdfRenderOptions,
  ) => Promise<number>;
  destroy: () => Promise<void>;
}

export async function openPdfSession(buffer: ArrayBuffer): Promise<PdfSession> {
  const loadingTask = pdfjsLib.getDocument({ data: new Uint8Array(buffer) });
  const doc = await loadingTask.promise;
  const cache = new Map<number, string>();
  let activeRenderTask: { cancel: () => void; promise: Promise<void> } | null = null;

  async function getPageText(pageNum: number): Promise<string> {
    const cached = cache.get(pageNum);
    if (cached !== undefined) return cached;

    const page = await doc.getPage(pageNum);
    const content = await page.getTextContent();
    const lines: string[] = [];
    let currentLine = "";

    for (const item of content.items) {
      if ("str" in item) {
        currentLine += item.str;
        if (item.hasEOL) {
          lines.push(currentLine);
          currentLine = "";
        } else {
          currentLine += " ";
        }
      }
    }
    if (currentLine.trim()) {
      lines.push(currentLine);
    }

    const joined = joinPdfLines(lines);
    cache.set(pageNum, joined);
    return joined;
  }

  async function renderPage(
    pageNum: number,
    canvas: HTMLCanvasElement,
    options: PdfRenderOptions,
  ): Promise<number> {
    if (activeRenderTask) {
      activeRenderTask.cancel();
      try {
        await activeRenderTask.promise;
      } catch {
        // expected RenderingCancelledException
      }
      activeRenderTask = null;
    }

    const page = await doc.getPage(pageNum);
    const natural = page.getViewport({ scale: 1 });

    let effectiveScale = options.scale;
    if (options.fitMode === "fit-width" && options.containerWidth > 40) {
      effectiveScale = (options.containerWidth - 32) / natural.width;
    } else if (
      options.fitMode === "fit-page" &&
      options.containerWidth > 40 &&
      options.containerHeight > 40
    ) {
      const scaleW = (options.containerWidth - 32) / natural.width;
      const scaleH = (options.containerHeight - 32) / natural.height;
      effectiveScale = Math.min(scaleW, scaleH);
    }
    effectiveScale = Math.max(0.4, Math.min(3.0, effectiveScale));

    const dpr = window.devicePixelRatio || 1;
    const baseViewport = page.getViewport({ scale: effectiveScale });
    const viewport = page.getViewport({ scale: effectiveScale * dpr });

    canvas.width = viewport.width;
    canvas.height = viewport.height;
    canvas.style.width = `${baseViewport.width}px`;
    canvas.style.height = `${baseViewport.height}px`;

    const ctx = canvas.getContext("2d");
    if (!ctx) return effectiveScale;

    const task = page.render({ canvas, canvasContext: ctx, viewport });
    activeRenderTask = task;
    try {
      await task.promise;
    } catch {
      // ignore cancelled render
    } finally {
      if (activeRenderTask === task) activeRenderTask = null;
    }

    return effectiveScale;
  }

  return {
    numPages: doc.numPages,
    getPageText,
    renderPage,
    destroy: () => loadingTask.destroy(),
  };
}
