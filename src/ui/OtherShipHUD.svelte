<script lang="ts">
  import type { OtherShipHudState } from './hud-state.svelte.js';
  import OtherShipStatBar from './OtherShipStatBar.svelte';

  let { ship }: { ship: OtherShipHudState } = $props();
</script>

<div class="grid grid-cols-[60px_56px_minmax(0,1fr)] items-center gap-x-1.5">
  <div class="flex min-h-8 shrink-0 items-center justify-center">
    <img
      class={`[image-rendering:pixelated] ${ship.dead ? 'grayscale brightness-75 opacity-80' : ''}`}
      style={`height: ${ship.portraitHeight * ship.renderScale}px; width: auto; max-width: none;`}
      src={ship.portraitUrl}
      alt={ship.shipName}
    />
  </div>

  <div class="flex w-[56px] min-h-[18px] items-center justify-center">
    {#if ship.dead}
      <div class="font-[StarCon] text-[14px] leading-none tracking-[0.08em] text-black">
        DIED
      </div>
    {:else}
      <div class="flex w-[56px] flex-col gap-0.5">
        <OtherShipStatBar type="crew" current={ship.crew} max={ship.maxCrew} barMax={ship.crewBarMax} />
        <OtherShipStatBar type="batt" current={ship.energy} max={ship.maxEnergy} barMax={ship.energyBarMax} />
      </div>
    {/if}
  </div>

  <div class={`min-w-0 font-[StarCon] text-black text-[17px] leading-none tracking-wide ${ship.dead ? 'text-[#8b8b8b]' : 'text-[#d6d6d6]'}`}>
    {ship.captainName}
  </div>
</div>
