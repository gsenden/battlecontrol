import Phaser from 'phaser';
import { BATTLE_WIDTH, BATTLE_HEIGHT, PHYSICS_DELTA } from '../constants.js';
import { Ship } from '../entities/Ship.js';
import { HUMAN_CRUISER } from '../ships/human/human-cruiser-stats.js';
import { SHIP_PRESETS } from '../ships/ship-presets.js';
import { shortestWrappedDelta } from '../utils/wrap.js';
import { BattleHUD } from '../ui/BattleHUD.js';
import type { HUDShipInfo } from '../ui/BattleHUD.js';
import {
  getCrewBarMax,
  getOtherShipPortraitHeight,
  getShipRenderScale,
  type OtherShipHudState,
} from '../ui/hud-state.svelte.js';
import type { ShipPreset } from '../ships/ship-presets.js';

import starBig from '../assets/stars/stars-000.png';
import starMed from '../assets/stars/stars-001.png';
import starSml from '../assets/stars/stars-002.png';
import starSml2 from '../assets/stars/stars-003.png';
import starSml3 from '../assets/stars/stars-004.png';
import starMiscBig0 from '../assets/stars/stars-misc-big-000.png';
import starMiscBig1 from '../assets/stars/stars-misc-big-001.png';
import starMiscMed0 from '../assets/stars/stars-misc-med-000.png';
import starMiscMed1 from '../assets/stars/stars-misc-med-001.png';
import starMiscSml0 from '../assets/stars/stars-misc-sml-000.png';
import starMiscSml1 from '../assets/stars/stars-misc-sml-001.png';

const shipModules = import.meta.glob('../assets/ships/*/*-big-*.png', { eager: true, import: 'default' }) as Record<string, string>;
const planetModules = import.meta.glob('../assets/planets/*.png', { eager: true, import: 'default' }) as Record<string, string>;
const planetBases = Object.keys(planetModules)
  .filter((path) => path.endsWith('-big-000.png'))
  .map((path) => path.split('/').pop()!.replace('-big-000.png', ''));

const BIG_STAR_COUNT = 200;
const MED_STAR_COUNT = 400;
const SML_STAR_COUNT = 600;

const BIG_SCROLL_FACTOR = 0.25;
const MED_SCROLL_FACTOR = 0.125;
const SML_SCROLL_FACTOR = 0.0625;

// SC2 gravity well: fixed-radius well with a constant velocity pull each tick.
const GRAVITY_THRESHOLD = 420;
const GRAVITY_PULL = 0.12;
const DEFAULT_CAMERA_ZOOM = 0.72;
const MIN_CAMERA_ZOOM = 0.52;
const MAX_CAMERA_ZOOM = 0.9;
const ZOOM_LERP = 0.035;
const ZOOM_MAX_RADIUS = 500;
const ZOOM_START_RADIUS = 1100;
const PLANET_RENDER_SCALE = 1.15;
const SHIP_NEAR_SCALE = 1.05;
const SHIP_FAR_SCALE = 0.68;
const PLANET_NEAR_SCALE = 1.15;
const PLANET_FAR_SCALE = 0.72;
const SELECTED_SHIP_STORAGE_KEY = 'battlecontrol.selected-ship';
const FLEET_LAYOUT_KEY = Phaser.Input.Keyboard.KeyCodes.T;
const CHMMR_PREFIX = 'chmmr-avatar';
const SYREEN_PREFIX = 'syreen-penetrator';

export class BattleScene extends Phaser.Scene {
  private playerShip!: Ship;
  private targetShip!: Ship;
  private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
  private plusKey!: Phaser.Input.Keyboard.Key;
  private minusKey!: Phaser.Input.Keyboard.Key;
  private shiftKey!: Phaser.Input.Keyboard.Key;
  private numpadAddKey!: Phaser.Input.Keyboard.Key;
  private numpadSubtractKey!: Phaser.Input.Keyboard.Key;
  private fleetLayoutKey!: Phaser.Input.Keyboard.Key;
  private cameraTarget!: Phaser.GameObjects.Container;
  private planet!: Phaser.GameObjects.Image;
  private planetBase!: string;
  private planetX = BATTLE_WIDTH / 2;
  private planetY = BATTLE_HEIGHT / 2;
  private physicsAccumulator = 0;
  private shipRenderScale = SHIP_FAR_SCALE;
  private planetRenderScale = PLANET_FAR_SCALE;
  private hud!: BattleHUD;
  private selectedShipIndex = 0;

  constructor() {
    super('BattleScene');
  }

  preload() {
    // Stars
    this.load.image('star-big', starBig);
    this.load.image('star-med', starMed);
    this.load.image('star-sml', starSml);
    this.load.image('star-sml2', starSml2);
    this.load.image('star-sml3', starSml3);
    this.load.image('star-misc-big-0', starMiscBig0);
    this.load.image('star-misc-big-1', starMiscBig1);
    this.load.image('star-misc-med-0', starMiscMed0);
    this.load.image('star-misc-med-1', starMiscMed1);
    this.load.image('star-misc-sml-0', starMiscSml0);
    this.load.image('star-misc-sml-1', starMiscSml1);

    // Ship frames (3 zoom tiers x 16 rotations)
    const shipEntries = Object.entries(shipModules).sort(([a], [b]) => a.localeCompare(b));
    shipEntries.forEach(([path, url]) => {
      const file = path.split('/').pop()!.replace('.png', '');
      const folder = path.split('/').at(-2)!;
      this.load.image(`${folder}-${file}`, url);
    });

    // Planets
    Object.entries(planetModules).forEach(([path, url]) => {
      const file = path.split('/').pop()!.replace('.png', '');
      this.load.image(`planet-${file}`, url);
    });
  }

  create() {
    this.selectedShipIndex = this.loadSelectedShipIndex();

    // Stars
    this.createStarLayer(
      ['star-big', 'star-misc-big-0', 'star-misc-big-1'],
      BIG_STAR_COUNT, BIG_SCROLL_FACTOR,
    );
    this.createStarLayer(
      ['star-med', 'star-misc-med-0', 'star-misc-med-1'],
      MED_STAR_COUNT, MED_SCROLL_FACTOR,
    );
    this.createStarLayer(
      ['star-sml', 'star-sml2', 'star-sml3', 'star-misc-sml-0', 'star-misc-sml-1'],
      SML_STAR_COUNT, SML_SCROLL_FACTOR,
    );

    // Planet in center
    this.planetBase = planetBases[Math.floor(Math.random() * planetBases.length)];
    this.planet = this.add.image(this.planetX, this.planetY, `planet-${this.planetBase}-big-000`);

    // Ion trail particle texture (small orange circle, generated)
    const gfx = this.add.graphics();
    gfx.fillStyle(0xffffff);
    gfx.fillCircle(2, 2, 2);
    gfx.generateTexture('ion-particle', 4, 4);
    gfx.destroy();

    // Ship — spawn offset from center so it's not on the planet
    this.playerShip = new Ship(this, this.planetX + 800, this.planetY, SHIP_PRESETS[this.selectedShipIndex].stats);
    this.targetShip = new Ship(this, this.planetX + 2600, this.planetY - 500, HUMAN_CRUISER);
    this.targetShip.setTint(0xff8866);

    // Camera follows an invisible container that tracks the ship position
    this.cameraTarget = this.add.container(this.playerShip.x, this.playerShip.y);
    this.cameras.main.setBounds(0, 0, BATTLE_WIDTH, BATTLE_HEIGHT);
    this.cameras.main.setZoom(DEFAULT_CAMERA_ZOOM);
    this.cameras.main.startFollow(this.cameraTarget);

    // HUD
    this.hud = new BattleHUD();
    this.syncHudWithSelectedShip();

    // Input
    this.cursors = this.input.keyboard!.createCursorKeys();
    this.plusKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.PLUS);
    this.minusKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.MINUS);
    this.shiftKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.SHIFT);
    this.numpadAddKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.NUMPAD_ADD);
    this.numpadSubtractKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.NUMPAD_SUBTRACT);
    this.fleetLayoutKey = this.input.keyboard!.addKey(FLEET_LAYOUT_KEY);
    this.input.mouse?.disableContextMenu();
  }

  update(_time: number, delta: number) {
    const input = {
      left: this.cursors.left.isDown,
      right: this.cursors.right.isDown,
      thrust: this.cursors.up.isDown,
      weapon: false,
      special: false,
    };
    const pointer = this.input.activePointer;
    const hudInput = {
      ...input,
      weapon: pointer.leftButtonDown(),
      special: pointer.rightButtonDown(),
    };

    if (
      Phaser.Input.Keyboard.JustDown(this.numpadAddKey)
      || (Phaser.Input.Keyboard.JustDown(this.plusKey) && this.shiftKey.isDown)
    ) {
      this.cyclePlayerShip(1);
    }

    if (
      Phaser.Input.Keyboard.JustDown(this.numpadSubtractKey)
      || Phaser.Input.Keyboard.JustDown(this.minusKey)
    ) {
      this.cyclePlayerShip(-1);
    }

    if (Phaser.Input.Keyboard.JustDown(this.fleetLayoutKey)) {
      this.toggleFleetLayout();
    }

    // Fixed timestep physics at 24fps
    this.physicsAccumulator += delta;
    while (this.physicsAccumulator >= PHYSICS_DELTA) {
      const playerInGravityWell = this.applyGravity(this.playerShip);
      this.playerShip.physicsUpdate(input, playerInGravityWell);
      this.playerShip.wrapPosition();
      this.physicsAccumulator -= PHYSICS_DELTA;
    }

    // Visual update every render frame
    this.updateCamera();
    this.updatePlanetRender();
    this.playerShip.renderUpdate(this.shipRenderScale);
    this.targetShip.renderUpdate(this.shipRenderScale);
    this.hud.update(hudInput);
  }

  private applyGravity(ship: Ship): boolean {
    const dx = shortestWrappedDelta(ship.x, this.planetX, BATTLE_WIDTH);
    const dy = shortestWrappedDelta(ship.y, this.planetY, BATTLE_HEIGHT);
    const absDx = Math.abs(dx);
    const absDy = Math.abs(dy);

    if (absDx <= GRAVITY_THRESHOLD && absDy <= GRAVITY_THRESHOLD) {
      const distSquared = (absDx * absDx) + (absDy * absDy);
      if (distSquared <= GRAVITY_THRESHOLD * GRAVITY_THRESHOLD && distSquared > 0) {
        const dist = Math.sqrt(distSquared);
        ship.addVelocity((dx / dist) * GRAVITY_PULL, (dy / dist) * GRAVITY_PULL);
        return true;
      }
    }

    return false;
  }

  private updateCamera() {
    const dx = this.targetShip.x - this.playerShip.x;
    const dy = this.targetShip.y - this.playerShip.y;
    const distance = Math.sqrt((dx * dx) + (dy * dy));

    this.cameraTarget.setPosition(this.playerShip.x, this.playerShip.y);

    const distanceT = Phaser.Math.Clamp(
      (distance - ZOOM_MAX_RADIUS) / (ZOOM_START_RADIUS - ZOOM_MAX_RADIUS),
      0,
      1,
    );
    const easedT = distanceT * distanceT * (3 - (2 * distanceT));
    const targetZoom = Phaser.Math.Linear(MAX_CAMERA_ZOOM, MIN_CAMERA_ZOOM, easedT);
    this.shipRenderScale = Phaser.Math.Linear(SHIP_NEAR_SCALE, SHIP_FAR_SCALE, easedT);
    this.planetRenderScale = Phaser.Math.Linear(PLANET_NEAR_SCALE, PLANET_FAR_SCALE, easedT);
    this.cameras.main.setZoom(
      Phaser.Math.Linear(this.cameras.main.zoom, targetZoom, ZOOM_LERP),
    );
  }

  private updatePlanetRender() {
    this.planet.setTexture(`planet-${this.planetBase}-big-000`);
    this.planet.setScale(PLANET_RENDER_SCALE * this.planetRenderScale);
  }

  private cyclePlayerShip(direction: 1 | -1) {
    this.selectedShipIndex = (this.selectedShipIndex + direction + SHIP_PRESETS.length) % SHIP_PRESETS.length;
    const currentX = this.playerShip.x;
    const currentY = this.playerShip.y;
    const currentVelocity = { ...this.playerShip.body.velocity };

    this.playerShip.destroy();
    this.playerShip = new Ship(this, currentX, currentY, SHIP_PRESETS[this.selectedShipIndex].stats);
    this.matter.body.setVelocity(this.playerShip.body, currentVelocity);
    this.cameraTarget.setPosition(this.playerShip.x, this.playerShip.y);
    this.saveSelectedShip();
    this.syncHudWithSelectedShip();
  }

  private syncHudWithSelectedShip() {
    const preset = SHIP_PRESETS[this.selectedShipIndex];
    this.hud.setShip({
      state: this.playerShip.state,
      stats: preset.stats,
      portraitUrl: preset.portraitUrl,
      captainFrameUrls: preset.captainFrameUrls,
      captainFrameStyles: preset.captainFrameStyles,
      captainLayout: preset.captainLayout,
      captainName: preset.stats.captainNames[Math.floor(Math.random() * preset.stats.captainNames.length)],
    } satisfies HUDShipInfo);
    this.syncFleetLayout();
  }

  private toggleFleetLayout() {
    this.syncFleetLayout();
  }

  private syncFleetLayout() {
    const [allies, opponents] = this.createRandomFleet(7, 8);
    this.hud.setFleet(allies, opponents);
  }

  private createRandomFleet(allyCount: number, opponentCount: number): [OtherShipHudState[], OtherShipHudState[]] {
    const playerShipPrefix = SHIP_PRESETS[this.selectedShipIndex].stats.spritePrefix;
    const availablePresets = SHIP_PRESETS.filter((preset) => preset.stats.spritePrefix !== playerShipPrefix);
    const availableByPrefix = new Map(availablePresets.map((preset) => [preset.stats.spritePrefix, preset]));
    const usedPrefixes = new Set<string>();

    return [
      this.buildFleetSide(allyCount, availablePresets, availableByPrefix, usedPrefixes),
      this.buildFleetSide(opponentCount, availablePresets, availableByPrefix, usedPrefixes),
    ];
  }

  private buildFleetSide(
    count: number,
    availablePresets: ShipPreset[],
    availableByPrefix: Map<string, ShipPreset>,
    usedPrefixes: Set<string>,
  ): OtherShipHudState[] {
    const selected: ShipPreset[] = [];

    for (const requiredPrefix of [CHMMR_PREFIX, SYREEN_PREFIX]) {
      if (selected.length >= count) break;

      const requiredPreset = availableByPrefix.get(requiredPrefix);
      if (!requiredPreset) continue;

      selected.push(requiredPreset);
    }

    const remaining = Phaser.Utils.Array.Shuffle(
      availablePresets.filter((preset) => {
        if (preset.stats.spritePrefix === CHMMR_PREFIX || preset.stats.spritePrefix === SYREEN_PREFIX) {
          return !selected.some((selectedPreset) => selectedPreset.stats.spritePrefix === preset.stats.spritePrefix);
        }

        return !usedPrefixes.has(preset.stats.spritePrefix);
      }),
    );

    for (const preset of remaining) {
      if (selected.length >= count) break;
      selected.push(preset);
    }

    for (const preset of selected) {
      if (preset.stats.spritePrefix === CHMMR_PREFIX || preset.stats.spritePrefix === SYREEN_PREFIX) continue;
      usedPrefixes.add(preset.stats.spritePrefix);
    }

    return selected.map((preset, index) => this.toOtherShipHudState(preset, `ally-${index}`));
  }

  private toOtherShipHudState(preset: ShipPreset, id: string): OtherShipHudState {
    return {
      id,
      portraitUrl: preset.portraitUrl,
      portraitHeight: getOtherShipPortraitHeight(preset.stats.size),
      renderScale: preset.stats.renderScale ?? getShipRenderScale(preset.stats.size),
      captainName: Phaser.Utils.Array.GetRandom(preset.stats.captainNames),
      shipName: preset.stats.raceName,
      crew: preset.stats.maxCrew,
      maxCrew: preset.stats.maxCrew,
      crewBarMax: getCrewBarMax(preset.stats.spritePrefix, preset.stats.maxCrew),
      energy: preset.stats.maxEnergy,
      maxEnergy: preset.stats.maxEnergy,
      energyBarMax: preset.stats.maxEnergy,
    };
  }

  private createStarLayer(textures: string[], count: number, scrollFactor: number) {
    // Parallax layers move less than the gameplay world, so they need extra bleed
    // outside the arena or the camera can reveal empty edges.
    const viewportWidth = this.scale.width / MIN_CAMERA_ZOOM;
    const viewportHeight = this.scale.height / MIN_CAMERA_ZOOM;
    const padX = ((1 - scrollFactor) * (BATTLE_WIDTH + viewportWidth)) / 2;
    const padY = ((1 - scrollFactor) * (BATTLE_HEIGHT + viewportHeight)) / 2;
    const spawnWidth = BATTLE_WIDTH + (2 * padX);
    const spawnHeight = BATTLE_HEIGHT + (2 * padY);
    const densityScale = (spawnWidth * spawnHeight) / (BATTLE_WIDTH * BATTLE_HEIGHT);
    const spawnCount = Math.ceil(count * densityScale);

    for (let i = 0; i < spawnCount; i++) {
      const texture = textures[Math.floor(Math.random() * textures.length)];
      const x = (-padX) + (Math.random() * spawnWidth);
      const y = (-padY) + (Math.random() * spawnHeight);
      const star = this.add.image(x, y, texture);
      star.setScrollFactor(scrollFactor);
    }
  }

  private loadSelectedShipIndex() {
    try {
      const storedSpritePrefix = window.localStorage.getItem(SELECTED_SHIP_STORAGE_KEY);
      if (!storedSpritePrefix) {
        return 0;
      }

      const index = SHIP_PRESETS.findIndex((preset) => preset.stats.spritePrefix === storedSpritePrefix);
      return index >= 0 ? index : 0;
    } catch {
      return 0;
    }
  }

  private saveSelectedShip() {
    try {
      window.localStorage.setItem(
        SELECTED_SHIP_STORAGE_KEY,
        SHIP_PRESETS[this.selectedShipIndex].stats.spritePrefix,
      );
    } catch {
      // Ignore storage failures and keep the current in-memory selection.
    }
  }
}
