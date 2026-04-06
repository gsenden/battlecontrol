SC2 collision reference for `matter`

Use [matter-sc2-blazer-human-collision.json](/home/gsenden/projects/battlecontrol/testdata/matter-sc2-blazer-human-collision.json).

Intent:
- `matter` keeps detection + separation
- `SkipVelocity` skips Matter bounce
- caller uses collision normal + masses to apply SC2 bounce

First target test in `matter`:
- head-on `blazer` vs `human cruiser`
- collision normal is readable
- custom SC2 bounce yields:
  - blazer `vx = -7.142857...`
  - human `vx = 2.857142...`

SC2 notes:
- blazer hit also does `3` damage before the ordinary collision bounce
- ordinary collision also applies cooldowns:
  - `turn_wait = 1`
  - `thrust_wait = 3`
- ordinary ship collision does not do generic crew damage by itself
