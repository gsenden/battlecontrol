<script lang="ts">

  type BarType = 'crew' | 'batt';

  let { type, current, max }: { type: BarType; current: number; max: number } = $props();

  const colors = $derived.by(() => {
    if (type === 'crew') {
      return { filled: '#008F00', empty: 'black' };
    }
    return { filled: '#7A0000', empty: 'black' };
  });

  // Build rows of 2 blocks, bottom-up (row 0 = top, last row = bottom)
  // SC2 fills from bottom: block 0 is bottom-right, block 1 is bottom-left, etc.
  let rows = $derived.by(() => {
    const numRows = Math.ceil(max / 2);
    const result: { left: boolean; right: boolean }[] = [];
    for (let row = 0; row < numRows; row++) {
      const rightIndex = row * 2;       // even = right column
      const leftIndex = row * 2 + 1;    // odd = left column
      result.push({
        right: rightIndex < max && (max - rightIndex) <= current,
        left: leftIndex < max && (max - leftIndex) <= current,
      });
    }
    return result;
  });

</script>

<div class="flex flex-col items-center gap-0.5">
  <div class="bg-black border-2 border-t-[#838383] border-l-[#838383] border-b-[#424242] border-r-[#424242] p-1">
    <div class="flex flex-col gap-[4px]">
      {#each rows as row}
        <div class="flex gap-[4px]">
          <div
            class="w-2.5 h-[5px]"
            style:background-color={row.left ? colors.filled : colors.empty}
          ></div>
          <div
            class="w-2.5 h-[5px]"
            style:background-color={row.right ? colors.filled : colors.empty}
          ></div>
        </div>
      {/each}
    </div>
  </div>
</div>
