import type { CaptainHudLayout } from './hud-state.svelte.js';

interface CaptainSection {
  readonly start: number;
  readonly count: number;
  readonly x: number;
  readonly y: number;
}

export interface CaptainAniFrame {
  readonly file: string;
  readonly x: number;
  readonly y: number;
}

function toStyle(x: number, y: number) {
  return `left: ${-x}px; top: ${-y}px;`;
}

function parseAniLines(aniContents: string): CaptainAniFrame[] {
  return aniContents
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0 && !line.startsWith('#') && !line.startsWith('//'))
    .map((line) => {
      const [file, , , xRaw, yRaw] = line.split(/\s+/);
      return {
        file,
        x: Number(xRaw),
        y: Number(yRaw),
      };
    });
}

function parseSections(frames: CaptainAniFrame[]): CaptainSection[] {
  const sections: CaptainSection[] = [];
  let currentX: number | null = null;
  let currentY: number | null = null;
  let currentStart = 1;
  let currentCount = 0;
  let frameIndex = 1;

  for (const frame of frames.slice(1)) {
    const { x, y } = frame;

    if (currentX === null || currentY === null || currentX !== x || currentY !== y) {
      if (currentX !== null && currentY !== null) {
        sections.push({
          start: currentStart,
          count: currentCount,
          x: currentX,
          y: currentY,
        });
        currentStart = frameIndex;
      }

      currentX = x;
      currentY = y;
      currentCount = 0;
    }

    currentCount += 1;
    frameIndex += 1;
  }

  if (currentX !== null && currentY !== null) {
    sections.push({
      start: currentStart,
      count: currentCount,
      x: currentX,
      y: currentY,
    });
  }

  return sections;
}

export function parseCaptainLayout(aniContents: string): CaptainHudLayout {
  const frames = parseAniLines(aniContents);
  const sections = parseSections(frames);
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

export function parseCaptainFrameFiles(aniContents: string): string[] {
  return parseAniLines(aniContents).map((frame) => frame.file);
}

export function parseCaptainFrameStyles(aniContents: string): string[] {
  return parseAniLines(aniContents).map((frame) => toStyle(frame.x, frame.y));
}
