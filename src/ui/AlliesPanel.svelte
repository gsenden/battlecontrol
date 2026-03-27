<script lang="ts">
  import type { OtherShipHudState } from './hud-state.svelte.js';
  import OtherShipHUD from './OtherShipHUD.svelte';
  import Panel from './Panel.svelte';

  let { ships }: { ships: OtherShipHudState[] } = $props();
</script>

<Panel title="ALLIES" panelClass="min-h-0 flex-1">
  <div class="fleet-scrollbar pointer-events-auto flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
    {#each ships as ship (ship.id)}
      <OtherShipHUD {ship} />
    {/each}
  </div>
</Panel>
