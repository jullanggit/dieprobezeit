(() => {
  const PDFJS_CDN_BASE =
    "https://cdnjs.cloudflare.com/ajax/libs/pdf.js/5.4.149"; // keep in sync with versions in main.rs
  const CONTAINER_SELECTOR = ".pdfjs-container[data-pdf-src]";
  const renderState = new WeakMap();

  function getPdfjs() {
    if (!window.pdfjsLib || !window.pdfjsViewer) {
      return null;
    }

    if (window.pdfjsLib.GlobalWorkerOptions.workerSrc !== undefined) {
      window.pdfjsLib.GlobalWorkerOptions.workerSrc =
        `${PDFJS_CDN_BASE}/pdf.worker.min.js`;
    }

    return {
      pdfjsLib: window.pdfjsLib,
      pdfjsViewer: window.pdfjsViewer,
    };
  }

  function scheduleRender(container) {
    const state = renderState.get(container) || {};
    if (state.timer) {
      clearTimeout(state.timer);
    }
    state.timer = setTimeout(() => {
      renderPdf(container);
    }, 60);
    renderState.set(container, state);
  }

  function setupResizeObserver(container) {
    if (container._pdfResizeObserver) {
      return;
    }

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
  }

  async function renderPdf(container) {
    const pdfjs = getPdfjs();
    if (!pdfjs) {
      return;
    }

    const pdfUrl = container.dataset.pdfSrc;
    if (!pdfUrl) {
      return;
    }

    if (container.dataset.pdfjsRendering === "true") {
      return;
    }

    container.dataset.pdfjsRendering = "true";
    container.innerHTML = "Loading PDF...";

    const { pdfjsLib, pdfjsViewer } = pdfjs;
    const loadingTask = pdfjsLib.getDocument(pdfUrl);

    try {
      const pdf = await loadingTask.promise;
      const linkService = new pdfjsViewer.PDFLinkService({
        externalLinkTarget: pdfjsViewer.LinkTarget.BLANK,
        externalLinkRel: "noopener noreferrer",
      });
      linkService.setDocument(pdf);

      const pageViews = new Map();
      container.innerHTML = "";

      const viewer = {
        scrollPageIntoView({ pageNumber, destArray }) {
          const pageView = pageViews.get(pageNumber);
          if (!pageView) {
            return;
          }

          if (!destArray) {
            pageView.div.scrollIntoView({ behavior: "smooth", block: "start" });
            return;
          }

          const destName = destArray[1] && destArray[1].name;
          let top = null;
          if (destName === "XYZ") {
            top = destArray[3];
          } else if (destName === "FitH" || destName === "FitBH") {
            top = destArray[2];
          }

          let yOffset = 0;
          if (top !== null && top !== undefined) {
            const [, y] = pageView.viewport.convertToViewportPoint(0, top);
            yOffset = y;
          }

          const pageTop =
            pageView.div.getBoundingClientRect().top + window.scrollY;
          window.scrollTo({
            top: pageTop + yOffset,
            behavior: "smooth",
          });
        },
      };

      linkService.setViewer(viewer);

      for (let pageNumber = 1; pageNumber <= pdf.numPages; pageNumber += 1) {
        const page = await pdf.getPage(pageNumber);
        const baseViewport = page.getViewport({ scale: 1 });
        const availableWidth =
          container.clientWidth ||
          (container.parentElement && container.parentElement.clientWidth) ||
          baseViewport.width;
        const scale = availableWidth / baseViewport.width;
        const viewport = page.getViewport({ scale });

        const pageDiv = document.createElement("div");
        pageDiv.className = "pdf-page";
        pageDiv.dataset.pageNumber = String(pageNumber);
        pageDiv.style.width = `${Math.ceil(viewport.width)}px`;
        pageDiv.style.height = `${Math.ceil(viewport.height)}px`;

        const canvas = document.createElement("canvas");
        canvas.className = "pdf-canvas";

        const outputScale = window.devicePixelRatio || 1;
        const canvasWidth = Math.floor(viewport.width * outputScale);
        const canvasHeight = Math.floor(viewport.height * outputScale);
        canvas.width = canvasWidth;
        canvas.height = canvasHeight;
        canvas.style.width = `${Math.floor(viewport.width)}px`;
        canvas.style.height = `${Math.floor(viewport.height)}px`;

        const context = canvas.getContext("2d", { alpha: false });
        const renderContext = {
          canvasContext: context,
          viewport,
          transform:
            outputScale !== 1
              ? [outputScale, 0, 0, outputScale, 0, 0]
              : null,
        };

        pageDiv.appendChild(canvas);

        const annotationLayer = document.createElement("div");
        annotationLayer.className = "annotationLayer";
        pageDiv.appendChild(annotationLayer);

        container.appendChild(pageDiv);
        await page.render(renderContext).promise;

        const annotations = await page.getAnnotations({ intent: "display" });
        pdfjsViewer.AnnotationLayer.render({
          annotations,
          div: annotationLayer,
          page,
          viewport,
          linkService,
          renderForms: false,
        });

        pageViews.set(pageNumber, { div: pageDiv, viewport });
      }

      container.dataset.pdfjsWidth = String(
        Math.round(container.clientWidth || 0),
      );
      setupResizeObserver(container);
    } catch (error) {
      container.innerHTML = "Failed to load PDF.";
      console.error("Failed to render PDF", error);
    } finally {
      container.dataset.pdfjsRendering = "false";
    }
  }

  function scanAndRender() {
    const containers = document.querySelectorAll(CONTAINER_SELECTOR);
    containers.forEach((container) => scheduleRender(container));
  }

  function observeDom() {
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of mutation.addedNodes) {
          if (!(node instanceof HTMLElement)) {
            continue;
          }

          if (node.matches && node.matches(CONTAINER_SELECTOR)) {
            scheduleRender(node);
          } else if (node.querySelectorAll) {
            node
              .querySelectorAll(CONTAINER_SELECTOR)
              .forEach((container) => scheduleRender(container));
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
})();
