import type { CaptainHudLayout } from './hud-state.svelte.js';

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

export function parseCaptainLayout(aniContents: string): CaptainHudLayout {
  const frames = parseAniLines(aniContents);

  if (frames.length < 15) {
    throw new Error(`Captain ani has ${frames.length} frames, expected at least 15.`);
  }

  return {
    turnLeft: { start: 1, count: 5, style: '' },
    thrust: { start: 6, count: 3, style: '' },
    weapon: { start: 9, count: 3, style: '' },
    special: { start: 12, count: frames.length - 12, style: '' },
  };
}

export function parseCaptainFrameFiles(aniContents: string): string[] {
  return parseAniLines(aniContents).map((frame) => frame.file);
}

export function parseCaptainFrameStyles(aniContents: string): string[] {
  return parseAniLines(aniContents).map((frame) => toStyle(frame.x, frame.y));
}
