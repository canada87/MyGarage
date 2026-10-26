<script lang="ts">
	import { dismissToast, toasts } from '#lib/state.svelte.ts';
	import Icon from './Icon.svelte';
</script>

<div class="toasts" role="status" aria-live="polite">
	{#each toasts as t (t.id)}
		<div class="toast {t.kind}">
			<Icon name={t.kind === 'error' ? 'circle-alert' : t.kind === 'success' ? 'ok' : 'info'} size={18} />
			<span class="msg">{t.message}</span>
			{#if t.action}
				<button type="button" class="act" onclick={() => t.action?.run()}>{t.action.label}</button>
			{:else}
				<button type="button" class="close" onclick={() => dismissToast(t.id)} aria-label="Chiudi notifica"><Icon name="x" size={16} /></button>
			{/if}
		</div>
	{/each}
</div>

<style>
	.toasts {
		position: fixed;
		left: 50%;
		transform: translateX(-50%);
		bottom: calc(148px + env(safe-area-inset-bottom));
		z-index: 100;
		display: grid;
		gap: 8px;
		width: min(440px, calc(100vw - 32px));
		pointer-events: none;
	}
	@media (min-width: 1024px) {
		.toasts {
			bottom: 32px;
		}
	}
	.toast {
		pointer-events: auto;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 10px 10px 14px;
		border-radius: 14px;
		background: var(--fg);
		color: var(--bg);
		box-shadow: var(--shadow-lg);
		font-weight: 600;
		font-size: 15px;
		animation: in 0.25s var(--ease);
	}
	.toast.error {
		background: var(--danger);
		color: #fff;
	}
	.msg {
		flex: 1;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.act {
		border: 0;
		background: transparent;
		color: inherit;
		font-weight: 800;
		text-decoration: underline;
		min-height: 40px;
		padding: 0 10px;
		cursor: pointer;
	}
	.close {
		border: 0;
		background: transparent;
		color: inherit;
		width: 36px;
		height: 36px;
		display: grid;
		place-items: center;
		cursor: pointer;
		opacity: 0.8;
	}
	@keyframes in {
		from {
			transform: translateY(12px);
			opacity: 0;
		}
	}
</style>
