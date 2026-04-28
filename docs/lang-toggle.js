// Global language toggle. Injects a Rust / Python / TypeScript segmented
// control into Mintlify's navbar (right of the logo), persists the choice
// in localStorage, and broadcasts it to every CodeGroup on the page by
// clicking the matching tab — Mintlify already syncs its own state across
// matching CodeGroup labels, so one click per render is enough.
//
// Reapplies the saved choice whenever the DOM changes (SPA navigation,
// React re-renders) so newly-mounted CodeGroups inherit the user's pick.

(function () {
  var LANGS = [
    { id: "rust", label: "Rust" },
    { id: "python", label: "Python" },
    { id: "typescript", label: "TypeScript" },
  ];
  var STORAGE_KEY = "openpx-preferred-lang";

  function getLang() {
    try {
      var v = localStorage.getItem(STORAGE_KEY);
      if (v && LANGS.some(function (l) { return l.id === v; })) return v;
    } catch (_) {}
    return "rust";
  }

  function setLang(lang) {
    try { localStorage.setItem(STORAGE_KEY, lang); } catch (_) {}
    syncToggleState(lang);
    applyLangToCodeGroups(lang);
  }

  // Find the visible language-tab button in every CodeGroup on the page
  // whose label matches the requested language, and click it. Mintlify's
  // CodeGroup is React-controlled, so we rely on the real click handler
  // (which updates Mintlify's own synced state) rather than mutating DOM.
  function applyLangToCodeGroups(lang) {
    var target = LANGS.find(function (l) { return l.id === lang; });
    if (!target) return;
    var label = target.label;
    var seen = new WeakSet();
    var tabs = document.querySelectorAll('button, [role="tab"]');
    for (var i = 0; i < tabs.length; i++) {
      var t = tabs[i];
      if (t.closest && t.closest("#openpx-lang-toggle")) continue;
      var text = (t.textContent || "").trim();
      if (text !== label) continue;
      // Only click tabs that aren't already the active one — prevents
      // an infinite click loop with the MutationObserver.
      var pressed = t.getAttribute("aria-selected") === "true" ||
                    t.getAttribute("data-state") === "active" ||
                    t.classList.contains("active");
      if (pressed) continue;
      if (seen.has(t)) continue;
      seen.add(t);
      try { t.click(); } catch (_) {}
    }
  }

  function syncToggleState(lang) {
    var btns = document.querySelectorAll("#openpx-lang-toggle button");
    for (var i = 0; i < btns.length; i++) {
      btns[i].setAttribute(
        "aria-pressed",
        btns[i].dataset.lang === lang ? "true" : "false"
      );
    }
  }

  function buildToggle() {
    var wrap = document.createElement("div");
    wrap.id = "openpx-lang-toggle";
    wrap.setAttribute("role", "group");
    wrap.setAttribute("aria-label", "Preferred language");
    LANGS.forEach(function (l) {
      var b = document.createElement("button");
      b.type = "button";
      b.dataset.lang = l.id;
      b.textContent = l.label;
      b.addEventListener("click", function () { setLang(l.id); });
      wrap.appendChild(b);
    });
    return wrap;
  }

  // Find a stable insertion point near the logo. Mintlify's logo lives
  // inside an <a> tag at the start of the header; insert the toggle as
  // its next sibling.
  function insertionPoint() {
    var header = document.querySelector("header") || document.querySelector('[role="banner"]');
    if (!header) return null;
    // Prefer a logo anchor; fall back to the header itself.
    var logoAnchor = header.querySelector('a[href="/"], a[aria-label*="home" i], a[aria-label*="logo" i]');
    return logoAnchor || header.firstElementChild || header;
  }

  function injectToggle() {
    if (document.getElementById("openpx-lang-toggle")) return;
    var anchor = insertionPoint();
    if (!anchor) return;
    var toggle = buildToggle();
    if (anchor.parentNode && anchor.nextSibling) {
      anchor.parentNode.insertBefore(toggle, anchor.nextSibling);
    } else if (anchor.parentNode) {
      anchor.parentNode.appendChild(toggle);
    } else {
      anchor.appendChild(toggle);
    }
    syncToggleState(getLang());
  }

  function tick() {
    injectToggle();
    applyLangToCodeGroups(getLang());
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tick);
  } else {
    tick();
  }

  // Debounced observer — Mintlify is a SPA, so re-run on navigation /
  // re-render. requestAnimationFrame keeps us off the critical path.
  var pending = false;
  var obs = new MutationObserver(function () {
    if (pending) return;
    pending = true;
    requestAnimationFrame(function () {
      pending = false;
      tick();
    });
  });
  obs.observe(document.body, { childList: true, subtree: true });
})();
