<script lang="ts">
  import type { HudState } from './hud-state.svelte.js';
  import crewLabelUrl from '../assets/ui/crew-label.png';
  import battLabelUrl from '../assets/ui/batt-label.png';
  import StatBar from './StatBar.svelte'

  let { state }: { state: HudState } = $props();

  let captainUrl = $derived(state.captainFrameUrls[state.captainFrame] ?? '');
</script>

<div class="flex flex-col items-center gap-1 bg-[#515151] border-6 border-[#424242] border-b-[#838383] border-r-[#838383] p-1.5 [image-rendering:pixelated]">

  <!-- Ship name -->
  <div class="flex items-start justify-center font-[990] font-[StarCon] text-black tracking-wider drop-shadow-[0_2px_0_#7D7D7D]"><span class="text-[40px] leading-none -mt-[1px]">{state.shipName[0]}</span><span class="text-[32px] leading-none">{state.shipName.slice(1)}</span></div>

  <div class="flex items-end w-full justify-between">
    <StatBar type="crew" current={state.crew} max={state.maxCrew} />


    <div class="w-[190px] h-[190px] flex items-center justify-center overflow-hidden">
      <img class="max-w-full max-h-full [image-rendering:pixelated]" src={state.portraitUrl} alt={state.shipName} />
    </div>


    <StatBar type="batt" current={state.energy} max={state.maxEnergy} />


  </div>

  <div class="flex items-end gap-1 justify-between w-full">

    <img class="h-5 [image-rendering:pixelated]" src={crewLabelUrl} alt="CREW" />
    <img class="h-5 [image-rendering:pixelated]" src={battLabelUrl} alt="BATT" />

  </div>



  <div class="text-black text-[32px] font-[StarCon] font-bold text-center">{state.captainName}</div>

  <div class="w-[260px] h-[142px] bg-black border-[#838383] border-b-[#424242] border-r-[#424242]] border-3 flex items-center justify-center overflow-hidden">
    <img class="w-full h-full object-contain [image-rendering:pixelated]" src={captainUrl} alt="Captain" />
  </div>


</div>
