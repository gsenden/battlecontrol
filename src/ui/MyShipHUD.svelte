<script lang="ts">
  import type { HudState } from './hud-state.svelte.js';
  import crewLabelUrl from '../assets/ui/crew-label.png';
  import battLabelUrl from '../assets/ui/batt-label.png';
  import StatBar from './StatBar.svelte'

  let { state }: { state: HudState } = $props();
</script>

<div
        class="flex w-full flex-col items-center gap-1 bg-[#515151] border-6 border-[#424242] border-b-[#838383] border-r-[#838383] p-1.5 [image-rendering:pixelated]"
>

  <div class="flex items-start justify-center font-[990] font-[StarCon] text-black tracking-wider drop-shadow-[0_2px_0_#7D7D7D]"><span class="text-[30px] leading-none -mt-[1px]">{state.shipName[0]}</span><span class="text-[22px] leading-none">{state.shipName.slice(1)}</span></div>

  <div class="flex items-end w-full justify-between">
    <StatBar type="crew" current={state.crew} max={state.maxCrew} />

    <div class="w-[160px] h-[160px] flex items-center justify-center overflow-hidden">
      <img class="max-w-full max-h-full [image-rendering:pixelated]" src={state.portraitUrl} alt={state.shipName} />
    </div>

    <StatBar type="batt" current={state.energy} max={state.maxEnergy} />
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
