<script lang="ts">
  import type { HudState } from './hud-state.svelte.js';
  import crewLabelUrl from '../assets/ui/crew-label.png';
  import battLabelUrl from '../assets/ui/batt-label.png';
  import Panel from './Panel.svelte';
  import StatBar from './StatBar.svelte'

  let { state }: { state: HudState } = $props();
</script>

<Panel title={state.shipName} panelClass="w-full h-[440px] items-center">
  <div class="mt-auto flex w-full flex-col items-center gap-1">
    <div class="flex items-end w-full justify-between">
      <StatBar type="crew" current={state.crew} max={state.maxCrew} barMax={state.crewBarMax} />

      <div class="self-center w-[160px] h-[160px] flex items-center justify-center overflow-hidden">
        <img class="max-w-full max-h-full [image-rendering:pixelated]" src={state.portraitUrl} alt={state.shipName} />
      </div>

      <StatBar type="batt" current={state.energy} max={state.maxEnergy} barMax={state.energyBarMax} />
    </div>

    <div class="flex items-end gap-1 justify-between w-full">
      <img class="h-4 [image-rendering:pixelated]" src={crewLabelUrl} alt="CREW" />
      <img class="h-4 [image-rendering:pixelated]" src={battLabelUrl} alt="BATT" />
    </div>

    <div class="text-black text-[22px] font-[StarCon] font-bold text-center">{state.captainName}</div>

    <div class="border-[#838383] border-b-[#424242] border-r-[#424242] border-3 bg-black">
      <div class="relative w-[220px] h-[120px] overflow-hidden">
        <img class="absolute left-0 top-0 w-[220px] h-[120px] [image-rendering:pixelated]" src={state.captainBackgroundUrl} alt="Captain" />

        {#each state.captainTurnLayers as turnLayer}
          <img class="absolute [image-rendering:pixelated]" style={turnLayer.style} src={turnLayer.url} alt="" />
        {/each}

        {#if state.captainThrustUrl}
          <img class="absolute [image-rendering:pixelated]" style={state.captainThrustStyle} src={state.captainThrustUrl} alt="" />
        {/if}

        {#if state.captainWeaponUrl}
          <img class="absolute [image-rendering:pixelated]" style={state.captainWeaponStyle} src={state.captainWeaponUrl} alt="" />
        {/if}

        {#if state.captainSpecialUrl}
          <img class="absolute [image-rendering:pixelated]" style={state.captainSpecialStyle} src={state.captainSpecialUrl} alt="" />
        {/if}
      </div>
    </div>
  </div>
</Panel>
