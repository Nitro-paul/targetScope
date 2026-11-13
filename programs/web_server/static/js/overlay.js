// doc/programs/web_server.md#coordinate-conversion-in-the-ui
// doc/programs/web_server.md#target-type-editor

export function displayedToImage(point, displayedWidth, imageWidth) {
  const f = imageWidth / displayedWidth;
  return { x: point.x * f, y: point.y * f };
}

export function imageToDisplayed(point, displayedWidth, imageWidth) {
  const f = displayedWidth / imageWidth;
  return { x: point.x * f, y: point.y * f };
}

// doc/programs/web_server.md#target-type-editor
export function createOverlay(container, image, handles, onChange) {
  container.classList.add('overlay');
  const layer = document.createElement('div');
  layer.className = 'overlay-layer';
  container.appendChild(layer);
  const elements = new Map();
  const place = () => {
    const w = image.clientWidth;
    for (const [key, h] of Object.entries(handles)) {
      const d = imageToDisplayed(h, w, image.naturalWidth || w);
      const e = elements.get(key);
      e.style.left = `${d.x}px`;
      e.style.top = `${d.y}px`;
    }
  };
  for (const key of Object.keys(handles)) {
    const e = document.createElement('div');
    e.className = `handle handle-${key}`;
    e.title = key.replaceAll('_', ' ');
    layer.appendChild(e);
    elements.set(key, e);
    e.addEventListener('pointerdown', (down) => {
      down.preventDefault();
      e.setPointerCapture(down.pointerId);
      const move = (ev) => {
        const r = image.getBoundingClientRect();
        const p = displayedToImage({ x: ev.clientX - r.left, y: ev.clientY - r.top }, image.clientWidth, image.naturalWidth || image.clientWidth);
        handles[key] = { x: Math.max(0, p.x), y: Math.max(0, p.y) };
        place();
        onChange(key, handles[key]);
      };
      const up = () => {
        e.removeEventListener('pointermove', move);
        e.removeEventListener('pointerup', up);
      };
      e.addEventListener('pointermove', move);
      e.addEventListener('pointerup', up);
    });
  }
  if (image.complete) place();
  image.addEventListener('load', place);
  window.addEventListener('resize', place);
  return { place, handles, destroy: () => layer.remove() };
}
