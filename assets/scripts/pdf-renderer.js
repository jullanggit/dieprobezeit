// keep versions in sync with main.rs
import * as pdfjsLib from "https://unpkg.com/pdfjs-dist@5.4.530/build/pdf.mjs";
import * as pdfjsViewer from "https://unpkg.com/pdfjs-dist@5.4.530/web/pdf_viewer.mjs";
const PDFJS_CDN_BASE = "https://unpkg.com/pdfjs-dist@5.4.530";

// how much of the page should be vertically visible, at a minimum
const MIN_VISIBLE_PAGE_FRACTION = 0.7;

const CONTAINER_SELECTOR = ".pdfjs-container[data-pdf-src]";
const renderState = new WeakMap();

pdfjsLib.GlobalWorkerOptions.workerSrc = `${PDFJS_CDN_BASE}/build/pdf.worker.min.mjs`;

function getState(container) {
  const state = renderState.get(container) || {};
  if (!state.pages) state.pages = [];
  if (!state.readTimesSetup) state.readTimesSetup = false;
  if (!state.generation) state.generation = 0;
  if (!state.renderTasks) state.renderTasks = new Map();
  if (!state.stopReadTimes) state.stopReadTimes = () => {};
  state.pdf ??= null;
  state.loadingTask ??= null;

  return state;
}

async function destroyDocument(state) {
  state.stopReadTimes?.();
  cancelRenderWork(state);

  // release PDF.js resources (main + worker thread)
  try {
    await state.pdf?.cleanup?.();
  } catch {}
  try {
    await state.pdf?.destroy?.();
  } catch {}
  state.pdf = null;

  try {
    await state.loadingTask?.destroy?.();
  } catch {}
  state.loadingTask = null;
}

function destroyContainer(container) {
  const state = renderState.get(container);
  if (!state) return;

  try {
    clearTimeout(state.timer);
  } catch {}
  try {
    container._pdfResizeObserver?.disconnect?.();
  } catch {}
  container._pdfResizeObserver = null;

  destroyDocument(state);
  renderState.delete(container);
}

function scheduleRender(container) {
  const state = getState(container);

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

      query.addEventListener(
        "change",
        () => {
          scheduleRender(container);
          listenForDprChange();
        },
        { once: true },
      );
    };

    listenForDprChange();
  }
}

function getVisualViewport() {
  return window.visualViewport
    ? window.visualViewport
    : {
        width: window.innerWidth,
        height: window.innerHeight,
        scale: 1,
      };
}

async function renderPdf(container) {
  const pdfUrl = container.dataset.pdfSrc;
  if (!pdfUrl) return;
  if (container.dataset.pdfjsRendering === "true") return;

  const state = getState(container);
  state.generation += 1;
  const generation = state.generation;

  cancelRenderWork(state);

  // If the PDF changed, drop cached page divs + read-time wiring.
  if (state.pdfUrl !== pdfUrl) {
    state.pdfUrl = pdfUrl;
    state.pages = [];
    state.readTimesSetup = false;

    await destroyDocument(state);
  }

  container.dataset.pdfjsRendering = "true";
  container.innerHTML = "Loading PDF...";

  renderState.set(container, state);

  try {
    let pdf = state.pdf;

    if (!pdf) {
      const loadingTask = pdfjsLib.getDocument({ url: pdfUrl });
      state.loadingTask = loadingTask;
      renderState.set(container, state);

      pdf = await loadingTask.promise;
      if (getState(container).generation !== generation) {
        // generation changed mid-load; release what we loaded
        try {
          await pdf.destroy?.();
        } catch {}
        return;
      }

      state.pdf = pdf;
      state.loadingTask = null;
      renderState.set(container, state);
    }

    if (getState(container).generation !== generation) return;

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
        else if (destName === "FitH" || destName === "FitBH")
          top = destArray[2];

        let yOffset = 0;
        if (top !== null && top !== undefined) {
          const [, y] = pageView.viewport.convertToViewportPoint(0, top);
          yOffset = y;
        }

        const pageTop =
          pageView.div.getBoundingClientRect().top + window.scrollY;
        window.scrollTo({ top: pageTop + yOffset, behavior: "smooth" });
      },
    };
    linkService.setViewer(viewer);

    for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
      const newState = getState(container);
      if (newState.generation !== generation) {
        cancelRenderWork(newState);
        return;
      }

      const page = await pdf.getPage(pageNumber);

      const baseViewport = page.getViewport({ scale: 1 });
      const availableWidth =
        container.clientWidth ||
        container.parentElement?.clientWidth ||
        baseViewport.width;

      const scaleToWidth = availableWidth / baseViewport.width;

      const visualViewport = getVisualViewport();
      const scaleMinVertical =
        visualViewport.height /
        (baseViewport.height * MIN_VISIBLE_PAGE_FRACTION);

      const visualScale = Math.min(scaleToWidth, scaleMinVertical);
      const finalScale =
        visualViewport.scale > 1
          ? visualScale * visualViewport.scale
          : visualScale;

      const viewport = page.getViewport({ scale: finalScale });

      const pages = state.pages;
      const pageDiv = (pages[pageNumber - 1] ??= document.createElement("div"));

      pageDiv.className = "pdf-page";
      pageDiv.dataset.pageNumber = String(pageNumber);
      pageDiv.style.width = `${Math.round(viewport.width)}px`;
      pageDiv.style.height = `${Math.round(viewport.height)}px`;

      pageDiv.replaceChildren();

      const canvas = document.createElement("canvas");
      canvas.className = "pdf-canvas";

      const context = canvas.getContext("2d");

      const RESOLUTION_MULTIPLIER = 2;
      const pixelRatio = RESOLUTION_MULTIPLIER * (window.devicePixelRatio || 1);

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

      const renderTask = page.render({
        canvasContext: context,
        viewport,
        transform: [pixelRatio, 0, 0, pixelRatio, 0, 0],
      });
      state.renderTasks.set(pageNumber, renderTask);
      await renderTask.promise.finally(() =>
        state.renderTasks.delete(pageNumber),
      );

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

      pageViews.set(pageNumber, { div: pageDiv, viewport });

      try {
        page.cleanup?.();
      } catch {}
    }

    if (!state.readTimesSetup) {
      state.readTimesSetup = true;
      setupReadTimes(container, state);
    }

    container.dataset.pdfjsWidth = String(
      Math.round(container.clientWidth || 0),
    );
    setupResizeObserver(container);
  } catch (error) {
    container.innerHTML = "Failed to load PDF.";
    console.error("Failed to render PDF", error);
  } finally {
    const state = getState(container);
    state.loadingTask = null;
    renderState.set(container, state);

    container.dataset.pdfjsRendering = "false";

    if (state.pending) {
      state.pending = false;
      renderState.set(container, state);
      scheduleRender(container);
    }
  }
}

function cancelRenderWork(state) {
  state.renderTasks?.forEach((t) => {
    try {
      t.cancel();
    } catch {}
  });
  state.renderTasks = new Map();

  state.visibilityObserver?.disconnect?.();
  state.visibilityObserver = null;
}

function scanAndRender() {
  document
    .querySelectorAll(CONTAINER_SELECTOR)
    .forEach((c) => scheduleRender(c));
}

function observeDom() {
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === "attributes") {
        const element = mutation.target;
        if (
          element instanceof HTMLElement &&
          element.matches(CONTAINER_SELECTOR)
        ) {
          scheduleRender(element);
        }
        continue;
      }

      for (const node of mutation.addedNodes) {
        if (!(node instanceof HTMLElement)) continue;

        if (node.matches?.(CONTAINER_SELECTOR)) {
          scheduleRender(node);
        } else {
          node
            .querySelectorAll?.(CONTAINER_SELECTOR)
            .forEach((c) => scheduleRender(c));
        }
      }

      for (const node of mutation.removedNodes) {
        if (!(node instanceof HTMLElement)) continue;
        if (node.matches?.(CONTAINER_SELECTOR)) {
          destroyContainer(node);
        } else {
          node.querySelectorAll?.(CONTAINER_SELECTOR).forEach(destroyContainer);
        }
      }
    }
  });

  observer.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ["data-pdf-src"],
  });
}

function setupReadTimes(container, state) {
  setupVisibility(state);

  const editionId = Number(container.dataset.editionId);

  let lastUpdate = Date.now();
  let lastSend = Date.now();
  let isSending = false;

  async function tick() {
    const now = Date.now();
    const updateElapsed = now - lastUpdate;
    lastUpdate = now; // update outside of focused-detection to skip any unfocused time

    // only actually increment/send read times if the tab is focused
    if (!document.hidden && document.hasFocus()) {
      updateReadTimes(updateElapsed, state);

      const sendElapsed = now - lastSend;
      if (sendElapsed > 5000 && !isSending) {
        isSending = true;

        try {
          await sendReadTimes(editionId, state);
          lastSend = now;
          clearReadTimes(state);
        } catch (error) {
          console.error("Failed to send read times:", error);
        } finally {
          isSending = false;
        }
      }
    }

    setTimeout(tick, 100);
  }

  tick();
}

function updateReadTimes(elapsed, state) {
  state.pages.forEach((page) => {
    const current = Number(page.dataset.time) || 0;
    page.dataset.time = current + elapsed * page.dataset.visibility;
  });
}

async function sendReadTimes(editionId, state) {
  const response = await fetch("/api/record-read-times", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      edition_id: editionId,
      page_times: state.pages.map((page) => Number(page.dataset.time)),
    }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`HTTP error: status: ${response.status}, body: ${text}`);
  }
}

function clearReadTimes(state) {
  state.pages.forEach((page) => (page.dataset.time = 0));
}

function setupVisibility(state) {
  state.visibilityObserver?.disconnect?.();

  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        entry.target.dataset.visibility = entry.intersectionRatio;
      });
    },
    {
      threshold: Array.from({ length: 101 }, (_, i) => i / 100),
    },
  );

  state.pages.forEach((page) => {
    page.dataset.visibility = 0;
    observer.observe(page);
  });

  state.visibilityObserver = observer;
}

function setup() {
  scanAndRender();
  observeDom();
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    setup();
  });
} else {
  setup();
}
