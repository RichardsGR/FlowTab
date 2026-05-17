<script lang="ts">
  let {
    message,
    icon,
    children,
  }: {
    message: string;
    icon?: string;
    children?: any;
  } = $props();
</script>

<div class="empty-state">
  {#if icon}
    <div class="empty-state-icon">{icon}</div>
  {/if}
  <div>{message}</div>
  {#if children}
    {@render children()}
  {/if}
</div>
