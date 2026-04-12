<script lang="ts">
  import type { HudState } from './hud-state.svelte.js';
  import MyShipHUD from './MyShipHUD.svelte';
  import EnemiesPanel from './EnemiesPanel.svelte';

  let { state }: { state: HudState } = $props();
</script>

<div
        class="flex h-full w-full flex-col items-stretch"
>
  <MyShipHUD {state} />

  <div class="flex min-h-0 flex-1 flex-col">
    <div class="flex min-h-0 flex-1">
      <EnemiesPanel ships={state.opponents} />
    </div>
  </div>
</div>
