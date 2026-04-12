<script lang="ts">
  let {
    label,
    onclick,
    className = "",
  }: {
    label: string;
    onclick?: () => void;
    className?: string;
  } = $props();
</script>

<button
  class={`text-sm text-grey-200 underline underline-offset-4 transition hover:text-[#d6e7ff] ${className}`.trim()}
  {onclick}
  type="button"
>
  {label}
</button>
