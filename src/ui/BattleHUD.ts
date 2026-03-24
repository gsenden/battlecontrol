import { mount, unmount } from 'svelte';
import BattleHUDComponent from './BattleHUD.svelte';
import { HudState } from './hud-state.svelte.js';
import type { ShipState } from '../ships/ship-state.js';
import type { ShipStats } from '../ships/ship-stats.js';
import type { ShipInput } from '../ships/ship-state.js';

export interface HUDShipInfo {
  state: ShipState;
  stats: ShipStats;
  portraitUrl: string;
  captainFrameUrls: string[];
  captainName: string;
}

export class BattleHUD {
  readonly hudState = new HudState();
  private container: HTMLElement;
  private component: Record<string, unknown>;
  private ship: HUDShipInfo | null = null;

  constructor() {
    this.container = document.createElement('div');
    this.container.id = 'battle-hud';
    this.container.className = 'fixed top-0 right-0 z-100 pointer-events-none';
    document.body.appendChild(this.container);

    this.component = mount(BattleHUDComponent, {
      target: this.container,
      props: { state: this.hudState },
    });
  }

  setShip(info: HUDShipInfo) {
    this.ship = info;
    this.hudState.crew = info.state.crew;
    this.hudState.maxCrew = info.stats.maxCrew;
    this.hudState.energy = info.state.energy;
    this.hudState.maxEnergy = info.stats.maxEnergy;
    this.hudState.shipName = info.stats.raceName.toUpperCase();
    this.hudState.captainName = info.captainName;
    this.hudState.portraitUrl = info.portraitUrl;
    this.hudState.captainFrameUrls = info.captainFrameUrls;

    // Preload captain frames
    for (const url of info.captainFrameUrls) {
      const img = new Image();
      img.src = url;
    }
  }

  update(input: ShipInput) {
    if (!this.ship) return;

    this.hudState.crew = this.ship.state.crew;
    this.hudState.energy = this.ship.state.energy;
    this.hudState.updateInput(input);
  }

  destroy() {
    unmount(this.component);
    this.container.remove();
  }
}
