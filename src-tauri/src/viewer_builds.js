(function () {
  'use strict';
  if (window.__managerBuildsInstalled) return;
  window.__managerBuildsInstalled = true;
  var data, panel, list, status, name, content, shadow, toggle, backdrop, recoveryList, busy = false;
  var palettes = {
    mocha: ['#1e1e2e', '#313244', '#45475a', '#cdd6f4', '#a6adc8', '#89b4fa', '#f38ba8'],
    macchiato: ['#24273a', '#363a4f', '#494d64', '#cad3f5', '#a5adcb', '#8aadf4', '#ed8796'],
    frappe: ['#303446', '#414559', '#51576d', '#c6d0f5', '#a5adce', '#8caaee', '#e78284'],
    latte: ['#eff1f5', '#ccd0da', '#bcc0cc', '#4c4f69', '#6c6f85', '#1e66f5', '#d20f39']
  };
  function element(tag, text, parent) {
    var node = document.createElement(tag);
    if (text) node.textContent = text;
    if (parent) parent.appendChild(node);
    return node;
  }
  function button(text, parent, action) {
    var node = element('button', text, parent); node.type = 'button'; node.onclick = action; return node;
  }
  function message(text, error) { status.textContent = text; status.classList.toggle('error', !!error); }
  function adapter() { return window.__cyoaManagerAdapter || {}; }
  async function request(method, body) {
    var response = await fetch('/__manager_builds', {
      method: method, headers: { 'Content-Type': 'application/json' },
      body: body ? JSON.stringify(body) : undefined
    });
    var result = await response.json();
    if (!response.ok) throw new Error(result.error || 'Build operation failed');
    return result;
  }
  async function operation(action) {
    if (busy) return;
    busy = true; panel.setAttribute('aria-busy', 'true');
    shadow.querySelectorAll('button').forEach(function (node) { node.disabled = true; });
    try { await action(); } catch (e) { message(e.message || String(e), true); }
    finally {
      busy = false; panel.removeAttribute('aria-busy');
      shadow.querySelectorAll('button').forEach(function (node) { node.disabled = false; });
    }
  }
  function download(text, filename, type) {
    var url = URL.createObjectURL(new Blob([text], { type: type }));
    var anchor = element('a', '', shadow); anchor.href = url; anchor.download = filename; anchor.click(); anchor.remove();
    setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
  }
  function filename(build) { return (build.project_name + ' - ' + build.name).replace(/[<>:"/\\|?*\x00-\x1f]/g, '_').slice(0, 160); }
  function compatible(build) { return build.fingerprint === data.fingerprint && build.viewer_id === data.viewerId; }
  function selectCode(build) {
    content.value = build.content; name.value = build.name; content.closest('details').open = true;
    content.focus(); content.select();
  }
  function allowVersion(build) {
    return compatible(build) || window.confirm('This build uses another version or viewer. Choices may have changed. Continue?');
  }
  async function copyCode(build) {
    selectCode(build);
    try {
      if (!navigator.clipboard) throw new Error('Clipboard unavailable');
      await navigator.clipboard.writeText(build.content); message('Build code copied.');
    } catch (_) { message('Code selected below. Copy it into the viewer’s native import control.'); }
  }
  async function refresh() {
    data = await request('GET');
    panel.querySelector('h2').textContent = data.projectName + ' — Builds';
    list.replaceChildren();
    if (!data.builds.length) element('p', 'No builds saved for this CYOA.', list);
    data.builds.forEach(function (build) {
      var item = element('section', '', list); item.className = 'build';
      element('strong', build.name, item);
      element('p', new Date(build.saved_at).toLocaleString() + ' · ' + (compatible(build) ? 'Current version' : 'Different CYOA version or viewer'), item);
      var actions = element('div', '', item); actions.className = 'actions';
      button('Load build', actions, function () { operation(async function () {
        if (!allowVersion(build)) return;
        var loader = adapter().load;
        var result = loader ? await loader(build.content) : false;
        if (result === false || result === null) {
          selectCode(build); message('Automatic loading is unavailable for this viewer. Copy the code or download a viewer file, then use its native import control.');
        } else { message('Build loaded.'); }
      }); });
      button('Copy code', actions, function () { operation(async function () { if (allowVersion(build)) await copyCode(build); }); });
      button('Export identified save', actions, function () { download(JSON.stringify(build, null, 2), filename(build) + '.cyoa-build.json', 'application/json'); });
      button('Download viewer file', actions, function () { if (allowVersion(build)) download(build.content, filename(build) + '.txt', 'text/plain'); });
    });
  }
  async function save(capture) {
    if (!name.value.trim()) { name.focus(); throw new Error('Enter a build name.'); }
    var code = content.value;
    if (capture) {
      code = adapter().capture ? await adapter().capture() : null;
      if (typeof code !== 'string' || !code.trim()) {
        content.closest('details').open = true;
        throw new Error('This viewer could not export the current choices. Use its native export, then import the file or paste the code below.');
      }
    }
    if (!code.trim()) { content.focus(); throw new Error('Paste build code or import a viewer file first.'); }
    await request('POST', { name: name.value.trim(), content: code });
    content.value = code; await refresh(); message('Build saved for this CYOA and version.');
  }
  async function importLegacy() {
    recoveryList.replaceChildren();
    var legacy = adapter().legacy ? await adapter().legacy() : [];
    if (!Array.isArray(legacy) || !legacy.length) return '';
    var recovered = 0, failed = [];
    for (var entry of legacy) {
      try {
        if (typeof entry.content !== 'string') throw new Error('Unsupported stored format');
        await request('POST', { name: entry.name || 'Recovered build', content: entry.content, source: entry.source, legacy: true });
        recovered++;
      } catch (e) { failed.push({ entry: entry, reason: e.message }); }
    }
    await refresh();
    if (failed.length) {
      var recovery = element('section', '', recoveryList); recovery.className = 'build';
      element('strong', 'Saved data needing review', recovery);
      element('p', 'These entries were preserved. Check their CYOA before associating them here.', recovery);
      failed.forEach(function (failure) {
        var row = element('div', '', recovery); row.className = 'recovery';
        element('p', (failure.entry.name || failure.entry.source || 'Stored build') + ': ' + failure.reason, row);
        button('Review code', row, function () {
          var value = failure.entry.content;
          selectCode({ name: failure.entry.name || 'Recovered build', content: typeof value === 'string' ? value : JSON.stringify(value, null, 2) });
          message('Review the recovered data below. Save pasted code only if it belongs to this CYOA.');
        });
        if (typeof failure.entry.content === 'string' && failure.entry.content.trim()) {
          button('Associate with this CYOA', row, function () { operation(async function () {
            if (!window.confirm('Associate “' + (failure.entry.name || 'Recovered build') + '” with ' + data.projectName + '? Only continue if you know this saved build belongs to this CYOA. Its original stored data will be preserved.')) return;
            await request('POST', { name: failure.entry.name || 'Recovered build', content: failure.entry.content, source: failure.entry.source });
            await refresh(); row.remove();
            if (!recovery.querySelector('.recovery')) recovery.remove();
            message('Recovered build associated with this CYOA. Original saved data preserved.');
          }); });
        }
      });
    }
    return recovered + ' existing build' + (recovered === 1 ? '' : 's') + ' recovered' + (failed.length ? '; ' + failed.length + ' require review.' : '.');
  }
  async function sessionTheme() {
    try {
      var response = await fetch('/__manager_session'); if (!response.ok) return;
      var session = await response.json(), colors = palettes[session.theme] || palettes.mocha;
      ['base', 'surface', 'border', 'text', 'muted', 'accent', 'error'].forEach(function (key, index) { shadow.host.style.setProperty('--' + key, colors[index]); });
    } catch (_) { /* The default palette also works when session settings are unavailable. */ }
  }
  function closePanel() { panel.hidden = true; backdrop.hidden = true; toggle.focus(); }
  function start() {
    var host = element('div', '', document.body); host.id = 'manager-builds-host';
    host.style.cssText = 'all:initial;position:fixed;inset:0;pointer-events:none;z-index:2147483647;font:14px/1.5 system-ui,sans-serif';
    shadow = host.attachShadow({ mode: 'open' });
    var style = element('style', '', shadow);
    style.textContent = ':host{font-weight:400;--base:#1e1e2e;--surface:#313244;--border:#45475a;--text:#cdd6f4;--muted:#a6adc8;--accent:#89b4fa;--error:#f38ba8;font:14px/1.5 system-ui,sans-serif;color:var(--text)}*{box-sizing:border-box}button,input,textarea{font:inherit}button{cursor:pointer;border:1px solid var(--border);border-radius:8px;background:var(--surface);color:var(--text);padding:9px 12px}button:hover{border-color:var(--accent)}button:disabled{opacity:.55;cursor:wait}button:focus-visible,input:focus-visible,textarea:focus-visible,summary:focus-visible{outline:2px solid var(--accent);outline-offset:2px}.primary{background:var(--accent);color:var(--base);font-weight:400}.dock{position:fixed;left:60px;bottom:8px;pointer-events:auto;padding:7px 12px;box-shadow:0 2px 10px #0004}.panel{position:fixed;inset:4vh 4vw;margin:auto;max-width:820px;max-height:92vh;background:var(--base);color:var(--text);border:1px solid var(--border);border-radius:14px;padding:24px;overflow:auto;pointer-events:auto;box-shadow:0 12px 50px #0008;overscroll-behavior:contain}.panel[hidden],.backdrop[hidden]{display:none}.backdrop{position:fixed;inset:0;background:#0006;pointer-events:auto}header{display:flex;align-items:flex-start;gap:16px;justify-content:space-between}h2{font-size:24px;line-height:1.25;margin:0;overflow-wrap:anywhere}p{margin:8px 0;color:var(--muted)}label{display:flex;flex-direction:column;gap:6px;margin:14px 0;font-weight:400}input,textarea{width:100%;min-width:0;padding:10px;border:1px solid var(--border);border-radius:8px;background:var(--surface);color:var(--text);user-select:text}textarea{resize:vertical;min-height:110px}input[type=file]{font-weight:400}input::file-selector-button{background:var(--accent);color:var(--base);border:0;border-radius:5px;padding:6px 10px;margin-right:10px}.actions{display:flex;flex-wrap:wrap;gap:8px;margin:12px 0}details{border:1px solid var(--border);border-radius:8px;padding:12px;margin-top:14px}summary{cursor:pointer}.build{border-top:1px solid var(--border);padding:16px 0;overflow-wrap:anywhere}.recovery{padding:8px 0}.error{color:var(--error)}#status{min-height:1.5em}@media(max-width:520px){.panel{inset:8px;max-height:calc(100vh - 16px);padding:16px}h2{font-size:20px}.actions button{flex:1 1 auto}}';
    backdrop = element('div', '', shadow); backdrop.className = 'backdrop'; backdrop.hidden = true;
    backdrop.onclick = closePanel;
    toggle = button('Builds', window.__cyoaManagerDock().querySelector('nav'), function () { operation(async function () {
      if (!panel.hidden) { closePanel(); return; }
      panel.hidden = false; backdrop.hidden = false; name.focus(); message('Loading builds…');
      await sessionTheme(); await refresh(); message(await importLegacy());
    }); }); toggle.className = 'dock'; toggle.id = 'manager-builds-button';
    panel = element('section', '', shadow); panel.id = 'manager-builds'; panel.className = 'panel'; panel.hidden = true;
    panel.setAttribute('role', 'dialog'); panel.setAttribute('aria-modal', 'true'); panel.setAttribute('aria-labelledby', 'manager-builds-title'); panel.tabIndex = -1;
    var header = element('header', '', panel), heading = element('h2', 'Builds', header); heading.id = 'manager-builds-title';
    button('Close', header, closePanel);
    element('p', 'Save current choices directly. Builds stay associated with this CYOA and its version.', panel);
    var label = element('label', 'Build name', panel); name = element('input', '', label); name.maxLength = 200; name.placeholder = 'e.g. First playthrough';
    var actions = element('div', '', panel); actions.className = 'actions';
    var current = button('Save current build', actions, function () { operation(function () { return save(true); }); }); current.className = 'primary';
    label = element('label', 'Import a viewer text file or identified save', panel);
    var file = element('input', '', label); file.type = 'file'; file.accept = '.txt,.json';
    var manual = element('details', '', panel); element('summary', 'Paste or review build code', manual);
    label = element('label', 'Viewer build code', manual); content = element('textarea', '', label); content.rows = 4;
    button('Save pasted code', manual, function () { operation(function () { return save(false); }); });
    file.onchange = function () { operation(async function () {
      try {
        var selected = file.files[0]; if (!selected) return;
        if (selected.size > 4 * 1024 * 1024) throw new Error('Build exceeds 4 MiB');
        var text = await selected.text();
        if (selected.name.endsWith('.cyoa-build.json')) {
          var build = JSON.parse(text);
          if (build.project_id !== data.projectId) throw new Error('This build belongs to a different CYOA. Open that CYOA to import it.');
          if (!compatible(build)) throw new Error('This identified save belongs to a different version or viewer. Restore that version first.');
          if (typeof build.content !== 'string' || typeof build.name !== 'string') throw new Error('Invalid identified save');
          selectCode(build); message('Identity checked. Click Save pasted code to import.');
        } else {
          selectCode({ content: text, name: selected.name.replace(/\.[^.]+$/, '') });
          message('This file has no CYOA identity. Check that it belongs here, then click Save pasted code.');
        }
      } finally { file.value = ''; }
    }); };
    status = element('p', '', panel); status.id = 'status'; status.setAttribute('role', 'status'); status.setAttribute('aria-live', 'polite');
    recoveryList = element('div', '', panel);
    list = element('div', '', panel);
    panel.addEventListener('keydown', function (event) {
      if (event.key === 'Escape') { event.preventDefault(); closePanel(); }
      if (event.key === 'Tab') {
        var focusable = Array.from(panel.querySelectorAll('button,input,textarea,summary')).filter(function (node) { return !node.disabled && node.getClientRects().length; });
        var first = focusable[0], last = focusable[focusable.length - 1], active = shadow.activeElement;
        if (event.shiftKey && (active === first || active === panel)) { event.preventDefault(); if (last) last.focus(); }
        else if (!event.shiftKey && active === last) { event.preventDefault(); if (first) first.focus(); }
      }
    });
    sessionTheme();
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start, { once: true });
  else start();
})();
