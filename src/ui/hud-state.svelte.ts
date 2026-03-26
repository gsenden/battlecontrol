import type { ShipInput } from '../ships/ship-state.js';

export interface CaptainAnimationLayer {
  start: number;
  count: number;
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

function stepOffset(offset: number, active: boolean, max: number): number {
  if (active && offset < max) return offset + 1;
  if (!active && offset > 0) return offset - 1;
  return offset;
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
  captainBackgroundUrl = $state('');
  captainTurnUrl = $state('');
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

  configureCaptain(urls: string[], layout: CaptainHudLayout) {
    this.captainFrameUrls = urls;
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
      this.leftTurnOffset = stepOffset(this.leftTurnOffset, true, this.captainLayout.turnLeft.count);
    } else if (input.right) {
      this.leftTurnOffset = 0;
      this.rightTurnOffset = stepOffset(this.rightTurnOffset, true, this.captainLayout.turnLeft.count);
    } else {
      this.leftTurnOffset = stepOffset(this.leftTurnOffset, false, this.captainLayout.turnLeft.count);
      this.rightTurnOffset = stepOffset(this.rightTurnOffset, false, this.captainLayout.turnLeft.count);
    }

    this.thrustOffset = stepOffset(this.thrustOffset, input.thrust, this.captainLayout.thrust.count);
    this.weaponOffset = stepOffset(this.weaponOffset, input.weapon, this.captainLayout.weapon.count);
    this.specialOffset = stepOffset(this.specialOffset, input.special, this.captainLayout.special.count);
    this.syncCaptainLayers();
  }

  private syncCaptainLayers() {
    this.captainBackgroundUrl = this.captainFrameUrls[0] ?? '';
    this.captainTurnUrl = this.getTurnUrl();
    this.captainThrustUrl = this.getFrameUrl(this.captainLayout.thrust.start, this.thrustOffset);
    this.captainWeaponUrl = this.getFrameUrl(this.captainLayout.weapon.start, this.weaponOffset);
    this.captainSpecialUrl = this.getFrameUrl(this.captainLayout.special.start, this.specialOffset);
  }

  private getTurnUrl() {
    if (this.leftTurnOffset > 0) {
      return this.getFrameUrl(this.captainLayout.turnLeft.start, this.leftTurnOffset);
    }

    if (this.rightTurnOffset > 0) {
      const index = this.captainLayout.turnLeft.start + this.captainLayout.turnLeft.count - this.rightTurnOffset;
      return this.captainFrameUrls[index] ?? '';
    }

    return '';
  }

  private getFrameUrl(start: number, offset: number) {
    if (offset <= 0) {
      return '';
    }

    return this.captainFrameUrls[start + offset - 1] ?? '';
  }
}
