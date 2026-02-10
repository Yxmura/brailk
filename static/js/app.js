document.addEventListener("DOMContentLoaded", function () {
  const vwcInput = document.getElementById("vwc-input");
  const convertBtn = document.getElementById("convert-btn");
  const renderedOutput = document.getElementById("rendered-output");
  const typstOutput = document.getElementById("typst-output");
  const toast = document.getElementById("toast");

  const clearBtn = document.getElementById("clear-btn");
  const pasteBtn = document.getElementById("paste-btn");
  const downloadBtn = document.getElementById("download-btn");

  const tabs = document.querySelectorAll(".tab");
  const panels = document.querySelectorAll(".tab-panel");

  init();

  function init() {
    setupTabs();
    setupKeyboardShortcuts();
    setupButtons();
  }

  function setupTabs() {
    tabs.forEach((tab) => {
      tab.addEventListener("click", () => {
        const target = tab.dataset.tab;
        tabs.forEach((t) => t.classList.remove("active"));
        panels.forEach((p) => p.classList.remove("active"));
        tab.classList.add("active");
        document
          .querySelector(`[data-panel="${target}"]`)
          .classList.add("active");
      });
    });
  }

  function setupKeyboardShortcuts() {
    vwcInput.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        performConversion();
      }
    });
  }

  function setupButtons() {
    convertBtn.addEventListener("click", performConversion);

    clearBtn.addEventListener("click", () => {
      vwcInput.value = "";
      vwcInput.focus();
      resetOutput();
    });

    pasteBtn.addEventListener("click", async () => {
      try {
        const text = await navigator.clipboard.readText();
        vwcInput.value = text;
      } catch {}
    });

    downloadBtn.addEventListener("click", downloadPDF);
  }

  async function performConversion() {
    const vwc = vwcInput.value.trim();
    if (!vwc) return;

    try {
      const response = await fetch("/convert-and-render", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ vwc: vwc, display: false }),
      });

      const data = await response.json();

      if (data.error) {
        showError(data.error);
      } else {
        showResult(data);
      }
    } catch (err) {
      showError("Failed: " + err.message);
    }
  }

  function resetOutput() {
    renderedOutput.innerHTML =
      '<div class="empty"><p>Your rendered math will appear here</p></div>';
    typstOutput.innerHTML =
      '<code class="placeholder language-typst">// Typst code will appear here</code>';
  }

  function showResult(data) {
    typstOutput.innerHTML = `<code class="language-typst">${escapeHtml(data.typst)}</code>`;

    if (data.image) {
      renderedOutput.innerHTML = `
                <div class="rendered-content">
                    <div class="svg-container">${data.image}</div>
                </div>
            `;
    }
    Prism.highlightElement(document.querySelector("#typst-output code"));
  }

  function showError(msg) {
    renderedOutput.innerHTML = `<div class="empty error-state"><p>${escapeHtml(msg)}</p></div>`;
  }

  async function downloadPDF() {
    const vwc = vwcInput.value.trim();
    if (!vwc) return;

    try {
      const response = await fetch("/download-pdf", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ vwc: vwc, display: false }),
      });

      if (response.ok) {
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `brailk-${Date.now()}.pdf`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
        showToast("PDF downloaded");
      }
    } catch {}
  }

  function showToast(msg) {
    toast.textContent = msg;
    toast.classList.add("show");
    setTimeout(() => toast.classList.remove("show"), 2000);
  }

  function escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }
});
