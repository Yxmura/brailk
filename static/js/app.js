let wasmReady = false;
let modHeld = false;

document.addEventListener('keydown', e => {
  if (e.ctrlKey || e.metaKey || e.altKey) modHeld = true;

  if (document.activeElement === input || e.ctrlKey || e.metaKey || e.altKey) return;
  const t = e.target;
  if (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable) return;
  if (e.key === 'Tab' || e.key === 'Escape' || e.key === 'F1' || e.key.startsWith('F')) return;
  if (e.key.length === 1 || e.key === 'Backspace' || e.key === 'Delete' || e.key === 'Enter') {
    input.focus();
  }
});
document.addEventListener('keyup', e => {
  if (e.key === 'Control' || e.key === 'Meta' || e.key === 'Alt') {
    setTimeout(() => { modHeld = false; }, 400);
  }
});

const $ = id => document.getElementById(id);
const input = $('input');
const preview = $('preview');
const code = $('code');

const loader = $('loader');
const themeBtn = $('theme-btn');
const copyBtn = $('copy-btn');
const errorBar = $('error-bar');
const errorMsg = $('error-msg');
const errorCtx = $('error-ctx');
const errorClose = $('error-close');
let timer = null;
let last = '';

init();

function init() {
  initTheme();
  if (!window.katex) {
    loader.classList.add('hidden');
    showError('KaTeX bibliotheek niet geladen', '');
    return;
  }
  initWasmConverter('static/vwc_converter.wasm')
    .then(() => {
      wasmReady = true;
      loader.classList.add('hidden');
      if (input.value.trim()) convert();
      if (!modHeld) input.focus();
    })
    .catch(e => {
      showError('Fout bij laden', e.message);
      loader.classList.add('hidden');
      if (!modHeld) input.focus();
    });

  input.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(convert, 300);
  });

  input.addEventListener('keydown', e => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      clearTimeout(timer);
      convert();
    }
  });

  const saved = localStorage.getItem('draft');
  if (saved) { input.value = saved; }

  themeBtn.addEventListener('click', toggleTheme);
  copyBtn.addEventListener('click', copyLatex);
  $('clear-btn').addEventListener('click', () => { input.value = ''; input.dispatchEvent(new Event('input')); input.focus(); });
  $('paste-btn').addEventListener('click', () => { navigator.clipboard.readText().then(t => { input.value = t; input.dispatchEvent(new Event('input')); input.focus(); }); });
  $('copy-input-btn').addEventListener('click', () => { navigator.clipboard.writeText(input.value).then(() => { $('copy-input-btn').style.color = 'var(--accent)'; setTimeout(() => { $('copy-input-btn').style.color = ''; }, 800); }); });
  errorClose.addEventListener('click', () => errorBar.hidden = true);
}

function initTheme() {
  const stored = localStorage.getItem('theme');
  if (stored) {
    document.documentElement.setAttribute('data-theme', stored);
  } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    document.documentElement.setAttribute('data-theme', 'dark');
  }
}

function toggleTheme() {
  const next = document.documentElement.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
  if (!document.startViewTransition) { setTheme(next); return; }

  document.startViewTransition(() => setTheme(next));
}

function setTheme(t) {
  document.documentElement.setAttribute('data-theme', t);
  localStorage.setItem('theme', t);
}

function convert() {
  const vwc = input.value;
  localStorage.setItem('draft', vwc);

  if (vwc === last) return;
  last = vwc;

  hideError();

  if (!vwc.trim() || !wasmReady) {
    preview.innerHTML = '<span class="empty-state">Je wiskunde verschijnt hier</span>';
    code.innerHTML = '<span class="code-placeholder">// LaTeX code</span>';
    copyBtn.classList.remove('visible');
    return;
  }

  const result = (() => {
    try { return callVwcConverter(vwc, false); }
    catch (e) { return { output: '$' + vwc + '$', errors: [{ type: 'Error', message: e.message }] }; }
  })();

  code.innerHTML = '';
  copyBtn.classList.add('visible');
  code.textContent = result.output;

  let raw = result.output;
  if (raw.startsWith('$$') && raw.endsWith('$$')) { raw = raw.slice(2, -2); }
  else if (raw.startsWith('$') && raw.endsWith('$')) { raw = raw.slice(1, -1); }

  try {
    const html = window.katex.renderToString(raw, { throwOnError: false, displayMode: true });
    preview.innerHTML = html;
  } catch (e) {
    preview.innerHTML = '<span style="color:var(--danger);font-size:14px">' + escapeHtml(e.message) + '</span>';
  }

  if (result.errors && result.errors.length) {
    const e = result.errors[0];
    showError(e.type + ': ' + e.message, e.context || '');
  }
}

function copyLatex() {
  const text = code.textContent;
  if (!text) return;
  navigator.clipboard.writeText(text).then(() => {
    copyBtn.style.color = 'var(--accent)';
    setTimeout(() => copyBtn.style.color = '', 800);
  });
}

function showError(msg, ctx) {
  errorMsg.textContent = msg;
  errorCtx.textContent = ctx || '';
  errorBar.hidden = false;
}

function hideError() {
  errorBar.hidden = true;
}

function escapeHtml(s) {
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}
