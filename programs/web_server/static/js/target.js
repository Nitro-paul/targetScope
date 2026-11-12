// SVG target: paper, black area, rings; click → paper mm → polar shot.
// Ring generation is the same as the shared libraries' generate_rings (tested with testdata/target_types).

export function generateRings(targetType) {
  // TODO: r(s) = r_a + (s_a - s) · (r_b - r_a) / (s_a - s_b)
  throw new Error('not implemented');
}

export function validateTargetType(targetType) {
  // TODO: returns null or the first broken rule
  throw new Error('not implemented');
}

// Paper mm (origin top-left, y down) → { angle_turns, radius_mm } around the ring center.
export function paperToShot(targetType, xMm, yMm) {
  throw new Error('not implemented');
}

export function shotToPaper(targetType, shot) {
  throw new Error('not implemented');
}

// Shot label: "3" with one paper, "<paper_no>.<n>" with several.
export function shotLabel(paperCount, paperNo, n) {
  throw new Error('not implemented');
}

export function renderTarget(svg, targetType, shots, options = {}) {
  // TODO
}
