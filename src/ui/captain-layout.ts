import type { CaptainHudLayout } from './hud-state.svelte.js';

interface CaptainSection {
  readonly start: number;
  readonly count: number;
  readonly x: number;
  readonly y: number;
}

function toStyle(x: number, y: number) {
  return `left: ${Math.abs(x)}px; top: ${Math.abs(y)}px;`;
}

function parseSections(lines: string[]): CaptainSection[] {
  const sections: CaptainSection[] = [];
  let currentX: number | null = null;
  let currentY: number | null = null;
  let currentStart = 1;
  let currentFrames = new Set<string>();
  let uniqueFrameIndex = 1;

  for (const line of lines.slice(1)) {
    const [file, , , xRaw, yRaw] = line.trim().split(/\s+/);
    const x = Number(xRaw);
    const y = Number(yRaw);

    if (currentX === null || currentY === null || currentX !== x || currentY !== y) {
      if (currentX !== null && currentY !== null) {
        sections.push({
          start: currentStart,
          count: currentFrames.size,
          x: currentX,
          y: currentY,
        });
        currentStart = uniqueFrameIndex;
      }

      currentX = x;
      currentY = y;
      currentFrames = new Set<string>();
    }

    if (!currentFrames.has(file)) {
      currentFrames.add(file);
      uniqueFrameIndex += 1;
    }
  }

  if (currentX !== null && currentY !== null) {
    sections.push({
      start: currentStart,
      count: currentFrames.size,
      x: currentX,
      y: currentY,
    });
  }

  return sections;
}

export function parseCaptainLayout(aniContents: string): CaptainHudLayout {
  const lines = aniContents
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const sections = parseSections(lines);
  const [turnLeft, thrust, weapon, special] = sections;

  if (!turnLeft || !thrust || !weapon || !special) {
    throw new Error('Captain ani does not contain the expected four control sections.');
  }

  return {
    turnLeft: { start: turnLeft.start, count: turnLeft.count, style: toStyle(turnLeft.x, turnLeft.y) },
    thrust: { start: thrust.start, count: thrust.count, style: toStyle(thrust.x, thrust.y) },
    weapon: { start: weapon.start, count: weapon.count, style: toStyle(weapon.x, weapon.y) },
    special: { start: special.start, count: special.count, style: toStyle(special.x, special.y) },
  };
}
