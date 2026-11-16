import * as pdfjsLib from "pdfjs-dist";
import workerSrc from "pdfjs-dist/build/pdf.worker.min.mjs?url";

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
  renderPage: (
    pageNum: number,
    canvas: HTMLCanvasElement,
    textLayerEl: HTMLDivElement | null,
    options: PdfRenderOptions,
  ) => Promise<number>;
  destroy: () => Promise<void>;
}

export async function openPdfSession(buffer: ArrayBuffer): Promise<PdfSession> {
  const loadingTask = pdfjsLib.getDocument({ data: new Uint8Array(buffer) });
  const doc = await loadingTask.promise;
  let activeRenderTask: { cancel: () => void; promise: Promise<void> } | null = null;

  async function renderPage(
    pageNum: number,
    canvas: HTMLCanvasElement,
    textLayerEl: HTMLDivElement | null,
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

    if (textLayerEl) {
      textLayerEl.innerHTML = "";
      textLayerEl.style.width = `${baseViewport.width}px`;
      textLayerEl.style.height = `${baseViewport.height}px`;
      textLayerEl.style.setProperty("--total-scale-factor", String(effectiveScale));
      textLayerEl.style.setProperty("--scale-factor", String(effectiveScale));
      try {
        const textContent = await page.getTextContent();
        const textLayer = new pdfjsLib.TextLayer({
          textContentSource: textContent,
          container: textLayerEl,
          viewport: baseViewport,
        });
        await textLayer.render();
      } catch {
        // ignore text layer errors on image-only pages
      }
    }

    return effectiveScale;
  }

  return {
    numPages: doc.numPages,
    renderPage,
    destroy: () => loadingTask.destroy(),
  };
}
