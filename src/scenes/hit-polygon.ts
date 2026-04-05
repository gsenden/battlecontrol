import { NUM_FACINGS } from '../constants.js';

export interface HitPoint {
  x: number;
  y: number;
}

interface OpaqueRow {
  y: number;
  left: number;
  right: number;
}

interface TextureLike {
  key: string;
  source: Array<{ width?: number; height?: number }>;
  getSourceImage(): unknown;
}

const texturePolygonCache = new Map<string, HitPoint[]>();

const HUMAN_CRUISER_BASE_POLYGON: HitPoint[] = [
  { x: 0, y: -68 },
  { x: 11, y: -65 },
  { x: 17, y: -57 },
  { x: 17, y: -43 },
  { x: 11, y: -35 },
  { x: 6, y: -27 },
  { x: 6, y: -14 },
  { x: 13, y: -10 },
  { x: 21, y: 5 },
  { x: 25, y: 39 },
  { x: 24, y: 67 },
  { x: 17, y: 68 },
  { x: 14, y: 42 },
  { x: 10, y: 8 },
  { x: 6, y: 0 },
  { x: 6, y: 66 },
  { x: -6, y: 66 },
  { x: -6, y: 0 },
  { x: -10, y: 8 },
  { x: -14, y: 42 },
  { x: -17, y: 68 },
  { x: -24, y: 67 },
  { x: -25, y: 39 },
  { x: -21, y: 5 },
  { x: -13, y: -10 },
  { x: -6, y: -14 },
  { x: -6, y: -27 },
  { x: -11, y: -35 },
  { x: -17, y: -43 },
  { x: -17, y: -57 },
  { x: -11, y: -65 },
];

const ANDROSYNTH_GUARDIAN_BASE_POLYGON: HitPoint[] = [
  { x: 0, y: -54 },
  { x: 18, y: -44 },
  { x: 24, y: -24 },
  { x: 20, y: -8 },
  { x: 30, y: 8 },
  { x: 24, y: 28 },
  { x: 10, y: 48 },
  { x: 0, y: 56 },
  { x: -10, y: 48 },
  { x: -24, y: 28 },
  { x: -30, y: 8 },
  { x: -20, y: -8 },
  { x: -24, y: -24 },
  { x: -18, y: -44 },
  { x: -8, y: -52 },
  { x: 8, y: -52 },
];

const ANDROSYNTH_BLAZER_BASE_POLYGON: HitPoint[] = [
  { x: 0, y: -60 },
  { x: 14, y: -46 },
  { x: 18, y: -22 },
  { x: 12, y: 6 },
  { x: 6, y: 36 },
  { x: 0, y: 58 },
  { x: -6, y: 36 },
  { x: -12, y: 6 },
  { x: -18, y: -22 },
  { x: -14, y: -46 },
  { x: -6, y: -58 },
  { x: 6, y: -58 },
];

const HUMAN_SATURN_BASE_POLYGON: HitPoint[] = [
  { x: 0, y: -34 },
  { x: 8, y: -22 },
  { x: 10, y: 8 },
  { x: 6, y: 24 },
  { x: 0, y: 34 },
  { x: -6, y: 24 },
  { x: -10, y: 8 },
  { x: -8, y: -22 },
];

const ANDROSYNTH_BUBBLE_BASE_POLYGON: HitPoint[] = [
  { x: 0, y: -20 },
  { x: 14, y: -14 },
  { x: 20, y: 0 },
  { x: 14, y: 14 },
  { x: 0, y: 20 },
  { x: -14, y: 14 },
  { x: -20, y: 0 },
  { x: -14, y: -14 },
];

export function getShipHitPolygon(spritePrefix: string, facing: number, scale = 1): HitPoint[] {
  const basePolygon = getShipBasePolygon(spritePrefix);
  if (!basePolygon) {
    return [];
  }

  return rotateAndScalePolygon(basePolygon, facing, scale);
}

export function getProjectileHitPolygon(texturePrefix: string, facing: number, scale = 1): HitPoint[] {
  const basePolygon = getProjectileBasePolygon(texturePrefix);
  if (!basePolygon) {
    return [];
  }

  return rotateAndScalePolygon(basePolygon, facing, scale);
}

function getShipBasePolygon(spritePrefix: string): HitPoint[] | null {
  switch (spritePrefix) {
    case 'human-cruiser':
      return HUMAN_CRUISER_BASE_POLYGON;
    case 'androsynth-guardian':
      return ANDROSYNTH_GUARDIAN_BASE_POLYGON;
    case 'androsynth-blazer':
      return ANDROSYNTH_BLAZER_BASE_POLYGON;
    default:
      return null;
  }
}

function getProjectileBasePolygon(texturePrefix: string): HitPoint[] | null {
  switch (texturePrefix) {
    case 'human-saturn':
      return HUMAN_SATURN_BASE_POLYGON;
    case 'androsynth-bubble':
      return ANDROSYNTH_BUBBLE_BASE_POLYGON;
    default:
      return null;
  }
}

function rotateAndScalePolygon(basePolygon: HitPoint[], facing: number, scale: number): HitPoint[] {
  const frame = facingToFrame(facing);
  const rotation = frame * ((2 * Math.PI) / NUM_FACINGS);

  return basePolygon.map((point) => ({
    x: Math.round(((point.x * Math.cos(rotation)) - (point.y * Math.sin(rotation))) * scale),
    y: Math.round(((point.x * Math.sin(rotation)) + (point.y * Math.cos(rotation))) * scale),
  }));
}

export function getTextureHitPolygon(texture: TextureLike, scale = 1): HitPoint[] {
  const cached = texturePolygonCache.get(texture.key);
  const polygon = cached ?? buildTexturePolygon(texture);
  if (!cached) {
    texturePolygonCache.set(texture.key, polygon);
  }

  return polygon.map((point) => ({
    x: Math.round(point.x * scale),
    y: Math.round(point.y * scale),
  }));
}

export function buildHitPolygonFromOpaqueRows(rows: OpaqueRow[], width: number, height: number): HitPoint[] {
  if (rows.length === 0) {
    return [];
  }

  const centerX = (width - 1) / 2;
  const centerY = (height - 1) / 2;
  const leftEdge = rows.map((row) => ({
    x: normalizeZero(Math.round(row.left - centerX)),
    y: normalizeZero(Math.round(row.y - centerY)),
  }));
  const rightEdge = rows.toReversed().map((row) => ({
    x: normalizeZero(Math.round(row.right - centerX)),
    y: normalizeZero(Math.round(row.y - centerY)),
  }));

  return dedupePolygonPoints([...leftEdge, ...rightEdge]);
}

function facingToFrame(facing: number): number {
  let angle = facing + Math.PI / 2;
  angle = ((angle % (2 * Math.PI)) + (2 * Math.PI)) % (2 * Math.PI);
  return Math.round(angle / (2 * Math.PI / NUM_FACINGS)) % NUM_FACINGS;
}

function buildTexturePolygon(texture: TextureLike): HitPoint[] {
  const sourceImage = texture.getSourceImage();
  if (typeof document === 'undefined') {
    return [];
  }

  const dimensions = getCanvasImageSourceDimensions(sourceImage);
  if (!dimensions) {
    return [];
  }

  const width = Math.max(1, texture.source[0]?.width ?? dimensions.width);
  const height = Math.max(1, texture.source[0]?.height ?? dimensions.height);
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext('2d', { willReadFrequently: true });
  if (!context) {
    return [];
  }

  context.clearRect(0, 0, width, height);
  context.drawImage(dimensions.image, 0, 0, width, height);
  const imageData = context.getImageData(0, 0, width, height);
  const rows = collectOpaqueRows(imageData.data, width, height);
  return buildHitPolygonFromOpaqueRows(rows, width, height);
}

function collectOpaqueRows(data: Uint8ClampedArray, width: number, height: number): OpaqueRow[] {
  const rows: OpaqueRow[] = [];

  for (let y = 0; y < height; y++) {
    let left = -1;
    let right = -1;

    for (let x = 0; x < width; x++) {
      const alpha = data[((y * width) + x) * 4 + 3];
      if (alpha < 16) {
        continue;
      }

      if (left === -1) {
        left = x;
      }
      right = x;
    }

    if (left !== -1 && right !== -1) {
      rows.push({ y, left, right });
    }
  }

  return rows;
}

function dedupePolygonPoints(points: HitPoint[]): HitPoint[] {
  return points.filter((point, index) => {
    const previous = points[index - 1];
    return !previous || previous.x !== point.x || previous.y !== point.y;
  });
}

function normalizeZero(value: number): number {
  return Object.is(value, -0) ? 0 : value;
}

function getCanvasImageSourceDimensions(sourceImage: unknown): { image: CanvasImageSource; width: number; height: number } | null {
  if (!sourceImage || typeof sourceImage !== 'object') {
    return null;
  }

  const candidate = sourceImage as { width?: unknown; height?: unknown };
  if (typeof candidate.width !== 'number' || typeof candidate.height !== 'number') {
    return null;
  }

  return {
    image: sourceImage as CanvasImageSource,
    width: candidate.width,
    height: candidate.height,
  };
}
