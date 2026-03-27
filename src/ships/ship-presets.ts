import type { CaptainHudLayout } from '../ui/hud-state.svelte.js';
import { parseCaptainFrameFiles, parseCaptainFrameStyles, parseCaptainLayout } from '../ui/captain-layout.js';
import type { ShipStats } from './ship-stats.js';
import { ANDROSYNTH_GUARDIAN } from './androsynth/androsynth-guardian-stats.js';
import { ARILOU_SKIFF } from './arilou/arilou-skiff-stats.js';
import { CHENJESU_BROODHOME } from './chenjesu/chenjesu-broodhome-stats.js';
import { CHMMR_AVATAR } from './chmmr/chmmr-avatar-stats.js';
import { DRUUGE_MAULER } from './druuge/druuge-mauler-stats.js';
import { HUMAN_CRUISER } from './human/human-cruiser-stats.js';
import { ILWRATH_AVENGER } from './ilwrath/ilwrath-avenger-stats.js';
import { KOHRAH_MARAUDER } from './kohrah/kohrah-marauder-stats.js';
import { MELNORME_TRADER } from './melnorme/melnorme-trader-stats.js';
import { MMRNMHRM_XFORM } from './mmrnmhrm/mmrnmhrm-xform-stats.js';
import { MYCON_PODSHIP } from './mycon/mycon-podship-stats.js';
import { ORZ_NEMESIS } from './orz/orz-nemesis-stats.js';
import { PKUNK_FURY } from './pkunk/pkunk-fury-stats.js';
import { SHOFIXTI_SCOUT } from './shofixti/shofixti-scout-stats.js';
import { SLYLANDRO_PROBE } from './slylandro/slylandro-probe-stats.js';
import { SPATHI_ELUDER } from './spathi/spathi-eluder-stats.js';
import { SUPOX_BLADE } from './supox/supox-blade-stats.js';
import { SYREEN_PENETRATOR } from './syreen/syreen-penetrator-stats.js';
import { THRADDASH_TORCH } from './thraddash/thraddash-torch-stats.js';
import { UMGAH_DRONE } from './umgah/umgah-drone-stats.js';
import { URQUAN_DREADNOUGHT } from './urquan/urquan-dreadnought-stats.js';
import { UTWIG_JUGGER } from './utwig/utwig-jugger-stats.js';
import { VUX_INTRUDER } from './vux/vux-intruder-stats.js';
import { YEHAT_TERMINATOR } from './yehat/yehat-terminator-stats.js';
import { ZOQFOTPIK_STINGER } from './zoqfotpik/zoqfotpik-stinger-stats.js';

const CAPTAIN_FRAME_MODULES = import.meta.glob('../assets/ships/*/*-cap-*.png', { eager: true, import: 'default' }) as Record<string, string>;
const CAPTAIN_ANI_MODULES = import.meta.glob('../assets/ships/*/*-cap.ani', { eager: true, query: '?raw', import: 'default' }) as Record<string, string>;
const PORTRAIT_MODULES = import.meta.glob('../assets/ships/*/*-icons-001.png', { eager: true, import: 'default' }) as Record<string, string>;

interface ShipPresetSource {
  stats: ShipStats;
  folder: string;
  base: string;
  captainFrameStyleOverrides?: Record<string, string>;
}

export interface ShipPreset {
  stats: ShipStats;
  portraitUrl: string;
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

function buildShipPreset({ stats, folder, base, captainFrameStyleOverrides = {} }: ShipPresetSource): ShipPreset {
  const portraitUrl = requireSingleModule(PORTRAIT_MODULES, `/${folder}/${base}-icons-001.png`);
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
  const captainFrameStyles = parseCaptainFrameStyles(captainAni).map((style, index) => {
    const file = captainFrameEntries[index];
    return captainFrameStyleOverrides[file] ?? style;
  });

  return {
    stats,
    portraitUrl,
    captainFrameUrls,
    captainFrameStyles,
    captainLayout: parseCaptainLayout(captainAni),
  };
}

const SHIP_PRESET_SOURCES: ShipPresetSource[] = [
  { stats: HUMAN_CRUISER, folder: 'human', base: 'cruiser' },
  { stats: CHMMR_AVATAR, folder: 'chmmr', base: 'avatar' },
  { stats: ANDROSYNTH_GUARDIAN, folder: 'androsynth', base: 'guardian' },
  { stats: ARILOU_SKIFF, folder: 'arilou', base: 'skiff' },
  { stats: CHENJESU_BROODHOME, folder: 'chenjesu', base: 'broodhome' },
  { stats: DRUUGE_MAULER, folder: 'druuge', base: 'mauler' },
  { stats: ILWRATH_AVENGER, folder: 'ilwrath', base: 'avenger' },
  { stats: KOHRAH_MARAUDER, folder: 'kohrah', base: 'marauder' },
  { stats: MELNORME_TRADER, folder: 'melnorme', base: 'trader' },
  { stats: MMRNMHRM_XFORM, folder: 'mmrnmhrm', base: 'xform' },
  { stats: MYCON_PODSHIP, folder: 'mycon', base: 'podship' },
  { stats: ORZ_NEMESIS, folder: 'orz', base: 'nemesis' },
  { stats: PKUNK_FURY, folder: 'pkunk', base: 'fury' },
  { stats: SHOFIXTI_SCOUT, folder: 'shofixti', base: 'scout' },
  { stats: SLYLANDRO_PROBE, folder: 'slylandro', base: 'probe' },
  { stats: SPATHI_ELUDER, folder: 'spathi', base: 'eluder' },
  { stats: SUPOX_BLADE, folder: 'supox', base: 'blade' },
  { stats: SYREEN_PENETRATOR, folder: 'syreen', base: 'penetrator' },
  { stats: THRADDASH_TORCH, folder: 'thraddash', base: 'torch' },
  { stats: UMGAH_DRONE, folder: 'umgah', base: 'drone' },
  { stats: URQUAN_DREADNOUGHT, folder: 'urquan', base: 'dreadnought' },
  { stats: UTWIG_JUGGER, folder: 'utwig', base: 'jugger' },
  { stats: VUX_INTRUDER, folder: 'vux', base: 'intruder' },
  { stats: YEHAT_TERMINATOR, folder: 'yehat', base: 'terminator' },
  { stats: ZOQFOTPIK_STINGER, folder: 'zoqfotpik', base: 'stinger' },
];

export const SHIP_PRESETS = SHIP_PRESET_SOURCES
    .sort((a, b) => a.folder.localeCompare(b.folder))
    .map(buildShipPreset);
