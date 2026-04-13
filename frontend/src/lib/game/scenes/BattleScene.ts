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
import androsynthPrimarySound from '../assets/audio/androsynth-primary.wav';
import androsynthSpecialSound from '../assets/audio/androsynth-special.wav';
import arilouPrimarySound from '../assets/audio/arilou-primary.wav';
import arilouSpecialSound from '../assets/audio/arilou-special.wav';
import arilouVictorySound from '../assets/audio/arilou-victory.ogg';
import humanPrimarySound from '../assets/audio/human-primary.wav';
import humanSpecialSound from '../assets/audio/human-special.ogg';
import humanVictorySound from '../assets/audio/human-victory.ogg';
import yehatPrimarySound from '../assets/audio/yehat-primary.wav';
import yehatSpecialSound from '../assets/audio/yehat-special.wav';
import battleShipDiesSound from '../assets/audio/battle-shipdies.wav';
import battleBoom23Sound from '../assets/audio/battle-boom-23.wav';
import battleBoom45Sound from '../assets/audio/battle-boom-45.wav';
import { projectileFrameFor } from './projectile-frame.js';
import { getShipHitPolygon, getTextureHitPolygon } from './hit-polygon.js';
import {
  appendDebugLine,
  appendDebugTrace,
  clearDebugTrace,
  isDebugUiEnabled,
  setDebugStatus,
} from '../debug-overlay.js';
import type { UserSettingsDto } from '$lib/auth/auth.js';
import type { BattleSetup } from '../boot.js';

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
const FIRE_ENEMY_WEAPON_KEY = Phaser.Input.Keyboard.KeyCodes.R;
const FIRE_ENEMY_WEAPON_ALT_KEY = Phaser.Input.Keyboard.KeyCodes.ONE;
const CHMMR_PREFIX = 'chmmr-avatar';
const SYREEN_PREFIX = 'syreen-penetrator';
const DEFAULT_USER_SETTINGS: UserSettingsDto = {
  turn_left_key: 'A',
  turn_right_key: 'D',
  thrust_key: 'W',
  music_enabled: true,
  music_volume: 45,
  sound_effects_enabled: true,
  sound_effects_volume: 60,
};

export class BattleScene extends Phaser.Scene {
  private gameLogic!: GameLogic;
  private shipPresets!: ShipPreset[];
  private playerShip!: Ship;
  private targetShip!: Ship;
  private playerHitPolygon!: Phaser.GameObjects.Graphics;
  private targetHitPolygon!: Phaser.GameObjects.Graphics;
  private moveLeftKey!: Phaser.Input.Keyboard.Key;
  private moveRightKey!: Phaser.Input.Keyboard.Key;
  private thrustKey!: Phaser.Input.Keyboard.Key;
  private plusKey!: Phaser.Input.Keyboard.Key;
  private minusKey!: Phaser.Input.Keyboard.Key;
  private shiftKey!: Phaser.Input.Keyboard.Key;
  private numpadAddKey!: Phaser.Input.Keyboard.Key;
  private numpadSubtractKey!: Phaser.Input.Keyboard.Key;
  private fleetLayoutKey!: Phaser.Input.Keyboard.Key;
  private rotateEnemyKey!: Phaser.Input.Keyboard.Key;
  private fireEnemyWeaponKey!: Phaser.Input.Keyboard.Key;
  private fireEnemyWeaponAltKey!: Phaser.Input.Keyboard.Key;
  private cameraTarget!: Phaser.GameObjects.Container;
  private planet!: Phaser.GameObjects.Image;
  private battleWorker?: Worker;
  private battleSocket: WebSocket | null = null;
  private battleSocketFailed = false;
  private planetBase!: string;
  private planetX = BATTLE_WIDTH / 2;
  private planetY = BATTLE_HEIGHT / 2;
  private shipRenderScale = SHIP_FAR_SCALE;
  private planetRenderScale = PLANET_FAR_SCALE;
  private hud!: BattleHUD;
  private selectedShipIndex = 0;
  private projectileSprites: Phaser.GameObjects.Image[] = [];
  private meteorSprites: Phaser.GameObjects.Image[] = [];
  private explosionSprites: Phaser.GameObjects.Image[] = [];
  private laserSprites: Phaser.GameObjects.Line[] = [];
  private battleMusic?: HTMLAudioElement;
  private started = false;
  private readonly startBattleMusic = () => {
    this.started = true;

    if (!this.battleMusic || !this.battleMusic.paused || this.battleMusic.volume <= 0) {
      return;
    }

    void this.battleMusic.play().catch(() => {});
  };
  private currentAllies: OtherShipHudState[] = [];
  private currentOpponents: OtherShipHudState[] = [];
  private targetCaptainName = '';
  private playerCaptainName = '';
  private projectileHitPolygons: Phaser.GameObjects.Graphics[] = [];
  private loggedExplosionKeys = new Set<string>();
  private readonly sfxChannels = new Map<string, Phaser.Sound.BaseSound>();
  private readonly sfxLastPlayedAtMs = new Map<string, number>();
  private readonly sfxPatternStep = new Map<string, number>();
  private projectileTraceFrame = 0;
  private projectileTraceStopped = false;
  private lastWeaponDown = false;
  private finishedEventSent = false;

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
      if (file.startsWith('boom-big-')) {
        this.load.image(`battle-blast-big-${file.slice('boom-big-'.length)}`, url);
      }
    });

    this.load.audio('androsynth-primary', androsynthPrimarySound);
    this.load.audio('yehat-primary', yehatPrimarySound);
    this.load.audio('yehat-special', yehatSpecialSound);
    this.load.audio('androsynth-special', androsynthSpecialSound);
    this.load.audio('arilou-primary', arilouPrimarySound);
    this.load.audio('arilou-special', arilouSpecialSound);
    this.load.audio('arilou-victory', arilouVictorySound);
    this.load.audio('human-primary', humanPrimarySound);
    this.load.audio('human-special', humanSpecialSound);
    this.load.audio('human-victory', humanVictorySound);
    this.load.audio('battle-shipdies', battleShipDiesSound);
    this.load.audio('battle-boom-23', battleBoom23Sound);
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
    const userSettings = (this.game.registry.get('userSettings') as UserSettingsDto | undefined) ?? DEFAULT_USER_SETTINGS;

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
    const battleSetup = this.game.registry.get('battleSetup') as BattleSetup | undefined;
    const selectedPreset = battleSetup
      ? this.getPresetBySpritePrefix(battleSetup.playerShipType)
      : this.shipPresets[this.selectedShipIndex];
    const targetPreset = this.getPresetBySpritePrefix(battleSetup?.targetShipType ?? 'human-cruiser');
    this.playerCaptainName = battleSetup?.playerCaptainName
      ?? selectedPreset.stats.captainNames[Math.floor(Math.random() * selectedPreset.stats.captainNames.length)];
    this.targetCaptainName = battleSetup?.opponents?.[0]?.captainName
      ?? Phaser.Utils.Array.GetRandom(targetPreset.stats.captainNames);
    this.playerShip = new Ship(this, this.planetX + 800, this.planetY, selectedPreset.stats);
    this.targetShip = new Ship(this, this.planetX + 2600, this.planetY - 500, targetPreset.stats);
    this.targetShip.setTint(0xff8866);
    this.playerHitPolygon = this.add.graphics();
    this.playerHitPolygon.setDepth(20);
    this.targetHitPolygon = this.add.graphics();
    this.targetHitPolygon.setDepth(20);
    if (battleSetup?.gameId) {
      this.started = true;
      this.connectBattleSocket(battleSetup.gameId);
    } else {
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
    }
    this.battleMusic = new Audio(battleMusic);
    this.battleMusic.loop = true;
    this.battleMusic.volume = userSettings.music_enabled ? userSettings.music_volume / 100 : 0;
    this.battleMusic.preload = 'auto';
    window.addEventListener('battlecontrol:start-game', this.startBattleMusic);
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
      this.battleWorker?.terminate();
      this.battleSocket?.close();
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
    const hudElement = this.game.registry.get('hudElement') as HTMLElement;
    this.hud = new BattleHUD(hudElement);
    this.syncHudWithSelectedShip();

    this.sound.mute = !userSettings.sound_effects_enabled || userSettings.sound_effects_volume === 0;
    this.sound.volume = userSettings.sound_effects_enabled ? userSettings.sound_effects_volume / 100 : 0;

    // Input
    this.moveLeftKey = this.input.keyboard!.addKey(this.keyCodeFor(userSettings.turn_left_key, Phaser.Input.Keyboard.KeyCodes.A));
    this.moveRightKey = this.input.keyboard!.addKey(this.keyCodeFor(userSettings.turn_right_key, Phaser.Input.Keyboard.KeyCodes.D));
    this.thrustKey = this.input.keyboard!.addKey(this.keyCodeFor(userSettings.thrust_key, Phaser.Input.Keyboard.KeyCodes.W));
    this.plusKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.PLUS);
    this.minusKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.MINUS);
    this.shiftKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.SHIFT);
    this.numpadAddKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.NUMPAD_ADD);
    this.numpadSubtractKey = this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.NUMPAD_SUBTRACT);
    this.fleetLayoutKey = this.input.keyboard!.addKey(FLEET_LAYOUT_KEY);
    this.rotateEnemyKey = this.input.keyboard!.addKey(ROTATE_ENEMY_KEY);
    this.fireEnemyWeaponKey = this.input.keyboard!.addKey(FIRE_ENEMY_WEAPON_KEY);
    this.fireEnemyWeaponAltKey = this.input.keyboard!.addKey(FIRE_ENEMY_WEAPON_ALT_KEY);
    this.input.mouse?.disableContextMenu();
    this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      if (!this.started) {
        return;
      }

      const x = Math.round(pointer.worldX);
      const y = Math.round(pointer.worldY);
      if (pointer.rightButtonDown()) {
        this.sendBattleMessage({ type: 'setPlayerSpecialTargetPoint', x, y });
        appendDebugLine(`teleport x=${x} y=${y}`);
        return;
      }

      const targetType = this.targetShip.containsPoint(pointer.worldX, pointer.worldY)
        ? 'ship'
        : 'point';
      if (targetType === 'ship') {
        this.sendBattleMessage({ type: 'setPlayerWeaponTargetShip' });
      } else {
        this.sendBattleMessage({ type: 'setPlayerWeaponTargetPoint', x, y });
      }
      appendDebugLine(`${targetType} x=${x} y=${y}`);
    });

    window.dispatchEvent(new Event('battlecontrol:scene-ready'));
  }

  update() {
    const input = {
      left: this.moveLeftKey.isDown,
      right: this.moveRightKey.isDown,
      thrust: this.thrustKey.isDown,
      weapon: false,
      special: false,
    };
    const pointer = this.input.activePointer;
    const hudInput = {
      ...input,
      weapon: pointer.leftButtonDown(),
      special: pointer.rightButtonDown(),
    };
    const neutralInput = {
      left: false,
      right: false,
      thrust: false,
      weapon: false,
      special: false,
    };

    if (!this.started) {
      this.updateCamera();
      this.updatePlanetRender();
      this.playerShip.renderUpdate(this.shipRenderScale);
      this.targetShip.renderUpdate(this.shipRenderScale);
      this.renderShipHitPolygon(this.playerHitPolygon, this.playerShip, 0x00ff66);
      this.renderTargetHitPolygon();
      this.hud.update(neutralInput);
      return;
    }

    if (isDebugUiEnabled() && hudInput.weapon && !this.lastWeaponDown) {
      this.resetProjectileTrace();
    }
    this.lastWeaponDown = hudInput.weapon;

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

    if (
      Phaser.Input.Keyboard.JustDown(this.fireEnemyWeaponKey)
      || Phaser.Input.Keyboard.JustDown(this.fireEnemyWeaponAltKey)
    ) {
      if (this.battleWorker) {
        this.battleWorker.postMessage({ type: 'triggerTargetWeapon' });
      }
    }

    this.sendBattleMessage({ type: 'setPlayerInput', input: hudInput });
    if (this.battleWorker) {
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
    }

    // Visual update every render frame
    this.updateCamera();
    this.updatePlanetRender();
    this.playerShip.renderUpdate(this.shipRenderScale);
    this.targetShip.renderUpdate(this.shipRenderScale);
    this.renderShipHitPolygon(this.playerHitPolygon, this.playerShip, 0x00ff66);
    this.renderTargetHitPolygon();
    this.hud.update(hudInput);
  }

  private updateCamera() {
    if (this.targetShip.dead) {
      this.cameraTarget.setPosition(this.playerShip.x, this.playerShip.y);
      this.shipRenderScale = SHIP_FAR_SCALE;
      this.planetRenderScale = PLANET_FAR_SCALE;
      this.cameras.main.setZoom(
        Phaser.Math.Linear(this.cameras.main.zoom, MIN_CAMERA_ZOOM, ZOOM_LERP),
      );
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
    if (this.battleWorker) {
      this.battleWorker.postMessage({ type: 'switchPlayerShip', shipType: preset.stats.spritePrefix });
    }
    this.cameraTarget.setPosition(this.playerShip.x, this.playerShip.y);
    this.saveSelectedShip();
    this.syncHudWithSelectedShip();
  }

  private applySnapshot(snapshot: BattleSnapshot) {
    this.playerShip.applySnapshot(snapshot.player);
    this.targetShip.applySnapshot(snapshot.target);
    this.syncProjectileTrace(snapshot);
    this.syncMeteors(snapshot);
    this.syncProjectiles(snapshot);
    this.syncExplosions(snapshot);
    this.syncLasers(snapshot);
    this.syncTargetOpponent(snapshot);
    this.playAudioEvents(snapshot);
    if ((snapshot.player.dead || snapshot.target.dead) && !this.finishedEventSent) {
      this.finishedEventSent = true;
      window.dispatchEvent(new CustomEvent('battlecontrol:battle-finished'));
    }
    const zoomStatus = `zoom=${this.cameras.main.zoom.toFixed(3)}`;
    setDebugStatus(
      snapshot.projectiles[0]
        ? `${zoomStatus} rocket life=${snapshot.projectiles[0].life}`
        : zoomStatus,
    );
  }

  private resetProjectileTrace() {
    this.projectileTraceFrame = 0;
    this.projectileTraceStopped = false;
    clearDebugTrace();
  }

  private syncProjectileTrace(snapshot: BattleSnapshot) {
    if (!isDebugUiEnabled()) {
      return;
    }

    const projectile = snapshot.projectiles[0];
    if (!projectile) {
      return;
    }

    if (this.projectileTraceStopped) {
      return;
    }

    const dx = snapshot.target.x - projectile.x;
    const dy = snapshot.target.y - projectile.y;
    const distance = Math.round(Math.sqrt((dx * dx) + (dy * dy)));
    appendDebugTrace(
      `frame=${this.projectileTraceFrame} p=${projectile.id} bubble x=${Math.round(projectile.x)} y=${Math.round(projectile.y)} vx=${Math.round(projectile.vx)} vy=${Math.round(projectile.vy)} enemy=${snapshot.target.id} x=${Math.round(snapshot.target.x)} y=${Math.round(snapshot.target.y)} d=${distance}`,
    );
    this.projectileTraceFrame += 1;
  }

  private playAudioEvents(snapshot: BattleSnapshot) {
    const userSettings = (this.game.registry.get('userSettings') as UserSettingsDto | undefined) ?? DEFAULT_USER_SETTINGS;
    if (!userSettings.sound_effects_enabled || userSettings.sound_effects_volume === 0) {
      return;
    }

    for (const event of snapshot.audioEvents) {
      switch (event.key) {
        case 'androsynth-primary':
          this.playSfx(event.key, 0.55);
          break;
        case 'yehat-primary':
          this.playSfx(event.key, 0.55);
          break;
        case 'yehat-special':
          this.playSfxPattern(event.key, 0.65, [120, 420]);
          break;
        case 'androsynth-special':
          this.playSfx(event.key, 0.65);
          break;
        case 'arilou-primary':
          this.playSfx(event.key, 0.55);
          break;
        case 'arilou-special':
          this.playSfx(event.key, 0.65);
          break;
        case 'arilou-victory':
          this.playSfx(event.key, 0.65);
          break;
        case 'human-primary':
          this.playSfx(event.key, 0.5);
          break;
        case 'human-special':
          this.playSfx(event.key, 0.6);
          break;
        case 'human-victory':
          this.playSfx(event.key, 0.65);
          break;
        case 'battle-shipdies':
          this.playSfx(event.key, 0.75);
          break;
        case 'battle-boom-23':
          this.playSfx(event.key, 0.5);
          break;
        case 'battle-boom-45':
          this.playSfx(event.key, 0.55);
          break;
        default:
          break;
      }
    }
  }

  private playSfx(key: string, volume: number) {
    let sound = this.sfxChannels.get(key);
    if (!sound) {
      sound = this.sound.add(key, { volume });
      this.sfxChannels.set(key, sound);
    }
    sound.setVolume(volume);
    if (sound.isPlaying) {
      sound.stop();
    }
    sound.play();
  }

  private playSfxIfIdle(key: string, volume: number) {
    let sound = this.sfxChannels.get(key);
    if (!sound) {
      sound = this.sound.add(key, { volume });
      this.sfxChannels.set(key, sound);
    }
    sound.setVolume(volume);
    if (!sound.isPlaying) {
      sound.play();
    }
  }

  private playSfxThrottled(key: string, volume: number, minIntervalMs: number) {
    const now = this.time.now;
    const lastPlayedAtMs = this.sfxLastPlayedAtMs.get(key) ?? Number.NEGATIVE_INFINITY;
    if (now - lastPlayedAtMs < minIntervalMs) {
      return;
    }
    this.playSfx(key, volume);
    this.sfxLastPlayedAtMs.set(key, now);
  }

  private playSfxPattern(key: string, volume: number, intervalsMs: number[]) {
    const now = this.time.now;
    const step = this.sfxPatternStep.get(key) ?? 0;
    const minIntervalMs = intervalsMs[step % intervalsMs.length] ?? 0;
    const lastPlayedAtMs = this.sfxLastPlayedAtMs.get(key) ?? Number.NEGATIVE_INFINITY;
    if (now - lastPlayedAtMs < minIntervalMs) {
      return;
    }
    this.playSfx(key, volume);
    this.sfxLastPlayedAtMs.set(key, now);
    this.sfxPatternStep.set(key, (step + 1) % intervalsMs.length);
  }

  private syncProjectiles(snapshot: BattleSnapshot) {
    while (this.projectileSprites.length < snapshot.projectiles.length) {
      const projectile = this.add.image(0, 0, 'ion-particle');
      projectile.setDepth(10);
      this.projectileSprites.push(projectile);
    }

    while (this.projectileHitPolygons.length < snapshot.projectiles.length) {
      const polygon = this.add.graphics();
      polygon.setDepth(10);
      this.projectileHitPolygons.push(polygon);
    }

    while (this.projectileSprites.length > snapshot.projectiles.length) {
      this.projectileSprites.pop()?.destroy();
    }

    while (this.projectileHitPolygons.length > snapshot.projectiles.length) {
      this.projectileHitPolygons.pop()?.destroy();
    }

    snapshot.projectiles.forEach((projectile, index) => {
      const sprite = this.projectileSprites[index];
      const polygon = this.projectileHitPolygons[index];
      sprite.setPosition(projectile.x, projectile.y);

      if (projectile.texturePrefix) {
        const frameIndex = projectileFrameFor(
          projectile.texturePrefix,
          projectile.vx,
          projectile.vy,
          projectile.life,
        );
        sprite.setTexture(`${projectile.texturePrefix}-big-${String(frameIndex).padStart(3, '0')}`);
        if (projectile.texturePrefix === 'androsynth-bubble') {
          sprite.setTint(0xa8f0ff);
          sprite.setScale(1.35);
        } else {
          sprite.clearTint();
          sprite.setScale(0.75);
        }
      } else {
        sprite.setTexture('ion-particle');
        sprite.setTint(0xffffff);
        sprite.setScale(2);
      }

      this.renderProjectileHitPolygon(polygon, sprite, 0x4488ff);
    });
  }

  private syncMeteors(snapshot: BattleSnapshot) {
    while (this.meteorSprites.length < snapshot.meteors.length) {
      const meteor = this.add.image(0, 0, 'battle-asteroid-big-000');
      meteor.setDepth(8);
      this.meteorSprites.push(meteor);
    }

    while (this.meteorSprites.length > snapshot.meteors.length) {
      this.meteorSprites.pop()?.destroy();
    }

    snapshot.meteors.forEach((meteor, index) => {
      const sprite = this.meteorSprites[index];
      sprite.setPosition(meteor.x, meteor.y);
      sprite.setTexture(
        `${meteor.texturePrefix}-big-${String(meteor.frameIndex).padStart(3, '0')}`,
      );
      sprite.setScale(0.9);
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
      const explosionKey = `${explosion.texturePrefix}:${explosion.frameIndex}:${Math.round(explosion.x)}:${Math.round(explosion.y)}`;
      if (explosion.frameIndex === 0 && !this.loggedExplosionKeys.has(explosionKey)) {
        appendDebugLine(
          `hit e=${explosion.id} type=${explosion.texturePrefix} x=${Math.round(explosion.x)} y=${Math.round(explosion.y)} player=${snapshot.player.id} x=${Math.round(snapshot.player.x)} y=${Math.round(snapshot.player.y)} enemy=${snapshot.target.id} x=${Math.round(snapshot.target.x)} y=${Math.round(snapshot.target.y)}`,
        );
        this.projectileTraceStopped = true;
        this.loggedExplosionKeys.add(explosionKey);
      }

      const sprite = this.explosionSprites[index];
      sprite.setPosition(explosion.x, explosion.y);
      sprite.setTexture(
        `${explosion.texturePrefix}-big-${String(explosion.frameIndex).padStart(3, '0')}`,
      );
      sprite.setScale(explosion.texturePrefix === 'battle-boom' ? 1 : 0.75);
    });

    this.loggedExplosionKeys = new Set(
      snapshot.explosions.map((explosion) => (
        `${explosion.texturePrefix}:${explosion.frameIndex}:${Math.round(explosion.x)}:${Math.round(explosion.y)}`
      )),
    );
  }

  private syncLasers(snapshot: BattleSnapshot) {
    while (this.laserSprites.length < snapshot.lasers.length) {
      const laser = this.add.line(0, 0, 0, 0, 0, 0, 0xffffff, 0.95);
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
      line.setStrokeStyle(laser.width, laser.color, 0.95);
    });
  }

  private renderTargetHitPolygon() {
    this.renderShipHitPolygon(this.targetHitPolygon, this.targetShip, 0xff0000);
  }

  private renderProjectileHitPolygon(
    graphics: Phaser.GameObjects.Graphics,
    sprite: Phaser.GameObjects.Image,
    color: number,
  ) {
    graphics.clear();
    if (!isDebugUiEnabled()) {
      return;
    }

    graphics.lineStyle(2, color, 0.95);
    graphics.beginPath();
    graphics.moveTo(sprite.x - 6, sprite.y);
    graphics.lineTo(sprite.x + 6, sprite.y);
    graphics.moveTo(sprite.x, sprite.y - 6);
    graphics.lineTo(sprite.x, sprite.y + 6);
    graphics.strokePath();

    const texturePolygon = getTextureHitPolygon(this.textures.get(sprite.texture.key), sprite.scaleX);
    if (texturePolygon.length < 2) {
      return;
    }

    graphics.lineStyle(2, color, 0.95);
    graphics.beginPath();
    graphics.moveTo(sprite.x + texturePolygon[0].x, sprite.y + texturePolygon[0].y);
    for (const point of texturePolygon.slice(1)) {
      graphics.lineTo(sprite.x + point.x, sprite.y + point.y);
    }
    graphics.closePath();
    graphics.strokePath();
  }

  private renderShipHitPolygon(
    graphics: Phaser.GameObjects.Graphics,
    ship: Ship,
    color: number,
  ) {
    graphics.clear();
    if (!isDebugUiEnabled() || ship.dead) {
      return;
    }

    const currentTextureKey = ship.getCurrentTextureKey();
    const currentTexturePrefix = currentTextureKey.replace(/-big-\d+$/, '');
    const polygon = getShipHitPolygon(
      currentTexturePrefix,
      ship.facing,
      this.shipRenderScale * getShipRenderScale(ship.stats.size),
    );
    const texturePolygon = getTextureHitPolygon(
      this.textures.get(currentTextureKey),
      this.shipRenderScale * getShipRenderScale(ship.stats.size),
    );
    const visiblePolygon = polygon.length >= 3 ? polygon : texturePolygon;
    if (visiblePolygon.length < 2) {
      return;
    }

    graphics.lineStyle(2, color, 0.95);
    graphics.beginPath();
    graphics.moveTo(ship.x + visiblePolygon[0].x, ship.y + visiblePolygon[0].y);
    for (const point of visiblePolygon.slice(1)) {
      graphics.lineTo(ship.x + point.x, ship.y + point.y);
    }
    graphics.closePath();
    graphics.strokePath();
  }

  private getPresetBySpritePrefix(spritePrefix: string): ShipPreset {
    const preset = this.shipPresets.find((candidate) => candidate.stats.spritePrefix === spritePrefix);
    if (!preset) {
      throw new Error(`Missing ship preset for ${spritePrefix}`);
    }

    return preset;
  }

  private syncHudWithSelectedShip() {
    const battleSetup = this.game.registry.get('battleSetup') as BattleSetup | undefined;
    const preset = battleSetup
      ? this.getPresetBySpritePrefix(battleSetup.playerShipType)
      : this.shipPresets[this.selectedShipIndex];
    this.hud.setShip({
      ship: this.playerShip,
      stats: preset.stats,
      portraitUrl: preset.portraitUrl,
      captainFrameUrls: preset.captainFrameUrls,
      captainFrameStyles: preset.captainFrameStyles,
      captainLayout: preset.captainLayout,
      captainName: this.playerCaptainName,
    } satisfies HUDShipInfo);
    this.syncFleetLayout();
  }

  private toggleFleetLayout() {
    this.syncFleetLayout();
  }

  private syncFleetLayout() {
    const battleSetup = this.game.registry.get('battleSetup') as BattleSetup | undefined;
    const targetPreset = this.getPresetBySpritePrefix(battleSetup?.targetShipType ?? 'human-cruiser');
    const targetOpponent = this.toOtherShipHudState(targetPreset, 'target-0', this.targetCaptainName, true);

    if (battleSetup?.opponents?.length) {
      this.currentAllies = [];
      this.currentOpponents = battleSetup.opponents.map((opponent, index) => {
        const preset = this.getPresetBySpritePrefix(index === 0 ? battleSetup.targetShipType : opponent.shipType);
        return this.toOtherShipHudState(
          preset,
          opponent.id,
          opponent.captainName,
          index === 0,
        );
      });
      this.hud.setFleet(this.currentAllies, this.currentOpponents);
      return;
    }

    this.currentAllies = [];
    this.currentOpponents = [targetOpponent];
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

    return selected.map((preset, index) => this.toOtherShipHudState(preset, `${side}-${index}`, undefined, true));
  }

  private toOtherShipHudState(
    preset: ShipPreset,
    id: string,
    captainName = Phaser.Utils.Array.GetRandom(preset.stats.captainNames),
    isActiveTarget = false,
  ): OtherShipHudState {
    return {
      id,
      portraitUrl: preset.portraitUrl,
      portraitHeight: getOtherShipPortraitHeight(preset.stats.size),
      renderScale: preset.stats.renderScale ?? getShipRenderScale(preset.stats.size),
      isActiveTarget,
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

  private keyCodeFor(keyName: string, fallback: number) {
    const normalized = keyName.trim();
    const legacyNormalized = normalized.toUpperCase();

    const explicitMappings: Record<string, number> = {
      Space: Phaser.Input.Keyboard.KeyCodes.SPACE,
      ArrowLeft: Phaser.Input.Keyboard.KeyCodes.LEFT,
      ArrowRight: Phaser.Input.Keyboard.KeyCodes.RIGHT,
      ArrowUp: Phaser.Input.Keyboard.KeyCodes.UP,
      ArrowDown: Phaser.Input.Keyboard.KeyCodes.DOWN,
    };

    if (explicitMappings[normalized]) {
      return explicitMappings[normalized];
    }

    if (normalized.startsWith('Key') && normalized.length === 4) {
      const letter = normalized.slice(3).toUpperCase();
      return Phaser.Input.Keyboard.KeyCodes[letter as keyof typeof Phaser.Input.Keyboard.KeyCodes] ?? fallback;
    }

    return Phaser.Input.Keyboard.KeyCodes[legacyNormalized as keyof typeof Phaser.Input.Keyboard.KeyCodes] ?? fallback;
  }

  private connectBattleSocket(gameId: string) {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    this.battleSocket = new WebSocket(`${protocol}//${window.location.host}/games/${gameId}/battle`);
    const notifyFailure = (message: string) => {
      if (this.battleSocketFailed) {
        return;
      }
      this.battleSocketFailed = true;
      window.dispatchEvent(new CustomEvent('battlecontrol:battle-connection-failed', {
        detail: { message },
      }));
    };
    this.battleSocket.onerror = () => {
      notifyFailure('Battle connection failed');
    };
    this.battleSocket.onclose = () => {
      notifyFailure('Battle connection lost');
    };
    this.battleSocket.onmessage = (event: MessageEvent<string>) => {
      const payload = JSON.parse(event.data) as BattleWorkerResponse;
      if (payload.type !== 'snapshot') {
        return;
      }

      this.applySnapshot(payload.snapshot);
    };
  }

  private sendBattleMessage(message: BattleWorkerMessage) {
    if (this.battleWorker) {
      this.battleWorker.postMessage(message);
      return;
    }

    if (!this.battleSocket || this.battleSocket.readyState !== WebSocket.OPEN) {
      return;
    }

    switch (message.type) {
      case 'setPlayerInput':
        this.battleSocket.send(JSON.stringify({ type: 'setInput', input: message.input }));
        break;
      case 'setPlayerWeaponTargetPoint':
        this.battleSocket.send(JSON.stringify({ type: 'setWeaponTargetPoint', x: message.x, y: message.y }));
        break;
      case 'setPlayerWeaponTargetShip':
        this.battleSocket.send(JSON.stringify({ type: 'setWeaponTargetShip' }));
        break;
      case 'setPlayerSpecialTargetPoint':
        this.battleSocket.send(JSON.stringify({ type: 'setSpecialTargetPoint', x: message.x, y: message.y }));
        break;
      case 'clearPlayerWeaponTarget':
        this.battleSocket.send(JSON.stringify({ type: 'clearWeaponTarget' }));
        break;
      case 'clearPlayerSpecialTarget':
        this.battleSocket.send(JSON.stringify({ type: 'clearSpecialTarget' }));
        break;
      default:
        break;
    }
  }
}
