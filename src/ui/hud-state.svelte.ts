import type { ShipInput } from '../ships/ship-state.js';

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

const DEFAULT_LAYOUT: CaptainHudLayout = {
  turnLeft: { start: 1, count: 3, style: '' },
  thrust: { start: 4, count: 2, style: '' },
  weapon: { start: 6, count: 2, style: '' },
  special: { start: 8, count: 2, style: '' },
};

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
  energy = $state(0);
  maxEnergy = $state(0);
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

  updateInput(input: ShipInput) {
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
