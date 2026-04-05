document.addEventListener("DOMContentLoaded", function () {
  const vwcInput = document.getElementById("vwc-input");
  const vwcHighlight = document.getElementById("vwc-highlight");
  const inputHint = document.getElementById("input-hint");
  const convertBtn = document.getElementById("convert-btn");
  const renderedOutput = document.getElementById("rendered-output");
  const typstOutput = document.getElementById("typst-output");
  const toast = document.getElementById("toast");

  const clearBtn = document.getElementById("clear-btn");
  const pasteBtn = document.getElementById("paste-btn");
  const downloadBtn = document.getElementById("download-btn");
  const themeToggle = document.getElementById("theme-toggle");

  const tabs = document.querySelectorAll(".tab");
  const panels = document.querySelectorAll(".tab-panel");

  // Debounce timer for auto-convert
  let debounceTimer = null;
  const DEBOUNCE_DELAY = 500; // 500ms

  // Track conversion state
  let isConverting = false;
  let lastConvertedInput = "";

  init();

  function init() {
    document.getElementById("current-year").textContent =
      new Date().getFullYear();
    setupTabs();
    setupKeyboardShortcuts();
    setupButtons();
    setupThemeToggle();
    setupGlobalKeyFocus();
    setupSyntaxHighlighting();
    setupAutoConvert();
    setupAutoSave();
    loadAutoSave();
  }

  function setupGlobalKeyFocus() {
    document.addEventListener("keydown", (e) => {
      const activeEl = document.activeElement;
      const isEditable =
        activeEl &&
        (activeEl.tagName === "INPUT" ||
          activeEl.tagName === "TEXTAREA" ||
          activeEl.isContentEditable);

      if (isEditable) return;

      if (e.ctrlKey || e.altKey || e.metaKey || e.key.length !== 1) return;

      e.preventDefault();
      vwcInput.focus();
      vwcInput.setRangeText(
        e.key,
        vwcInput.selectionStart,
        vwcInput.selectionEnd,
        "end",
      );
      vwcInput.dispatchEvent(new Event("input"));
    });
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

  function setupThemeToggle() {
    if (!themeToggle) return;

    const savedTheme = localStorage.getItem("brailk_theme");
    if (savedTheme === "dark" || savedTheme === "light") {
      applyTheme(savedTheme, false);
    } else {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      applyTheme(prefersDark ? "dark" : "light", false);
    }

    themeToggle.addEventListener("click", () => {
      const nextTheme = document.body.classList.contains("theme-dark")
        ? "light"
        : "dark";
      applyTheme(nextTheme, true);
    });
  }

  function applyTheme(theme, showNotification) {
    const isDark = theme === "dark";
    document.body.classList.toggle("theme-dark", isDark);

    if (themeToggle) {
      themeToggle.setAttribute("aria-pressed", isDark ? "true" : "false");
      themeToggle.textContent = isDark ? "Licht" : "Donker";
      themeToggle.setAttribute(
        "aria-label",
        isDark ? "Licht thema inschakelen" : "Donker thema inschakelen",
      );
    }

    localStorage.setItem("brailk_theme", theme);

    if (showNotification) {
      showToast(isDark ? "Donker thema actief" : "Licht thema actief");
    }
  }

  function setupSyntaxHighlighting() {
    const syncHighlightScroll = () => {
      vwcHighlight.scrollTop = vwcInput.scrollTop;
      vwcHighlight.scrollLeft = vwcInput.scrollLeft;
    };

    vwcInput.addEventListener("input", updateInputHighlight);
    vwcInput.addEventListener("scroll", syncHighlightScroll);

    updateInputHighlight();
  }

  function updateInputHighlight() {
    const input = vwcInput.value;
    vwcHighlight.innerHTML = highlightVwcInput(input);
  }

  function highlightVwcInput(input) {
    if (!input) {
      return "";
    }

    let highlighted = "";
    let i = 0;

    while (i < input.length) {
      if (input.startsWith("$wt", i)) {
        highlighted += wrapSyntaxToken("vwc-token-wt", "$wt");
        i += 3;
        continue;
      }

      if (input.startsWith("$br", i)) {
        highlighted += wrapSyntaxToken("vwc-token-br", "$br");
        i += 3;
        continue;
      }

      const char = input[i];

      if (char === "#") {
        highlighted += wrapSyntaxToken("vwc-token-hash", char);
      } else if (char === "^") {
        highlighted += wrapSyntaxToken("vwc-token-sup", char);
      } else if (char === "_") {
        highlighted += wrapSyntaxToken("vwc-token-sub", char);
      } else if (char === "μ" || char === "µ") {
        highlighted += wrapSyntaxToken("vwc-token-mu", char);
      } else {
        highlighted += escapeHtml(char);
      }

      i += 1;
    }

    return highlighted;
  }

  function wrapSyntaxToken(className, token) {
    return `<span class="${className}">${escapeHtml(token)}</span>`;
  }

  function setupAutoConvert() {
    // Debounced auto-convert on input
    vwcInput.addEventListener("input", () => {
      // Clear existing timer
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }

      const vwc = vwcInput.value.trim();

      // Validate basic syntax
      validateInput(vwc);

      // Debounce the conversion
      if (vwc.length > 0) {
        debounceTimer = setTimeout(() => {
          // Only convert if input changed
          if (vwc !== lastConvertedInput) {
            performConversion();
          }
        }, DEBOUNCE_DELAY);
      }
    });
  }

  function validateInput(vwc) {
    // Clear any previous states
    vwcInput.classList.remove("input-error", "input-warning");
    setInputHint("");

    if (!vwc) return;

    // Soft validation - only show hints for really obvious issues
    // The converter will try to handle most things
    
    // Check for obvious broken patterns
    const dollarMatches = vwc.match(/\$/g) || [];
    if (dollarMatches.length >= 5) {
      setInputHint("Controleer $ symbolen");
      vwcInput.classList.add("input-warning");
      return;
    }

    // Check for consecutive structural opens (e.g., $br$wt without #)
    const structuralOpens = vwc.match(/\$(br|wt|mat|det|com|st)/g) || [];
    const closingHashes = (vwc.match(/#/g) || []).length;
    
    if (structuralOpens.length >= 3 && closingHashes < structuralOpens.length) {
      setInputHint("Mogelijk # vergeten?");
      vwcInput.classList.add("input-warning");
    }
  }

  function setInputHint(message) {
    if (!inputHint) return;

    if (!message) {
      inputHint.textContent = "";
      inputHint.classList.remove("show");
      return;
    }

    inputHint.textContent = message;
    inputHint.classList.add("show");
  }

  function setupAutoSave() {
    // Save to localStorage on input
    vwcInput.addEventListener("input", () => {
      localStorage.setItem("brailk_draft", vwcInput.value);
      localStorage.setItem("brailk_timestamp", Date.now());
    });
  }

  function loadAutoSave() {
    const saved = localStorage.getItem("brailk_draft");
    const timestamp = localStorage.getItem("brailk_timestamp");

    if (saved && saved.trim()) {
      // Check if saved within last 24 hours
      const hoursSinceSave =
        (Date.now() - parseInt(timestamp)) / (1000 * 60 * 60);

      if (hoursSinceSave < 24) {
        vwcInput.value = saved;
        updateInputHighlight();
        validateInput(saved.trim());
        showToast("Concept hersteld");
        // Auto-convert the restored draft
        setTimeout(() => performConversion(), 100);
      }
    }
  }

  function setupButtons() {
    convertBtn.addEventListener("click", () => {
      // Cancel debounce and convert immediately
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }
      performConversion();
    });

    clearBtn.addEventListener("click", () => {
      vwcInput.value = "";
      updateInputHighlight();
      validateInput("");
      vwcInput.focus();
      resetOutput();
      localStorage.removeItem("brailk_draft");
      lastConvertedInput = "";
    });

    pasteBtn.addEventListener("click", async () => {
      try {
        const text = await navigator.clipboard.readText();
        vwcInput.value = text;
        vwcInput.dispatchEvent(new Event("input"));
        showToast("Geplakt van klembord");
      } catch {
        showToast("Kan klembord niet benaderen");
      }
    });

    downloadBtn.addEventListener("click", downloadPDF);
  }

  function sanitizeInput(input) {
    // Minimal sanitization - only remove truly problematic chars for Typst
    // Allow more flexibility to match "accepts a lot more" behavior
    const problematicChars = /\\/g;
    const sanitized = input.replace(problematicChars, "");

    // Only show toast if backslashes were removed (more impactful)
    const removedCount = input.length - sanitized.length;
    if (removedCount > 0) {
      showToast("Backslash verwijderd");
    }

    return sanitized;
  }

  async function performConversion() {
    let vwc = vwcInput.value.trim();
    if (!vwc) {
      resetOutput();
      return;
    }

    // Sanitize input voor Typst
    const originalVwc = vwc;
    vwc = sanitizeInput(vwc);

    // Update textarea als er iets is veranderd
    if (vwc !== originalVwc) {
      vwcInput.value = vwc;
      updateInputHighlight();
      validateInput(vwc);
    }

    // Prevent duplicate conversions
    if (isConverting) return;
    isConverting = true;
    lastConvertedInput = vwc;

    // Show loading state
    showLoading();
    convertBtn.disabled = true;

    try {
      const response = await fetch("/convert-and-render", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ vwc: vwc, display: false }),
      });

      const data = await response.json();

      if (data.error) {
        showDetailedError(data.error, data.traceback, data.suggestion);
      } else {
        showResult(data);
        // Clear any error states on success
        vwcInput.classList.remove("input-error");
      }
    } catch (err) {
      showDetailedError("Network error: " + err.message, null);
    } finally {
      isConverting = false;
      convertBtn.disabled = false;
    }
  }

  function showLoading() {
    renderedOutput.innerHTML = `
      <div class="empty loading-state">
        <div class="spinner"></div>
        <p>Bezig met omzetten...</p>
      </div>
    `;
  }

  function resetOutput() {
    renderedOutput.innerHTML =
      '<div class="empty"><p>Jouw wiskunde verschijnt hier</p></div>';
    typstOutput.innerHTML =
      '<code class="placeholder language-typst">// Typst code verschijnt hier</code>';
  }

  function showResult(data) {
    typstOutput.innerHTML = `<code class="language-typst">${escapeHtml(data.typst)}</code>`;

    if (data.image) {
      renderedOutput.innerHTML = `
        <div class="rendered-content">
          <div class="svg-container">${data.image}</div>
          <div class="image-actions">
            <button type="button" class="btn-icon" id="copy-image-btn" title="Afbeelding kopiëren" aria-label="Afbeelding kopiëren">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
                <rect x="4" y="4" width="8" height="10" rx="1" stroke="currentColor" stroke-width="1.5"/>
                <path d="M4 6H12" stroke="currentColor" stroke-width="1.5"/>
                <rect x="6" y="2" width="4" height="2" rx="0.5" stroke="currentColor" stroke-width="1.5"/>
              </svg>
              Kopiëren
            </button>
          </div>
        </div>
      `;

      // Add copy image handler
      document
        .getElementById("copy-image-btn")
        .addEventListener("click", () => {
          copyImageToClipboard(data.image);
        });
    }

    if (window.Prism) {
      Prism.highlightElement(document.querySelector("#typst-output code"));
    }
  }

  async function copyImageToClipboard(svgContent) {
    try {
      // Convert SVG to PNG for clipboard
      const vwc = vwcInput.value.trim();
      const response = await fetch("/export-png", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ vwc: vwc, display: false, ppi: 288 }),
      });

      const data = await response.json();

      if (data.error) {
        showToast("Error: " + data.error);
        return;
      }

      // Convert base64 to blob
      const byteCharacters = atob(data.png);
      const byteNumbers = new Array(byteCharacters.length);
      for (let i = 0; i < byteCharacters.length; i++) {
        byteNumbers[i] = byteCharacters.charCodeAt(i);
      }
      const byteArray = new Uint8Array(byteNumbers);
      const blob = new Blob([byteArray], { type: "image/png" });

      // Copy to clipboard
      if (navigator.clipboard && navigator.clipboard.write) {
        await navigator.clipboard.write([
          new ClipboardItem({ "image/png": blob }),
        ]);
        showToast("Afbeelding gekopieerd!");
      } else {
        // Fallback: download
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `wiskunde-${Date.now()}.png`;
        a.click();
        URL.revokeObjectURL(url);
        showToast("Afbeelding gedownload");
      }
    } catch (err) {
      showToast("Kopiëren mislukt: " + err.message);
    }
  }

  function showDetailedError(msg, traceback, backendSuggestion) {
    // Use backend suggestion if available (from VWC validation)
    if (backendSuggestion) {
      renderedOutput.innerHTML = `
        <div class="error-container">
          <div class="error-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
          </div>
          <div class="error-content">
            <h3 class="error-title">${escapeHtml(msg)}</h3>
            <p class="error-suggestion">${escapeHtml(backendSuggestion)}</p>
          </div>
          ${traceback ? `<details class="error-details"><summary>Technische details (voor ontwikkelaars)</summary><pre>${escapeHtml(traceback)}</pre></details>` : ""}
        </div>
      `;
      vwcInput.classList.add("input-error");
      const clearError = () => {
        vwcInput.classList.remove("input-error");
        vwcInput.removeEventListener("input", clearError);
      };
      vwcInput.addEventListener("input", clearError);
      return;
    }

    // Fallback for old-style errors
    let userMessage = msg;
    let errorTitle = "Er is iets misgegaan";
    let errorSuggestion = "";

    if (
      msg.includes("EOF") ||
      msg.includes("unclosed") ||
      msg.includes("delimiter")
    ) {
      errorTitle = "Niet afgesloten haakjes of #";
      userMessage = "Je bent een haakje of # vergeten af te sluiten.";
      errorSuggestion =
        "💡 Controleer of elk '(' een ')' heeft en elke '#' een tweede '#' heeft om te sluiten.";
    } else if (msg.includes("bracket") || msg.includes("parenthesis")) {
      errorTitle = "Haakjes kloppen niet";
      userMessage = "Je haakjes zijn niet in evenwicht.";
      errorSuggestion = "💡 Tel het aantal '(' en ')' - dit moet gelijk zijn.";
    } else if (msg.includes("index") || msg.includes("range")) {
      errorTitle = "Fout in VWC code";
      userMessage = "Je VWC notatie bevat een fout.";
      errorSuggestion =
        "💡 Controleer of je alle codes correct hebt geschreven (bijv. $br voor breuken).";
    } else if (msg.includes("TypstError") || msg.includes("compile")) {
      errorTitle = "Kan niet renderen";
      userMessage = "De VWC code kon niet worden omgezet naar wiskunde.";
      errorSuggestion =
        "💡 Probeer het eenvoudiger te maken of controleer je notatie.";
    }

    renderedOutput.innerHTML = `
      <div class="error-container">
        <div class="error-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
        </div>
        <div class="error-content">
          <h3 class="error-title">${escapeHtml(errorTitle)}</h3>
          <p class="error-message">${escapeHtml(userMessage)}</p>
          ${errorSuggestion ? `<p class="error-suggestion">${escapeHtml(errorSuggestion)}</p>` : ""}
        </div>
        ${traceback ? `<details class="error-details"><summary>Technische details (voor nerds)</summary><pre>${escapeHtml(traceback)}</pre></details>` : ""}
      </div>
    `;

    // Add error indicator to input
    vwcInput.classList.add("input-error");

    // Clear error on input
    const clearError = () => {
      vwcInput.classList.remove("input-error");
      vwcInput.removeEventListener("input", clearError);
    };
    vwcInput.addEventListener("input", clearError);
  }

  async function downloadPDF() {
    const vwc = vwcInput.value.trim();
    if (!vwc) {
      showToast("Voer eerst VWC code in");
      return;
    }

    convertBtn.disabled = true;
    showToast("PDF wordt gegenereerd...");

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
        showToast("PDF gedownload!");
      } else {
        const data = await response.json();
        showToast("Fout: " + (data.error || "PDF genereren mislukt"));
      }
    } catch (err) {
      showToast("Download mislukt: " + err.message);
    } finally {
      convertBtn.disabled = false;
    }
  }

  function showToast(msg) {
    toast.textContent = msg;
    toast.classList.add("show");
    setTimeout(() => toast.classList.remove("show"), 6000);
  }

  function escapeHtml(text) {
    const div = document.createElement("div");
    div.textContent = text;
    return div.innerHTML;
  }
});
