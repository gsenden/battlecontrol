# BattleControl - SC2 Super Melee Web Port

Port van het Super Melee gevechtsysteem uit Star Control 2 naar een web game.

**Stack:** Phaser 3 + ingebouwde Matter.js plugin (rendering + physics), Vite + TypeScript, Vitest (tests)
**Later:** matter-rs (native Rust) op de server voor authoritative multiplayer physics
**Referentie:** `/home/gsenden/projects/sc2/src/uqm/` (originele C broncode)

---

## Stap 0: Reference data uit originele SC2 code

- [ ] Klein C test-harness dat SC2 physics functies aanroept voor bekende scenarios
- [ ] Output als JSON: positie/velocity na N frames thrust, turn sequences, energy drain, etc.
- [ ] `testdata/` map met reference JSON bestanden
- [ ] Vitest helper die reference data inlaadt en vergelijkt

**Scenarios:**
- [ ] Ship thrust: N frames → verwachte velocity/positie
- [ ] Turning: N keer links/rechts met turnWait → verwachte facing
- [ ] Energy: regen timing, weapon/special drain
- [ ] Gravity well: positie schip nabij planeet → kracht per frame
- [ ] Max speed cap: thrust voorbij max → velocity afgekapt
- [ ] Cooldowns: weapon/special/thrust timing

**Verificatie:** `npm test` vergelijkt onze output met reference data

---

## Stap 1: Project setup + leeg speelveld

- [x] Vite + TypeScript + Vitest + Phaser 3
- [ ] Phaser config met Matter.js physics plugin
- [ ] BattleScene: zwart speelveld
- [ ] Statische planeet in het midden (cirkel body, static)

**Verificatie:** `npm run dev` toont zwart veld met planeet

---

## Stap 2: 1 schip met beweging

- [ ] ShipStats interface + Human Cruiser constanten
- [ ] ShipState class: cooldowns, energy, facing
- [ ] Ship als Matter.js polygon body (driehoek placeholder)
- [ ] Keyboard input: pijltjes voor turn/thrust
- [ ] Thrust: `applyForce()` in kijkrichting met thrustWait
- [ ] Turning: discrete hoekstappen met turnWait
- [ ] Max speed cap
- [ ] Thruster flame visueel effect bij thrust

**Tests:**
- [ ] ShipState cooldowns (turn, thrust, weapon, special)
- [ ] Energy regen timing
- [ ] Thrust richting na turn
- [ ] Speed cap
- [ ] Vergelijking met reference data (stap 0)

**Verificatie:** schip vliegt rond, voelt als SC2

---

## Stap 3: Toroidal wrapping

- [ ] Schip dat de rand verlaat verschijnt aan de andere kant
- [ ] Ghost rendering: schip zichtbaar aan beide kanten bij de rand
- [ ] Correcte afstandsberekening over de wrap-grens

**Tests:**
- [ ] Wrap positie berekening
- [ ] Wrap delta (kortste pad)

**Verificatie:** vloeiend wrappen zonder visuele glitches

---

## Stap 4: Planeet zwaartekracht

- [ ] Gravity well rond de planeet (afstandsgebaseerd)
- [ ] `applyForce()` richting planeet wanneer schip binnen threshold
- [ ] Schip kan boven max speed komen in gravity well (gravity whip)
- [ ] Botsing met planeet = schade
- [ ] Vergelijking met reference data gravity scenario

**Tests:**
- [ ] Gravity force richting en magnitude
- [ ] Speed override in gravity well
- [ ] Planet collision damage

**Verificatie:** schip wordt aangetrokken, gravity whip werkt

---

## Stap 5: Battle UI (HUD)

- [ ] Crew bar (health) per speler
- [ ] Energy bar per speler
- [ ] Ship naam
- [ ] Responsive web layout (niet de originele SC2 zijbalk)

**Verificatie:** bars reageren op energy gebruik en schade

---

## Stap 6: Tweede schip + 2P local input

- [ ] Speler 1: WASD + Q/E (weapon/special)
- [ ] Speler 2: Pijltjes + ./slash (weapon/special)
- [ ] Camera: zoom op basis van afstand tussen schepen
- [ ] Toroidal distance voor camera berekening

**Tests:**
- [ ] Input mapping
- [ ] Camera zoom logic

**Verificatie:** 2 spelers besturen elk een schip

---

## Stap 7: Botsingen tussen schepen

- [ ] Elastische botsingen via Matter.js restitution
- [ ] Botsing = tijdelijk turn/thrust cooldown (SC2: COLLISION_TURN_WAIT=1, COLLISION_THRUST_WAIT=3)
- [ ] Schade bij botsing op basis van relatieve snelheid
- [ ] Visueel: flash/shake bij impact

**Tests:**
- [ ] Collision damage berekening
- [ ] Cooldown reset na botsing

**Verificatie:** schepen botsen realistisch, nemen schade

---

## Stap 8: Wapensysteem (projectielen)

- [ ] Human Cruiser wapen: homing nuke missile
- [ ] Projectiel als Matter.js body (klein, snel)
- [ ] Energie check + cooldown
- [ ] Projectiel-schip botsing = schade
- [ ] Projectiel-projectiel botsing = vernietiging

**Tests:**
- [ ] Projectiel spawning + richting
- [ ] Energy drain
- [ ] Collision damage

**Verificatie:** schieten, projectiel vliegt, raakt vijand

---

## Stap 9: Tweede schiptype

- [ ] Tweede schip met andere stats en ander wapen
- [ ] Ship selectie voor de battle begint
- [ ] Balans tuning

---

## Stap 10: matter-rs WASM migratie (optioneel)

- [ ] Phaser Matter.js plugin vervangen door matter-rs WASM
- [ ] Verificatie: identiek gedrag
- [ ] Voorbereiding voor server-side physics
