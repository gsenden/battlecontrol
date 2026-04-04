import Phaser from 'phaser';
import { BATTLE_WIDTH, BATTLE_HEIGHT } from '../constants.js';
import { Ship } from '../entities/Ship.js';
import { buildShipPresets } from '../ships/ship-presets.js';
import type { ShipPreset } from '../ships/ship-presets.js';
import { BattleHUD } from '../ui/BattleHUD.js';
import type { HUDShipInfo } from '../ui/BattleHUD.js';
import { getGameLogic } from '../game-logic.js';
import type { GameLogic } from 'game-logic-wasm';
import {
  getCrewBarMax,
  getOtherShipPortraitHeight,
  getShipRenderScale,
  type OtherShipHudState,
} from '../ui/hud-state.svelte.js';
import type { BattleSnapshot, BattleWorkerResponse } from '../workers/battle-worker-types.js';

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
import battleMusic from '../assets/audio/battle-music.ogg';
import humanPrimarySound from '../assets/audio/human-primary.wav';
import humanSpecialSound from '../assets/audio/human-special.ogg';
import humanVictorySound from '../assets/audio/human-victory.ogg';
import battleShipDiesSound from '../assets/audio/battle-shipdies.wav';
import battleBoom45Sound from '../assets/audio/battle-boom-45.wav';
import { velocityToSaturnFrame } from './projectile-frame.js';
import { appendDebugLine, setDebugStatus } from '../debug-overlay.js';

const shipModules = import.meta.glob('../assets/ships/*/*-big-*.png', { eager: true, import: 'default' }) as Record<string, string>;
const battleModules = import.meta.glob('../assets/battle/*.png', { eager: true, import: 'default' }) as Record<string, string>;
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
const SELECTED_SHIP_STORAGE_KEY = 'battlecontrol.selected-ships';
const FLEET_LAYOUT_KEY = Phaser.Input.Keyboard.KeyCodes.T;
const ROTATE_ENEMY_KEY = 106;
const FIRE_ENEMY_WEAPON_KEY = Phaser.Input.Keyboard.KeyCodes.ONE;
const CHMMR_PREFIX = 'chmmr-avatar';
const SYREEN_PREFIX = 'syreen-penetrator';

export class BattleScene extends Phaser.Scene {
  private gameLogic!: GameLogic;
  private shipPresets!: ShipPreset[];
  private playerShip!: Ship;
  private targetShip!: Ship;
  private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
  private plusKey!: Phaser.Input.Keyboard.Key;
  private minusKey!: Phaser.Input.Keyboard.Key;
  private shiftKey!: Phaser.Input.Keyboard.Key;
  private numpadAddKey!: Phaser.Input.Keyboard.Key;
  private numpadSubtractKey!: Phaser.Input.Keyboard.Key;
  private fleetLayoutKey!: Phaser.Input.Keyboard.Key;
  private rotateEnemyKey!: Phaser.Input.Keyboard.Key;
  private fireEnemyWeaponKey!: Phaser.Input.Keyboard.Key;
  private cameraTarget!: Phaser.GameObjects.Container;
  private planet!: Phaser.GameObjects.Image;
  private battleWorker!: Worker;
  private planetBase!: string;
  private planetX = BATTLE_WIDTH / 2;
  private planetY = BATTLE_HEIGHT / 2;
  private shipRenderScale = SHIP_FAR_SCALE;
  private planetRenderScale = PLANET_FAR_SCALE;
  private hud!: BattleHUD;
  private selectedShipIndex = 0;
  private projectileSprites: Phaser.GameObjects.Image[] = [];
  private explosionSprites: Phaser.GameObjects.Image[] = [];
  private laserSprites: Phaser.GameObjects.Line[] = [];
  private battleMusic?: HTMLAudioElement;
  private readonly startBattleMusic = () => {
    if (!this.battleMusic || !this.battleMusic.paused) {
      return;
    }

    void this.battleMusic.play().catch(() => {});
  };
  private currentAllies: OtherShipHudState[] = [];
  private currentOpponents: OtherShipHudState[] = [];
  private targetCaptainName = '';

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

    Object.entries(battleModules).forEach(([path, url]) => {
      const file = path.split('/').pop()!.replace('.png', '');
      this.load.image(`battle-${file}`, url);
    });

    this.load.audio('human-primary', humanPrimarySound);
    this.load.audio('human-special', humanSpecialSound);
    this.load.audio('human-victory', humanVictorySound);
    this.load.audio('battle-shipdies', battleShipDiesSound);
    this.load.audio('battle-boom-45', battleBoom45Sound);

    // Planets
    Object.entries(planetModules).forEach(([path, url]) => {
      const file = path.split('/').pop()!.replace('.png', '');
      this.load.image(`planet-${file}`, url);
    });
  }

  create() {
    this.gameLogic = getGameLogic();
    this.shipPresets = buildShipPresets(this.gameLogic);
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
    const selectedPreset = this.shipPresets[this.selectedShipIndex];
    const targetPreset = this.getPresetBySpritePrefix('human-cruiser');
    this.targetCaptainName = Phaser.Utils.Array.GetRandom(targetPreset.stats.captainNames);
    this.playerShip = new Ship(this, this.planetX + 800, this.planetY, selectedPreset.stats);
    this.targetShip = new Ship(this, this.planetX + 2600, this.planetY - 500, targetPreset.stats);
    this.targetShip.setTint(0xff8866);
    this.battleWorker = new Worker(new URL('../workers/battle-worker.ts', import.meta.url), { type: 'module' });
    this.battleWorker.onmessage = (event: MessageEvent<BattleWorkerResponse>) => {
      if (event.data.type !== 'snapshot') {
        return;
      }

      this.applySnapshot(event.data.snapshot);
    };
    this.battleWorker.postMessage({
      type: 'initBattle',
      playerShipType: selectedPreset.stats.spritePrefix,
      targetShipType: targetPreset.stats.spritePrefix,
      playerX: this.playerShip.x,
      playerY: this.playerShip.y,
      targetX: this.targetShip.x,
      targetY: this.targetShip.y,
      planetX: this.planetX,
      planetY: this.planetY,
      width: BATTLE_WIDTH,
      height: BATTLE_HEIGHT,
    });
    this.battleMusic = new Audio(battleMusic);
    this.battleMusic.loop = true;
    this.battleMusic.volume = 0.45;
    this.battleMusic.preload = 'auto';
    window.addEventListener('battlecontrol:start-game', this.startBattleMusic);
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
      this.battleWorker.terminate();
      window.removeEventListener('battlecontrol:start-game', this.startBattleMusic);
      if (this.battleMusic) {
        this.battleMusic.pause();
        this.battleMusic.currentTime = 0;
      }
    });

    // Camera follows an invisible container that tracks the ships position
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
    this.rotateEnemyKey = this.input.keyboard!.addKey(ROTATE_ENEMY_KEY);
    this.fireEnemyWeaponKey = this.input.keyboard!.addKey(FIRE_ENEMY_WEAPON_KEY);
    this.input.mouse?.disableContextMenu();
    this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      const x = Math.round(pointer.worldX);
      const y = Math.round(pointer.worldY);
      const targetType = this.targetShip.containsPoint(pointer.worldX, pointer.worldY)
        ? 'ship'
        : 'point';
      if (targetType === 'ship') {
        this.battleWorker.postMessage({ type: 'setPlayerWeaponTargetShip' });
      } else {
        this.battleWorker.postMessage({ type: 'setPlayerWeaponTargetPoint', x, y });
      }
      appendDebugLine(`${targetType} x=${x} y=${y}`);
    });

    window.dispatchEvent(new Event('battlecontrol:scene-ready'));
  }

  update() {
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

    if (Phaser.Input.Keyboard.JustDown(this.fireEnemyWeaponKey)) {
      this.battleWorker.postMessage({ type: 'triggerTargetWeapon' });
    }

    this.battleWorker.postMessage({ type: 'setPlayerInput', input: hudInput });
    this.battleWorker.postMessage({
      type: 'setTargetInput',
      input: {
        left: false,
        right: this.rotateEnemyKey.isDown,
        thrust: false,
        weapon: false,
        special: false,
      },
    });

    // Visual update every render frame
    this.updateCamera();
    this.updatePlanetRender();
    this.playerShip.renderUpdate(this.shipRenderScale);
    this.targetShip.renderUpdate(this.shipRenderScale);
    this.hud.update(hudInput);
  }

  private updateCamera() {
    if (this.targetShip.dead) {
      this.cameraTarget.setPosition(this.playerShip.x, this.playerShip.y);
      return;
    }

    const targetX = this.targetShip.x;
    const targetY = this.targetShip.y;
    const dx = targetX - this.playerShip.x;
    const dy = targetY - this.playerShip.y;
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
    this.selectedShipIndex = (this.selectedShipIndex + direction + this.shipPresets.length) % this.shipPresets.length;
    const currentX = this.playerShip.x;
    const currentY = this.playerShip.y;

    this.playerShip.destroy();
    const preset = this.shipPresets[this.selectedShipIndex];
    this.playerShip = new Ship(this, currentX, currentY, preset.stats);
    this.battleWorker.postMessage({ type: 'switchPlayerShip', shipType: preset.stats.spritePrefix });
    this.cameraTarget.setPosition(this.playerShip.x, this.playerShip.y);
    this.saveSelectedShip();
    this.syncHudWithSelectedShip();
  }

  private applySnapshot(snapshot: BattleSnapshot) {
    this.playerShip.applySnapshot(snapshot.player);
    this.targetShip.applySnapshot(snapshot.target);
    this.syncProjectiles(snapshot);
    this.syncExplosions(snapshot);
    this.syncLasers(snapshot);
    this.syncTargetOpponent(snapshot);
    this.playAudioEvents(snapshot);
    setDebugStatus(
      snapshot.projectiles[0]
        ? `rocket life=${snapshot.projectiles[0].life}`
        : '',
    );
  }

  private playAudioEvents(snapshot: BattleSnapshot) {
    for (const event of snapshot.audioEvents) {
      switch (event.key) {
        case 'human-primary':
          this.sound.play(event.key, { volume: 0.5 });
          break;
        case 'human-special':
          this.sound.play(event.key, { volume: 0.6 });
          break;
        case 'human-victory':
          this.sound.play(event.key, { volume: 0.65 });
          break;
        case 'battle-shipdies':
          this.sound.play(event.key, { volume: 0.75 });
          break;
        case 'battle-boom-45':
          this.sound.play(event.key, { volume: 0.55 });
          break;
        default:
          break;
      }
    }
  }

  private syncProjectiles(snapshot: BattleSnapshot) {
    while (this.projectileSprites.length < snapshot.projectiles.length) {
      const projectile = this.add.image(0, 0, 'ion-particle');
      projectile.setDepth(10);
      this.projectileSprites.push(projectile);
    }

    while (this.projectileSprites.length > snapshot.projectiles.length) {
      this.projectileSprites.pop()?.destroy();
    }

    snapshot.projectiles.forEach((projectile, index) => {
      const sprite = this.projectileSprites[index];
      sprite.setPosition(projectile.x, projectile.y);

      if (projectile.texturePrefix) {
        const frameIndex = projectile.texturePrefix === 'human-saturn'
          ? velocityToSaturnFrame(projectile.vx, projectile.vy)
          : this.velocityToProjectileFrame(projectile.vx, projectile.vy, 25);
        sprite.setTexture(`${projectile.texturePrefix}-big-${String(frameIndex).padStart(3, '0')}`);
        sprite.clearTint();
        sprite.setScale(0.75);
      } else {
        sprite.setTexture('ion-particle');
        sprite.setTint(0xffffff);
        sprite.setScale(2);
      }
    });
  }

  private syncExplosions(snapshot: BattleSnapshot) {
    while (this.explosionSprites.length < snapshot.explosions.length) {
      const explosion = this.add.image(0, 0, 'shofixti-destruct-big-000');
      explosion.setDepth(11);
      this.explosionSprites.push(explosion);
    }

    while (this.explosionSprites.length > snapshot.explosions.length) {
      this.explosionSprites.pop()?.destroy();
    }

    snapshot.explosions.forEach((explosion, index) => {
      const sprite = this.explosionSprites[index];
      sprite.setPosition(explosion.x, explosion.y);
      sprite.setTexture(
        `${explosion.texturePrefix}-big-${String(explosion.frameIndex).padStart(3, '0')}`,
      );
      sprite.setScale(explosion.texturePrefix === 'battle-boom' ? 1 : 0.75);
    });
  }

  private syncLasers(snapshot: BattleSnapshot) {
    while (this.laserSprites.length < snapshot.lasers.length) {
      const laser = this.add.line(0, 0, 0, 0, 0, 0, 0xffffff, 0.95);
      laser.setLineWidth(3, 3);
      laser.setDepth(12);
      this.laserSprites.push(laser);
    }

    while (this.laserSprites.length > snapshot.lasers.length) {
      this.laserSprites.pop()?.destroy();
    }

    snapshot.lasers.forEach((laser, index) => {
      const line = this.laserSprites[index];
      line.setTo(laser.startX, laser.startY, laser.endX, laser.endY);
      line.setOrigin(0, 0);
    });
  }

  private velocityToProjectileFrame(vx: number, vy: number, frameCount: number) {
    let angle = Math.atan2(vy, vx) + Math.PI / 2;
    angle = ((angle % (2 * Math.PI)) + (2 * Math.PI)) % (2 * Math.PI);
    return Math.round(angle / (2 * Math.PI / frameCount)) % frameCount;
  }

  private getPresetBySpritePrefix(spritePrefix: string): ShipPreset {
    const preset = this.shipPresets.find((candidate) => candidate.stats.spritePrefix === spritePrefix);
    if (!preset) {
      throw new Error(`Missing ship preset for ${spritePrefix}`);
    }

    return preset;
  }

  private syncHudWithSelectedShip() {
    const preset = this.shipPresets[this.selectedShipIndex];
    this.hud.setShip({
      ship: this.playerShip,
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
    const [allies, opponents] = this.createRandomFleet(7, 7);
    const targetPreset = this.getPresetBySpritePrefix('human-cruiser');
    const targetOpponent = this.toOtherShipHudState(targetPreset, 'target-0', this.targetCaptainName);
    this.currentAllies = allies;
    this.currentOpponents = [targetOpponent, ...opponents];
    this.hud.setFleet(this.currentAllies, this.currentOpponents);
  }

  private createRandomFleet(allyCount: number, opponentCount: number): [OtherShipHudState[], OtherShipHudState[]] {
    const playerShipPrefix = this.shipPresets[this.selectedShipIndex].stats.spritePrefix;
    const targetShipPrefix = this.targetShip.stats.spritePrefix;
    const availablePresets = this.shipPresets.filter((preset) => (
      preset.stats.spritePrefix !== playerShipPrefix
      && preset.stats.spritePrefix !== targetShipPrefix
    ));
    const availableByPrefix = new Map(availablePresets.map((preset) => [preset.stats.spritePrefix, preset]));
    const usedPrefixes = new Set<string>();

    return [
      this.buildFleetSide('ally', allyCount, availablePresets, availableByPrefix, usedPrefixes),
      this.buildFleetSide('opponent', opponentCount, availablePresets, availableByPrefix, usedPrefixes),
    ];
  }

  private buildFleetSide(
    side: 'ally' | 'opponent',
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

    return selected.map((preset, index) => this.toOtherShipHudState(preset, `${side}-${index}`));
  }

  private toOtherShipHudState(preset: ShipPreset, id: string, captainName = Phaser.Utils.Array.GetRandom(preset.stats.captainNames)): OtherShipHudState {
    return {
      id,
      portraitUrl: preset.portraitUrl,
      portraitHeight: getOtherShipPortraitHeight(preset.stats.size),
      renderScale: preset.stats.renderScale ?? getShipRenderScale(preset.stats.size),
      captainName,
      shipName: preset.stats.raceName,
      crew: preset.stats.maxCrew,
      maxCrew: preset.stats.maxCrew,
      crewBarMax: getCrewBarMax(preset.stats.spritePrefix, preset.stats.maxCrew),
      energy: preset.stats.maxEnergy,
      maxEnergy: preset.stats.maxEnergy,
      energyBarMax: preset.stats.maxEnergy,
      dead: false,
    };
  }

  private syncTargetOpponent(snapshot: BattleSnapshot) {
    if (this.currentOpponents.length === 0) {
      return;
    }

    const [targetOpponent, ...rest] = this.currentOpponents;
    this.currentOpponents = [{
      ...targetOpponent,
      crew: snapshot.target.crew,
      energy: snapshot.target.energy,
      dead: snapshot.target.dead,
    }, ...rest];
    this.hud.setFleet(this.currentAllies, this.currentOpponents);
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

      const index = this.shipPresets.findIndex((preset) => preset.stats.spritePrefix === storedSpritePrefix);
      return index >= 0 ? index : 0;
    } catch {
      return 0;
    }
  }

  private saveSelectedShip() {
    try {
      window.localStorage.setItem(
        SELECTED_SHIP_STORAGE_KEY,
        this.shipPresets[this.selectedShipIndex].stats.spritePrefix,
      );
    } catch {
      // Ignore storage failures and keep the current in-memory selection.
    }
  }
}
