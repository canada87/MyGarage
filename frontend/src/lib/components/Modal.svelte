<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';

	let {
		title,
		onclose,
		children,
		footer,
		wide = false
	}: { title: string; onclose: () => void; children: Snippet; footer?: Snippet; wide?: boolean } = $props();

	let el: HTMLDialogElement;
	const titleId = `modal-${Math.random().toString(36).slice(2)}`;

	onMount(() => {
		el.showModal();
		return () => el.open && el.close();
	});
</script>

<dialog
	bind:this={el}
	class:wide
	aria-labelledby={titleId}
	oncancel={(e) => {
		e.preventDefault();
		onclose();
	}}
	onclick={(e) => {
		if (e.target === el) onclose();
	}}
>
	<div class="sheet">
		<header>
			<h2 id={titleId}>{title}</h2>
			<button type="button" class="icon-btn ghost" onclick={onclose} aria-label="Chiudi"><Icon name="x" /></button>
		</header>
		<div class="body">
			{@render children()}
		</div>
		{#if footer}
			<footer>{@render footer()}</footer>
		{/if}
	</div>
</dialog>

<style>
	dialog {
		border: 0;
		padding: 0;
		background: transparent;
		color: inherit;
		max-width: 100vw;
		max-height: 100dvh;
		width: 100%;
		margin: auto auto 0;
		overflow: visible;
	}
	dialog::backdrop {
		background: var(--scrim);
		animation: fade 0.2s var(--ease);
	}
	.sheet {
		background: var(--surface);
		border-radius: 20px 20px 0 0;
		box-shadow: var(--shadow-lg);
		max-height: 92dvh;
		display: flex;
		flex-direction: column;
		animation: up 0.28s var(--ease);
		padding-bottom: env(safe-area-inset-bottom);
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 12px 12px 4px 20px;
	}
	.body {
		padding: 8px 20px 20px;
		overflow-y: auto;
		overscroll-behavior: contain;
	}
	footer {
		padding: 12px 20px 16px;
		border-top: 1px solid var(--border);
	}
	@media (min-width: 640px) {
		dialog {
			width: min(560px, calc(100vw - 32px));
			margin: auto;
		}
		dialog.wide {
			width: min(720px, calc(100vw - 32px));
		}
		.sheet {
			border-radius: 20px;
			max-height: 88dvh;
			animation: pop 0.22s var(--ease);
		}
	}
	@keyframes up {
		from {
			transform: translateY(40px);
			opacity: 0;
		}
	}
	@keyframes pop {
		from {
			transform: scale(0.97);
			opacity: 0;
		}
	}
	@keyframes fade {
		from {
			opacity: 0;
		}
	}
</style>
