// Sandboxed mail body frame. The frame runs without allow-same-origin, so the
// only channel back to the app is postMessage. Everything the frame may run is
// gated on this per-load nonce: email HTML is DOMPurify'd (no scripts survive)
// and the CSP refuses any inline script that lacks the nonce.
const frameNonce =
  globalThis.crypto?.randomUUID?.() ?? Math.random().toString(36).slice(2);

const BRIDGE = `<script nonce="${frameNonce}">
(function () {
  var host = window.parent;
  function post(type, data) {
    host.postMessage({ source: 'kestrel-mail', type: type, data: data }, '*');
  }
  function item(cb) {
    return cb.closest('.kestrel-task-item');
  }
  function paint(cb) {
    var node = item(cb);
    var text = node && node.querySelector('.kestrel-task-text');
    if (!text) return;
    text.style.textDecoration = cb.checked ? 'line-through' : 'none';
    text.style.opacity = cb.checked ? '0.7' : '1';
  }
  function init(states) {
    var boxes = document.querySelectorAll('.kestrel-task-checkbox');
    for (var i = 0; i < boxes.length; i++) {
      var cb = boxes[i];
      var id = item(cb) && item(cb).getAttribute('data-task-id');
      if (id && states && Object.prototype.hasOwnProperty.call(states, id)) {
        cb.checked = states[id];
      }
      paint(cb);
      cb.addEventListener('change', function (box) {
        return function () {
          paint(box);
          var taskId = item(box) && item(box).getAttribute('data-task-id');
          if (taskId) post('checklist', { taskId: taskId, checked: box.checked });
        };
      }(cb));
    }
  }
  function report() {
    post('height', document.documentElement.scrollHeight + 20);
  }
  window.addEventListener('message', function (event) {
    var msg = event.data;
    if (event.source !== host || !msg || msg.source !== 'kestrel-host') return;
    if (msg.type === 'init') {
      init(msg.data);
      report();
    }
  });
  document.addEventListener('selectionchange', function () {
    var sel = window.getSelection();
    post('selection', sel ? String(sel).trim().slice(0, 2000) : '');
  });
  window.addEventListener('load', function () {
    report();
    if (window.ResizeObserver) new ResizeObserver(report).observe(document.body);
  });
})();
</script>`;

/**
 * Build the sandboxed `srcdoc` for a sanitized email body: restrictive CSP,
 * nonced bridge script, then the (already DOMPurify'd) message HTML.
 */
export function buildMailFrameSrcdoc(html: string, allowRemoteImages: boolean): string {
  const csp = [
    "default-src 'none'",
    "style-src 'unsafe-inline'",
    `script-src 'nonce-${frameNonce}'`,
    allowRemoteImages ? 'img-src * data: blob:' : "img-src 'none'",
  ].join('; ');

  return `<meta http-equiv="Content-Security-Policy" content="${csp}">\n${BRIDGE}\n${html}`;
}
