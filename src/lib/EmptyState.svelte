<script lang="ts">
  let {
    message,
    icon,
    children,
  }: {
    message: string;
    icon?: any;
    children?: any;
  } = $props();
</script>

<div class="empty-state">
  {#if icon}
    <div class="empty-state-icon">{@render icon()}</div>
  {/if}
  <div class="empty-state-message">{message}</div>
  {#if children}
    {@render children()}
  {/if}
</div>
