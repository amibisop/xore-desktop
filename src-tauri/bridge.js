/* xore desktop shell bridge — injected before any page script runs.
   The site needs no changes: it already calls navigator.setAppBadge(), and we
   hijack that to drive the tray, the taskbar title and native toasts. */
(function () {
  'use strict';

  window.__XORE_DESKTOP__ = true;

  function invoke(cmd, args) {
    try {
      var t = window.__TAURI__;
      if (t && t.core && typeof t.core.invoke === 'function') return t.core.invoke(cmd, args || {});
      var i = window.__TAURI_INTERNALS__;
      if (i && typeof i.invoke === 'function') return i.invoke(cmd, args || {});
    } catch (e) { /* not in the shell */ }
    return Promise.resolve();
  }
  window.__xoreInvoke = invoke;

  /* ---- unread count -> tray tooltip + window title + toast on increase ---- */
  var last = 0;
  function push(n) {
    n = Math.max(0, n | 0);
    var grew = n > last;
    last = n;
    invoke('xore_badge', { count: n, notify: grew });
  }
  navigator.setAppBadge = function (n) { push(n || 0); return Promise.resolve(); };
  navigator.clearAppBadge = function () { push(0); return Promise.resolve(); };

  /* ---- let the site style itself for desktop if it ever wants to ---- */
  function mark() { try { document.documentElement.classList.add('xore-desktop'); } catch (e) { /* */ } }
  if (document.documentElement) mark();
  document.addEventListener('DOMContentLoaded', mark);

  /* ---- links to other sites open in the real browser, not inside the app ---- */
  function external(href) {
    if (!/^https?:/i.test(href)) return false;
    try { return new URL(href, location.href).origin !== location.origin; } catch (e) { return false; }
  }
  document.addEventListener('click', function (e) {
    var a = e.target && e.target.closest && e.target.closest('a[href]');
    if (!a) return;
    var href = a.href || '';
    if (!external(href)) return;
    e.preventDefault();
    e.stopPropagation();
    invoke('xore_open_external', { url: new URL(href, location.href).href });
  }, true);

  var open0 = window.open;
  window.open = function (url) {
    if (url && external(url)) {
      invoke('xore_open_external', { url: new URL(url, location.href).href });
      return null;
    }
    return open0 ? open0.apply(window, arguments) : null;
  };
})();
