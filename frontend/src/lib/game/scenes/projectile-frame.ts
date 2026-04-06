const SATURN_FACING_COUNT = 16;
const DEFAULT_DIRECTIONAL_FACING_COUNT = 16;

const DIRECTIONAL_PROJECTILES: Record<string, number> = {
  'human-saturn': SATURN_FACING_COUNT,
  'spathi-missile': DEFAULT_DIRECTIONAL_FACING_COUNT,
  'yehat-missile': DEFAULT_DIRECTIONAL_FACING_COUNT,
  'syreen-dagger': DEFAULT_DIRECTIONAL_FACING_COUNT,
  'shofixti-missile': DEFAULT_DIRECTIONAL_FACING_COUNT,
  'mmrnmhrm-torpedo': DEFAULT_DIRECTIONAL_FACING_COUNT,
  'urquan-fighter': DEFAULT_DIRECTIONAL_FACING_COUNT,
};

const ANIMATED_PROJECTILES: Record<string, number> = {
  'androsynth-bubble': 3,
  'chenjesu-spark': 11,
  'chenjesu-doggy': 7,
  'druuge-cannon': 24,
  'urquan-fusion': 24,
  'pkunk-bug': 1,
  'ilwrath-fire': 8,
  'chmmr-satellite': 8,
  'melnorme-pumpup': 26,
  'melnorme-confuse': 16,
  'orz-howitzer': 22,
  'vux-limpets': 4,
  'zoqfotpik-spit': 13,
  'kohrah-buzzsaw': 8,
};

export function velocityToSaturnFrame(vx: number, vy: number): number {
  return velocityToDirectionalFrame(vx, vy, SATURN_FACING_COUNT);
}

export function velocityToDirectionalFrame(vx: number, vy: number, frameCount: number): number {
  let angle = Math.atan2(vy, vx) + Math.PI / 2;
  angle = ((angle % (2 * Math.PI)) + (2 * Math.PI)) % (2 * Math.PI);
  return Math.round(angle / (2 * Math.PI / frameCount)) % frameCount;
}

export function projectileFrameFor(
  texturePrefix: string,
  vx: number,
  vy: number,
  life: number,
): number {
  if (texturePrefix in ANIMATED_PROJECTILES) {
    const frameCount = ANIMATED_PROJECTILES[texturePrefix];
    return frameCount <= 1 ? 0 : Math.abs(life) % frameCount;
  }

  if (texturePrefix in DIRECTIONAL_PROJECTILES) {
    return velocityToDirectionalFrame(vx, vy, DIRECTIONAL_PROJECTILES[texturePrefix]);
  }

  return velocityToDirectionalFrame(vx, vy, DEFAULT_DIRECTIONAL_FACING_COUNT);
}
