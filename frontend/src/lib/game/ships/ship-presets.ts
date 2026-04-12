import type { GameLogic } from 'game-logic-wasm';
import type { CaptainHudLayout } from '../ui/hud-state.svelte.js';
import { getShipRenderScale } from '../ui/hud-state.svelte.js';
import { parseCaptainFrameFiles, parseCaptainFrameStyles, parseCaptainLayout } from '../ui/captain-layout.js';
import type { ShipStats } from './ship-stats.js';

const CAPTAIN_FRAME_MODULES = import.meta.glob('../assets/ships/*/*-cap-*.png', { eager: true, import: 'default' }) as Record<string, string>;
const CAPTAIN_ANI_MODULES = import.meta.glob('../assets/ships/*/*-cap.ani', { eager: true, query: '?raw', import: 'default' }) as Record<string, string>;
const PORTRAIT_MODULES = import.meta.glob('../assets/ships/*/*-icons-001.png', { eager: true, import: 'default' }) as Record<string, string>;
const SELECTION_PORTRAIT_MODULES = import.meta.glob('../assets/ships/*/*-med-000.png', { eager: true, import: 'default' }) as Record<string, string>;

export interface ShipPreset {
  stats: ShipStats;
  portraitUrl: string;
  selectionPortraitUrl: string;
  captainFrameUrls: string[];
  captainFrameStyles: string[];
  captainLayout: CaptainHudLayout;
}

function readMatchingEntries(modules: Record<string, string>, pattern: string) {
  return Object.entries(modules)
    .filter(([path]) => path.includes(pattern))
    .sort(([a], [b]) => a.localeCompare(b));
}

function requireSingleModule(modules: Record<string, string>, pattern: string) {
  const matches = readMatchingEntries(modules, pattern);
  if (matches.length !== 1) {
    throw new Error(`Expected exactly one asset for pattern "${pattern}", found ${matches.length}.`);
  }

  return matches[0][1];
}

function parseSpritePrefix(prefix: string): { folder: string; base: string } {
  const idx = prefix.indexOf('-');
  return { folder: prefix.slice(0, idx), base: prefix.slice(idx + 1) };
}

function buildShipPreset(stats: ShipStats): ShipPreset {
  const enrichedStats: ShipStats = {
    ...stats,
    renderScale: getShipRenderScale(stats.size),
  };
  const { folder, base } = parseSpritePrefix(stats.spritePrefix);
  const portraitUrl = requireSingleModule(PORTRAIT_MODULES, `/${folder}/${base}-icons-001.png`);
  const selectionPortraitUrl = requireSingleModule(SELECTION_PORTRAIT_MODULES, `/${folder}/${base}-med-000.png`);
  const captainAni = requireSingleModule(CAPTAIN_ANI_MODULES, `/${folder}/${base}-cap.ani`);
  const captainFrameEntries = parseCaptainFrameFiles(captainAni);
  const frameUrlsByFile = new Map(readMatchingEntries(CAPTAIN_FRAME_MODULES, `/${folder}/${base}-cap-`).map(([path, url]) => [
    path.split('/').pop()!,
    url,
  ]));
  const captainFrameUrls = captainFrameEntries.map((file) => {
    const url = frameUrlsByFile.get(file);
    if (!url) {
      throw new Error(`Missing captain frame "${file}" for ${folder}/${base}.`);
    }

    return url;
  });
  const captainFrameStyles = parseCaptainFrameStyles(captainAni);

  return {
    stats: enrichedStats,
    portraitUrl,
    selectionPortraitUrl,
    captainFrameUrls,
    captainFrameStyles,
    captainLayout: parseCaptainLayout(captainAni),
  };
}

export function buildShipPresets(gameLogic: GameLogic): ShipPreset[] {
  const shipTypes = gameLogic.getAllShipTypes() as string[];
  return shipTypes
    .map((type) => {
      const stats = gameLogic.getStatsByType(type) as unknown as ShipStats;
      return buildShipPreset(stats);
    })
    .sort((a, b) => a.stats.spritePrefix.localeCompare(b.stats.spritePrefix));
}
