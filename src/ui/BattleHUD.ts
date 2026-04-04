import { mount, unmount } from 'svelte';
import BattleHUDComponent from './BattleHUD.svelte';
import { getCrewBarMax, HudState } from './hud-state.svelte.js';
import type { ShipStats, ShipInput } from '../ships/ship-stats.js';
import type { CaptainHudLayout, OtherShipHudState } from './hud-state.svelte.js';

export interface HUDShipInfo {
  ship: { crew: number; energy: number; dead?: boolean };
  stats: ShipStats;
  portraitUrl: string;
  captainFrameUrls: string[];
  captainFrameStyles: string[];
  captainLayout: CaptainHudLayout;
  captainName: string;
}

export class BattleHUD {
  readonly hudState = new HudState();
  private container: HTMLElement;
  private component: Record<string, unknown>;
  private ship: HUDShipInfo | null = null;

  constructor() {
    const hudRoot = document.getElementById('hud');
    if (!hudRoot) {
      throw new Error('Missing #hud mount point');
    }

    this.container = document.createElement('div');
    this.container.id = 'battle-hud';
    this.container.className = 'h-full w-full pointer-events-none';
    hudRoot.appendChild(this.container);

    this.component = mount(BattleHUDComponent, {
      target: this.container,
      props: { state: this.hudState },
    });
  }

  setShip(info: HUDShipInfo) {
    this.ship = info;
    this.hudState.crew = info.ship.crew;
    this.hudState.maxCrew = info.stats.maxCrew;
    this.hudState.crewBarMax = getCrewBarMax(info.stats.spritePrefix, info.stats.maxCrew);
    this.hudState.energy = info.ship.energy;
    this.hudState.maxEnergy = info.stats.maxEnergy;
    this.hudState.energyBarMax = info.stats.maxEnergy;
    this.hudState.dead = Boolean(info.ship.dead);
    this.hudState.shipName = info.stats.raceName.toUpperCase();
    this.hudState.captainName = info.captainName;
    this.hudState.portraitUrl = info.portraitUrl;
    this.hudState.configureCaptain(info.captainFrameUrls, info.captainLayout, info.captainFrameStyles);

    // Preload captain frames
    for (const url of info.captainFrameUrls) {
      const img = new Image();
      img.src = url;
    }
  }

  update(input: ShipInput) {
    if (!this.ship) return;

    this.hudState.crew = this.ship.ship.crew;
    this.hudState.energy = this.ship.ship.energy;
    this.hudState.dead = Boolean(this.ship.ship.dead);
    this.hudState.updateInput(input);
  }

  setFleet(allies: OtherShipHudState[], opponents: OtherShipHudState[]) {
    this.hudState.setFleet(allies, opponents);
  }

  destroy() {
    unmount(this.component);
    this.container.remove();
  }
}
