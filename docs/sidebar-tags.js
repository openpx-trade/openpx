// Color-coded sidebar tags. Mintlify renders frontmatter `tag:` as a span
// somewhere inside the sidebar link, but exposes no class hook for its
// content. This script walks the sidebar and, for any element whose text
// content is exactly one of our verb tags (GET / POST / DELETE / WSS),
// stamps a data-openpx-tag attribute that style.css can target.

(function () {
  var TAGS = { GET: 1, POST: 1, DELETE: 1, WSS: 1 };

  function tagSidebar() {
    var nav = document.querySelector('nav, aside, #sidebar, [aria-label*="navigation" i]');
    var root = nav || document.body;
    var spans = root.querySelectorAll('span, div');
    for (var i = 0; i < spans.length; i++) {
      var el = spans[i];
      if (el.dataset.openpxTag) continue;
      if (el.children.length) continue;
      var text = (el.textContent || "").trim();
      if (TAGS[text]) el.dataset.openpxTag = text;
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tagSidebar);
  } else {
    tagSidebar();
  }

  // Mintlify is a SPA — re-tag whenever the DOM changes. Debounced via
  // requestAnimationFrame so we don't churn during large updates.
  var pending = false;
  var observer = new MutationObserver(function () {
    if (pending) return;
    pending = true;
    requestAnimationFrame(function () {
      pending = false;
      tagSidebar();
    });
  });
  observer.observe(document.body, { childList: true, subtree: true });
})();
