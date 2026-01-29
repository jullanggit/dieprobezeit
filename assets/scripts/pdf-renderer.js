// keep versions in sync with main.rs
import * as pdfjsLib from "https://unpkg.com/pdfjs-dist@5.4.530/build/pdf.mjs";
import * as pdfjsViewer from "https://unpkg.com/pdfjs-dist@5.4.530/web/pdf_viewer.mjs";

// how much of the page should be vertically visible, at a minimum
const MIN_VISIBLE_PAGE_FRACTION = 0.7;

const PDFJS_CDN_BASE = "https://unpkg.com/pdfjs-dist@latest";
const CONTAINER_SELECTOR = ".pdfjs-container[data-pdf-src]";
const renderState = new WeakMap();

pdfjsLib.GlobalWorkerOptions.workerSrc =
  `${PDFJS_CDN_BASE}/build/pdf.worker.min.mjs`;

function scheduleRender(container) {
  const state = renderState.get(container) || {};

  if (container.dataset.pdfjsRendering === "true") {
    state.pending = true;
    renderState.set(container, state);
    return;
  }

  if (state.timer) clearTimeout(state.timer);
  state.timer = setTimeout(() => renderPdf(container), 60);
  renderState.set(container, state);
}

function setupResizeObserver(container) {
  if (container._pdfResizeObserver) return;

  const observer = new ResizeObserver((entries) => {
    for (const entry of entries) {
      const width = Math.round(entry.contentRect.width);
      const lastWidth = Number(container.dataset.pdfjsWidth || 0);
      if (width && Math.abs(width - lastWidth) > 2) {
        container.dataset.pdfjsWidth = String(width);
        scheduleRender(container);
      }
    }
  });

  observer.observe(container);
  container._pdfResizeObserver = observer;

  if (!container._pdfDprListener) {
      container._pdfDprListener = true;

      const listenForDprChange = () => {
        const query = matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);

        query.addEventListener('change', () => {
          scheduleRender(container);
          listenForDprChange();
        }, { once: true });
      };

      listenForDprChange();
    }
}

async function renderPdf(container) {
  const pdfUrl = container.dataset.pdfSrc;
  if (!pdfUrl) return;
  if (container.dataset.pdfjsRendering === "true") return;

  container.dataset.pdfjsRendering = "true";
  container.innerHTML = "Loading PDF...";

  const loadingTask = pdfjsLib.getDocument({ url: pdfUrl });

  try {
    const pdf = await loadingTask.promise;

    const eventBus = new pdfjsViewer.EventBus();
    const linkService = new pdfjsViewer.PDFLinkService({
      eventBus,
      externalLinkTarget: pdfjsViewer.LinkTarget.BLANK,
      externalLinkRel: "noopener noreferrer",
    });
    linkService.setDocument(pdf, null);

    const annotationStorage = pdf.annotationStorage;

    const pageViews = new Map();
    container.innerHTML = "";

    // Minimal viewer shim so internal links can scroll pages into view.
    const viewer = {
      currentPageNumber: 1,
      scrollPageIntoView({ pageNumber, destArray }) {
        this.currentPageNumber = pageNumber;

        const pageView = pageViews.get(pageNumber);
        if (!pageView) return;

        // No destination array -> just jump to page top.
        if (!destArray) {
          pageView.div.scrollIntoView({ behavior: "smooth", block: "start" });
          return;
        }

        const destName = destArray[1] && destArray[1].name;
        let top = null;

        if (destName === "XYZ") top = destArray[3];
        else if (destName === "FitH" || destName === "FitBH") top = destArray[2];

        let yOffset = 0;
        if (top !== null && top !== undefined) {
          const [, y] = pageView.viewport.convertToViewportPoint(0, top);
          yOffset = y;
        }

        const pageTop = pageView.div.getBoundingClientRect().top + window.scrollY;
        window.scrollTo({ top: pageTop + yOffset, behavior: "smooth" });
      },
    };
    linkService.setViewer(viewer);

    for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
      const page = await pdf.getPage(pageNumber);

      const baseViewport = page.getViewport({ scale: 1 });
      const availableWidth =
        container.clientWidth ||
        container.parentElement?.clientWidth ||
        baseViewport.width;

      const scaleToWidth = availableWidth / baseViewport.width;

      const viewportH = window.visualViewport?.height ?? window.innerHeight;
      const scaleMinVertical = viewportH / (baseViewport.height * MIN_VISIBLE_PAGE_FRACTION);

      const visualScale = Math.min(scaleToWidth, scaleMinVertical);

      const viewport = page.getViewport({ scale: visualScale });

      const pageDiv = document.createElement("div");
      pageDiv.className = "pdf-page";
      pageDiv.dataset.pageNumber = String(pageNumber);
      pageDiv.style.width = `${Math.round(viewport.width)}px`;
      pageDiv.style.height = `${Math.round(viewport.height)}px`;

      const canvas = document.createElement("canvas");
      canvas.className = "pdf-canvas";

      const pixelRatio = window.devicePixelRatio || 1;
      canvas.width = Math.round(viewport.width * pixelRatio);
      canvas.height = Math.round(viewport.height * pixelRatio);
      canvas.style.width = `${Math.round(viewport.width)}px`;
      canvas.style.height = `${Math.round(viewport.height)}px`;

      pageDiv.appendChild(canvas);

      const annotationLayerDiv = document.createElement("div");
      annotationLayerDiv.className = "annotationLayer";
      pageDiv.appendChild(annotationLayerDiv);

      const textLayerDiv = document.createElement("div");
      textLayerDiv.className = "textLayer";
      pageDiv.appendChild(textLayerDiv);

      container.appendChild(pageDiv);

      await page.render({
        canvasContext: canvas.getContext('2d'),
        viewport,
        transform: [pixelRatio, 0, 0, pixelRatio, 0, 0],
      }).promise;

      const annotations = await page.getAnnotations({ intent: "display" });

      const annotationCanvasMap = new Map();
      const annotationLayer = new pdfjsLib.AnnotationLayer({
        div: annotationLayerDiv,
        accessibilityManager: null,
        annotationCanvasMap,
        annotationEditorUIManager: null,
        page,
        viewport,
        structTreeLayer: null,
        commentManager: null,
        linkService,
        annotationStorage,
      });

      await annotationLayer.render({
        viewport,
        div: annotationLayerDiv,
        annotations,
        page,
        linkService,
        annotationStorage,
        annotationCanvasMap,
        renderForms: false,
        // Needed so link icons etc. resolve correctly when using the CDN. :contentReference[oaicite:7]{index=7}
        imageResourcesPath: `${PDFJS_CDN_BASE}/web/images/`,
      });

      const textContent = await page.getTextContent();
      const textLayer = new pdfjsLib.TextLayer({
          textContentSource: textContent,
          viewport: viewport,
          container: textLayerDiv,
      });
      await textLayer.render(textContent);

      console.log("DPR:", window.devicePixelRatio);
      console.log("Base Viewport (scale=1):", page.getViewport({ scale: 1 }));
      console.log("Calculated scale:", visualScale);
      console.log("Final viewport dimensions:", viewport.width, viewport.height);

      pageViews.set(pageNumber, { div: pageDiv, viewport });
    }

    container.dataset.pdfjsWidth = String(Math.round(container.clientWidth || 0));
    setupResizeObserver(container);
  } catch (error) {
    container.innerHTML = "Failed to load PDF.";
    console.error("Failed to render PDF", error);
  } finally {
    container.dataset.pdfjsRendering = "false";

    const state = renderState.get(container) || {};
    if (state.pending) {
      state.pending = false;
      renderState.set(container, state);
      scheduleRender(container);
    }
  }
}

function scanAndRender() {
  document.querySelectorAll(CONTAINER_SELECTOR).forEach((c) => scheduleRender(c));
}

function observeDom() {
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes) {
        if (!(node instanceof HTMLElement)) continue;

        if (node.matches?.(CONTAINER_SELECTOR)) {
          scheduleRender(node);
        } else {
          node.querySelectorAll?.(CONTAINER_SELECTOR).forEach((c) => scheduleRender(c));
        }
      }
    }
  });

  observer.observe(document.body, { childList: true, subtree: true });
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    scanAndRender();
    observeDom();
  });
} else {
  scanAndRender();
  observeDom();
}
