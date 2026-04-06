import type { ShipInput } from '../ships/ship-stats.js';

export interface CaptainAnimationLayer {
  start: number;
  count: number;
  style: string;
}

export interface CaptainRenderLayer {
  url: string;
  style: string;
}

export interface CaptainHudLayout {
  turnLeft: CaptainAnimationLayer;
  thrust: CaptainAnimationLayer;
  weapon: CaptainAnimationLayer;
  special: CaptainAnimationLayer;
}

export interface OtherShipHudState {
  id: string;
  portraitUrl: string;
  portraitHeight: number;
  renderScale: number;
  captainName: string;
  shipName: string;
  crew: number;
  maxCrew: number;
  crewBarMax: number;
  energy: number;
  maxEnergy: number;
  energyBarMax: number;
  dead: boolean;
}

const SYREEN_PREFIX = 'syreen-penetrator';
const CHMMR_CREW_BAR_MAX = 42;
const MAX_OTHER_SHIP_PORTRAIT_HEIGHT = 36;
const BASE_SHIP_RENDER_SIZE = 16;

const DEFAULT_LAYOUT: CaptainHudLayout = {
  turnLeft: { start: 1, count: 3, style: '' },
  thrust: { start: 4, count: 2, style: '' },
  weapon: { start: 6, count: 2, style: '' },
  special: { start: 8, count: 2, style: '' },
};

export function getFleetPanelKeys(teammateCount: number): string[] {
  if (teammateCount === 0) {
    return ['opponents'];
  }

  return ['teammates', 'opponents'];
}

export function getCrewBarMax(spritePrefix: string, maxCrew: number): number {
  if (spritePrefix === SYREEN_PREFIX) {
    return CHMMR_CREW_BAR_MAX;
  }

  return maxCrew;
}

export function getShipRenderScale(size: number): number {
  return size / BASE_SHIP_RENDER_SIZE;
}

export function getOtherShipPortraitHeight(_size: number): number {
  return MAX_OTHER_SHIP_PORTRAIT_HEIGHT;
}

export function stepCaptainOffset(offset: number, active: boolean): number {
  if (active && offset < 2) return offset + 1;
  if (!active && offset > 0) return offset - 1;
  return offset;
}

export function getCaptainTurnFrameIndexes(turnStart: number, leftTurnOffset: number, rightTurnOffset: number): number[] {
  if (leftTurnOffset > 0) {
    const indexes = [turnStart + 3, turnStart + 2];

    for (let i = 0; i < leftTurnOffset; i += 1) {
      indexes.push(turnStart + 1 - i);
    }

    return indexes;
  }

  if (rightTurnOffset > 0) {
    const indexes = [turnStart + 1, turnStart + 2];

    for (let i = 0; i < rightTurnOffset; i += 1) {
      indexes.push(turnStart + 3 + i);
    }

    return indexes;
  }

  return [];
}

export function getCaptainAnimationFrameIndex(start: number, offset: number): number | null {
  if (offset <= 0) {
    return null;
  }

  return start + offset;
}

export class HudState {
  crew = $state(0);
  maxCrew = $state(0);
  crewBarMax = $state(0);
  energy = $state(0);
  maxEnergy = $state(0);
  energyBarMax = $state(0);
  dead = $state(false);
  deathAnimationFrame = $state(0);
  allies = $state<OtherShipHudState[]>([]);
  opponents = $state<OtherShipHudState[]>([]);
  shipName = $state('');
  captainName = $state('');
  portraitUrl = $state('');
  captainFrameUrls = $state<string[]>([]);
  captainFrameStyles = $state<string[]>([]);
  captainBackgroundUrl = $state('');
  captainTurnLayers = $state<CaptainRenderLayer[]>([]);
  captainThrustUrl = $state('');
  captainWeaponUrl = $state('');
  captainSpecialUrl = $state('');
  captainTurnStyle = $state('');
  captainThrustStyle = $state('');
  captainWeaponStyle = $state('');
  captainSpecialStyle = $state('');

  private leftTurnOffset = 0;
  private rightTurnOffset = 0;
  private thrustOffset = 0;
  private weaponOffset = 0;
  private specialOffset = 0;
  private captainLayout: CaptainHudLayout = DEFAULT_LAYOUT;
  private wasDead = false;

  configureCaptain(urls: string[], layout: CaptainHudLayout, frameStyles: string[] = []) {
    this.captainFrameUrls = urls;
    this.captainFrameStyles = frameStyles;
    this.captainLayout = layout;
    this.captainTurnStyle = layout.turnLeft.style;
    this.captainThrustStyle = layout.thrust.style;
    this.captainWeaponStyle = layout.weapon.style;
    this.captainSpecialStyle = layout.special.style;
    this.leftTurnOffset = 0;
    this.rightTurnOffset = 0;
    this.thrustOffset = 0;
    this.weaponOffset = 0;
    this.specialOffset = 0;
    this.syncCaptainLayers();
  }

  setFleet(allies: OtherShipHudState[], opponents: OtherShipHudState[]) {
    this.allies = allies;
    this.opponents = opponents;
  }

  updateInput(input: ShipInput) {
    if (this.dead && !this.wasDead) {
      this.deathAnimationFrame = 0;
    }
    if (this.dead && this.deathAnimationFrame < 40) {
      this.deathAnimationFrame += 1;
    }
    if (!this.dead) {
      this.deathAnimationFrame = 0;
    }
    this.wasDead = this.dead;

    if (input.left && !input.right) {
      this.rightTurnOffset = 0;
      this.leftTurnOffset = stepCaptainOffset(this.leftTurnOffset, true);
    } else if (input.right) {
      this.leftTurnOffset = 0;
      this.rightTurnOffset = stepCaptainOffset(this.rightTurnOffset, true);
    } else {
      this.leftTurnOffset = stepCaptainOffset(this.leftTurnOffset, false);
      this.rightTurnOffset = stepCaptainOffset(this.rightTurnOffset, false);
    }

    this.thrustOffset = stepCaptainOffset(this.thrustOffset, input.thrust);
    this.weaponOffset = stepCaptainOffset(this.weaponOffset, input.weapon);
    this.specialOffset = stepCaptainOffset(this.specialOffset, input.special);
    this.syncCaptainLayers();
  }

  private syncCaptainLayers() {
    this.captainBackgroundUrl = this.captainFrameUrls[0] ?? '';
    this.captainTurnLayers = this.getTurnLayers();
    this.captainThrustUrl = this.getAnimationUrl(this.captainLayout.thrust.start, this.thrustOffset);
    this.captainWeaponUrl = this.getAnimationUrl(this.captainLayout.weapon.start, this.weaponOffset);
    this.captainSpecialUrl = this.getAnimationUrl(this.captainLayout.special.start, this.specialOffset);
  }

  private getTurnLayers() {
    return getCaptainTurnFrameIndexes(
      this.captainLayout.turnLeft.start,
      this.leftTurnOffset,
      this.rightTurnOffset,
    )
      .map((index) => this.getFrameLayer(index, this.captainLayout.turnLeft.style))
      .filter((layer) => layer.url.length > 0);
  }

  private getAnimationUrl(start: number, offset: number) {
    const index = getCaptainAnimationFrameIndex(start, offset);
    if (index === null) {
      if (start === this.captainLayout.thrust.start) {
        this.captainThrustStyle = this.captainLayout.thrust.style;
      } else if (start === this.captainLayout.weapon.start) {
        this.captainWeaponStyle = this.captainLayout.weapon.style;
      } else if (start === this.captainLayout.special.start) {
        this.captainSpecialStyle = this.captainLayout.special.style;
      }

      return '';
    }

    this.captainThrustStyle = start === this.captainLayout.thrust.start
      ? this.getFrameStyle(index, this.captainLayout.thrust.style)
      : this.captainThrustStyle;
    this.captainWeaponStyle = start === this.captainLayout.weapon.start
      ? this.getFrameStyle(index, this.captainLayout.weapon.style)
      : this.captainWeaponStyle;
    this.captainSpecialStyle = start === this.captainLayout.special.start
      ? this.getFrameStyle(index, this.captainLayout.special.style)
      : this.captainSpecialStyle;

    return this.getAbsoluteFrameUrl(index);
  }

  private getFrameLayer(index: number, baseStyle: string): CaptainRenderLayer {
    return {
      url: this.getAbsoluteFrameUrl(index),
      style: this.getFrameStyle(index, baseStyle),
    };
  }

  private getFrameStyle(index: number, baseStyle: string) {
    const override = this.captainFrameStyles[index] ?? '';
    if (!override) {
      return baseStyle;
    }

    if (!baseStyle) {
      return override;
    }

    return `${baseStyle} ${override}`;
  }

  private getAbsoluteFrameUrl(index: number) {
    return this.captainFrameUrls[index] ?? '';
  }
}
