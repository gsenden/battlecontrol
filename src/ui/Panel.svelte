<script lang="ts">
  import type { Snippet } from 'svelte';

  let {
    title,
    panelClass = '',
    bodyClass = '',
    children,
  }: {
    title: string;
    panelClass?: string;
    bodyClass?: string;
    children?: Snippet;
  } = $props();
</script>

<section
  class={`pointer-events-auto flex flex-col gap-1 bg-[#515151] border-6 border-[#424242] border-b-[#838383] border-r-[#838383] p-1.5 [image-rendering:pixelated] ${panelClass}`.trim()}
>
  <div class="flex items-start justify-center font-[990] font-[StarCon] text-black tracking-wider drop-shadow-[0_2px_0_#7D7D7D]">
    <span class="text-[30px] leading-none -mt-[1px]">{title[0] ?? ''}</span>
    <span class="text-[22px] leading-none">{title.slice(1)}</span>
  </div>

  <div class={`flex min-h-0 flex-1 flex-col ${bodyClass}`.trim()}>
    {@render children?.()}
  </div>
</section>
