<script lang="ts">
  type BarType = 'crew' | 'batt';
  const CHMMR_BAR_MAX = 42;

  let {
    type,
    current,
    max,
    barMax,
  }: {
    type: BarType;
    current: number;
    max: number;
    barMax: number;
  } = $props();

  const widthPercent = $derived(`${(Math.max(0, barMax) / CHMMR_BAR_MAX) * 100}%`);
  const fillPercent = $derived(barMax <= 0 ? 0 : (Math.max(0, Math.min(current, barMax)) / barMax) * 100);
  const colors = $derived(
    type === 'crew'
      ? { filled: '#008F00', empty: '#0b250b' }
      : { filled: '#7A0000', empty: '#240808' },
  );
</script>

<div
  class="h-[7px] border border-[#838383] border-b-[#424242] border-r-[#424242] bg-black p-[1px]"
  style:width={widthPercent}
>
  <div
    class="h-full"
    style={`background: linear-gradient(to right, ${colors.filled} 0 ${fillPercent}%, ${colors.empty} ${fillPercent}% 100%);`}
  ></div>
</div>
