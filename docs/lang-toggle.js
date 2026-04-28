// Global language toggle. Injects a Rust / Python / TypeScript segmented
// control immediately to the right of the OpenPX logo, persists the
// choice in localStorage, and switches every CodeGroup on the page to
// the chosen language by programmatically clicking the matching tab —
// Mintlify's CodeGroup state syncs across matching labels, so one click
// per render is enough.

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
    applyLang(lang);
  }

  function applyLang(lang) {
    var target = LANGS.find(function (l) { return l.id === lang; });
    if (!target) return;
    var label = target.label;

    // Cast a wide net — Mintlify renders CodeGroup tabs as buttons or
    // role=tab elements, sometimes with the language label nested inside
    // a span. We match by trimmed text and skip our own toggle.
    var candidates = document.querySelectorAll(
      'button, [role="tab"], [role="button"]'
    );
    for (var i = 0; i < candidates.length; i++) {
      var el = candidates[i];
      if (el.closest && el.closest("#openpx-lang-toggle")) continue;
      var text = (el.textContent || "").trim();
      if (text !== label) continue;

      // Fire both a synthesized MouseEvent (so React's synthetic-event
      // system picks it up) and a plain click() (for any non-React
      // listeners). Skip if already active to avoid redundant work, but
      // be permissive about how "active" is encoded.
      var pressed =
        el.getAttribute("aria-selected") === "true" ||
        el.getAttribute("data-state") === "active" ||
        el.dataset.active === "true" ||
        el.classList.contains("active") ||
        el.classList.contains("selected");
      if (pressed) continue;

      try {
        el.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, view: window }));
      } catch (_) {
        try { el.click(); } catch (__) {}
      }
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
      b.addEventListener("click", function (e) {
        e.preventDefault();
        e.stopPropagation();
        setLang(l.id);
      });
      wrap.appendChild(b);
    });
    return wrap;
  }

  // Anchor: the logo image. docs.json declares /images/logo-light.png and
  // /images/logo-dark.png. Match either, walk up to the wrapping anchor,
  // and insert the toggle as the anchor's next sibling so it sits to the
  // logo's right inside the same flex row.
  function findLogoAnchor() {
    var img =
      document.querySelector('img[src*="/images/logo-"]') ||
      document.querySelector('img[alt*="OpenPX" i]');
    if (!img) return null;
    return img.closest("a") || img.parentElement;
  }

  function injectToggle() {
    var existing = document.getElementById("openpx-lang-toggle");
    var anchor = findLogoAnchor();
    if (!anchor) return;
    // If already in DOM but not adjacent to the logo (e.g. landed in
    // the wrong slot), move it.
    if (existing && existing.previousElementSibling !== anchor) {
      existing.remove();
      existing = null;
    }
    if (existing) return;
    var toggle = buildToggle();
    if (anchor.parentNode) {
      anchor.parentNode.insertBefore(toggle, anchor.nextSibling);
    }
    syncToggleState(getLang());
  }

  // Apply the saved choice once on initial load. Retry briefly because
  // Mintlify's CodeGroup tabs may render after our first tick.
  var initialApplied = false;
  function applyOnce() {
    if (initialApplied) return;
    var tabsExist = !!document.querySelector('button, [role="tab"]');
    if (!tabsExist) return;
    applyLang(getLang());
    initialApplied = true;
  }

  function tick() {
    injectToggle();
    syncToggleState(getLang());
    applyOnce();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tick);
  } else {
    tick();
  }

  // Re-run on every DOM mutation, but coalesce per frame. We deliberately
  // do NOT re-apply the language on every tick — that would fight any
  // user who clicks a CodeGroup tab directly. The user's choice is only
  // re-broadcast when they click our toggle (or on initial page load).
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

  // Reset the "applied once" flag on SPA navigations so the saved
  // preference re-applies on the new page's CodeGroups.
  window.addEventListener("popstate", function () { initialApplied = false; });
  var origPush = history.pushState;
  history.pushState = function () {
    initialApplied = false;
    return origPush.apply(this, arguments);
  };
})();
