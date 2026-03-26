<script lang="ts">
  import type { HudState } from './hud-state.svelte.js';
  import MyShipHUD from './MyShipHUD.svelte';

  let { state }: { state: HudState } = $props();
</script>

<div
        class="flex h-full w-full flex-col items-stretch gap-2"
>
  <MyShipHUD {state} />
</div>
