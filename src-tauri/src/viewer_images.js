// Keep decoded images near the viewport, not every image visited during a session.
// Weak records and removal observation let detached tabs/rows be collected.
(() => {
  const records = new WeakMap();
  const observer = typeof IntersectionObserver === 'function' ? new IntersectionObserver(entries => {
    for (const {target: image, isIntersecting} of entries) {
      const record = records.get(image);
      if (!record) continue;
      record.visible = isIntersecting;
      record.observed = true;
      if (isIntersecting && record.source) image.setAttribute('src', record.source);
      else if (record.placeholder) image.setAttribute('src', record.placeholder);
    }
  }, {rootMargin: '1000px'}) : null;
  window.__cyoaManagerImages = {setSource(image, name, source) {
    if (!(image instanceof HTMLImageElement) || !observer || typeof source !== 'string' || !source) {
      if (records.has(image)) {observer?.unobserve(image);records.delete(image);}
      if (source == null) image.removeAttribute(name); else image.setAttribute(name, source);
      return;
    }
    let record = records.get(image);
    if (!record) {
      record = {source: '', visible: false, observed: false, placeholder: null}; records.set(image, record);
      image.decoding = 'async';
      image.loading = 'lazy';
      image.addEventListener('load', () => {
        if (image.getAttribute('src') !== record.source || !image.naturalWidth) return;
        reserve(image, record, image.naturalWidth, image.naturalHeight);
        if (record.observed && !record.visible) image.setAttribute('src', record.placeholder);
      });
      observer.observe(image);
    }
    if (record.source === source) return;
    record.source = source || '';
    record.placeholder = null;
    if (source.startsWith('/__manager_images/')) {
      const size = new URL(source, location.href).searchParams;
      reserve(image, record, Number(size.get('w')), Number(size.get('h')));
    }
    if (record.visible && record.source) image.setAttribute('src', record.source);
    else image.setAttribute('src', record.placeholder || record.source);
  }};
  function reserve(image, record, width, height) {
    if (!(width>0 && height>0)) return;
    record.placeholder = 'data:image/svg+xml,'+encodeURIComponent(
      `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"/>`);
    // The SVG already supplies the original intrinsic ratio. Width/height
    // attributes here would override authored responsive CSS (width:98% with
    // no explicit height), stretching the image to its full pixel height.
  }
  if (!observer) return;
  const visit = (node, action) => {
    if (!(node instanceof Element)) return;
    if (node instanceof HTMLImageElement && records.has(node)) action(node);
    for (const image of node.querySelectorAll('img')) if (records.has(image)) action(image);
  };
  const mutations = new MutationObserver(changes => {
    for (const change of changes) {
      for (const node of change.removedNodes) visit(node, image => {
        if (!image.isConnected) observer.unobserve(image);
      });
      for (const node of change.addedNodes) visit(node, image => observer.observe(image));
    }
  });
  mutations.observe(document.documentElement, {childList: true, subtree: true});
  window.addEventListener('pagehide', () => {observer.disconnect(); mutations.disconnect();}, {once:true});
})();
