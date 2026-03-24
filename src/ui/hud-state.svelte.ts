import type { ShipInput } from '../ships/ship-state.js';

const ANIM = {
  turnLeft:  { start: 1, count: 5 },
  thrust:    { start: 6, count: 3 },
  weapon:    { start: 9, count: 3 },
  special:   { start: 12, count: 3 },
} as const;

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
  captainFrame = $state(0);

  private thrustOffset = 0;
  private weaponOffset = 0;
  private specialOffset = 0;
  private turnOffset = 0;

  updateInput(input: ShipInput) {
    this.thrustOffset = stepOffset(this.thrustOffset, input.thrust, ANIM.thrust.count);
    this.weaponOffset = stepOffset(this.weaponOffset, input.weapon, ANIM.weapon.count);
    this.specialOffset = stepOffset(this.specialOffset, input.special, ANIM.special.count);
    this.turnOffset = stepOffset(this.turnOffset, input.left || input.right, ANIM.turnLeft.count);

    let frame = 0;
    if (this.specialOffset > 0) {
      frame = ANIM.special.start + this.specialOffset - 1;
    } else if (this.weaponOffset > 0) {
      frame = ANIM.weapon.start + this.weaponOffset - 1;
    } else if (this.thrustOffset > 0) {
      frame = ANIM.thrust.start + this.thrustOffset - 1;
    } else if (this.turnOffset > 0) {
      frame = ANIM.turnLeft.start + this.turnOffset - 1;
    }

    this.captainFrame = frame;
  }
}
