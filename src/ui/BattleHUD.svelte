<script lang="ts">
  import { getFleetPanelKeys, type HudState } from './hud-state.svelte.js';
  import AlliesPanel from './AlliesPanel.svelte';
  import MyShipHUD from './MyShipHUD.svelte';
  import EnemiesPanel from './EnemiesPanel.svelte';

  let { state }: { state: HudState } = $props();
  const panelKeys = $derived(getFleetPanelKeys(state.allies.length));
</script>

<div
        class="flex h-full w-full flex-col items-stretch"
>
  <MyShipHUD {state} />

  <div class="flex min-h-0 flex-1 flex-col">
    {#each panelKeys as panelKey}
      {#if panelKey === 'teammates'}
        <div class="flex min-h-0" style="flex: 7 7 0%;">
          <AlliesPanel ships={state.allies} />
        </div>
      {:else}
        <div class="flex min-h-0" style={`flex: ${state.allies.length > 0 ? 8 : 1} ${state.allies.length > 0 ? 8 : 1} 0%;`}>
          <EnemiesPanel ships={state.opponents} />
        </div>
      {/if}
    {/each}
  </div>
</div>
