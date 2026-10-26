<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import Icon from '#lib/components/Icon.svelte';
	import Toasts from '#lib/components/Toasts.svelte';
	import DialogHost from '#lib/components/DialogHost.svelte';
	import { app, loadVehicles, openDialog } from '#lib/state.svelte.ts';
	import { getTheme, setTheme, isDark } from '#lib/theme.ts';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	let dark = $state(false);
	onMount(() => {
		dark = isDark(getTheme());
		void loadVehicles();
	});

	function toggleTheme() {
		const next = dark ? 'light' : 'dark';
		setTheme(next);
		dark = next === 'dark';
	}

	const nav = [
		{ href: '/', label: 'Home', side: 'Dashboard', icon: 'home' },
		{ href: '/auto', label: 'Auto', side: 'Auto', icon: 'car' },
		{ href: '/scadenze', label: 'Scadenze', side: 'Scadenze', icon: 'calendar' },
		{ href: '/spese', label: 'Spese', side: 'Spese', icon: 'wallet' },
		{ href: '/rubrica', label: 'Rubrica', side: 'Rubrica', icon: 'contacts' }
	];

	function active(href: string) {
		const p = page.url.pathname;
		return href === '/' ? p === '/' : p === href || p.startsWith(`${href}/`);
	}

	/** Car currently open, so that quick actions default to it. */
	const currentVehicleId = $derived.by(() => {
		const m = page.url.pathname.match(/^\/auto\/(\d+)/);
		return m ? Number(m[1]) : undefined;
	});
</script>

<a class="skip" href="#main">Vai al contenuto</a>

<div class="app">
	<nav class="sidebar" aria-label="Navigazione principale">
		<a class="brand" href="/"><span class="brand-mark"><Icon name="car" /></span>MyGarage</a>
		{#each nav as n (n.href)}
			<a class="side-link" href={n.href} aria-current={active(n.href) ? 'page' : undefined}>
				<Icon name={n.icon} />{n.side}
			</a>
			{#if n.href === '/auto' && app.vehicles.length}
				<div class="side-sub">
					{#each app.vehicles as v (v.id)}
						<a class="side-link" href="/auto/{v.id}" aria-current={currentVehicleId === v.id ? 'page' : undefined}>
							<span class="dot" style="background:{v.color}"></span>{v.name}
						</a>
					{/each}
				</div>
			{/if}
		{/each}
		<div class="push"></div>
		<button class="side-link" type="button" onclick={toggleTheme}>
			<Icon name={dark ? 'sun' : 'moon'} />{dark ? 'Tema chiaro' : 'Tema scuro'}
		</button>
		<a class="side-link" href="/impostazioni" aria-current={active('/impostazioni') ? 'page' : undefined}>
			<Icon name="sliders" />Impostazioni
		</a>
	</nav>

	<div class="content">
		<header class="topbar">
			<a class="brand" href="/"><span class="brand-mark"><Icon name="car" /></span>MyGarage</a>
			<span class="spacer"></span>
			<button class="icon-btn" type="button" onclick={toggleTheme} aria-label={dark ? 'Passa al tema chiaro' : 'Passa al tema scuro'}>
				<Icon name={dark ? 'sun' : 'moon'} />
			</button>
			<a class="icon-btn" href="/impostazioni" aria-label="Impostazioni"><Icon name="sliders" /></a>
		</header>

		<main id="main" tabindex="-1">
			{@render children()}
		</main>
	</div>

	<nav class="bottomnav" aria-label="Navigazione">
		{#each nav as n (n.href)}
			<a href={n.href} aria-current={active(n.href) ? 'page' : undefined}><Icon name={n.icon} />{n.label}</a>
		{/each}
	</nav>

	<button class="fab" type="button" onclick={() => openDialog({ kind: 'actions', vehicleId: currentVehicleId })}>
		<Icon name="plus" />Registra
	</button>
</div>

<Toasts />
<DialogHost />

<style>
	.skip {
		position: absolute;
		left: -999px;
		top: 8px;
		z-index: 200;
		background: var(--primary);
		color: var(--on-primary);
		padding: 10px 16px;
		border-radius: 10px;
		font-weight: 700;
	}
	.skip:focus {
		left: 8px;
	}
	.app {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		min-height: 100dvh;
	}
	.sidebar {
		display: none;
	}
	main {
		padding: 16px 16px calc(150px + env(safe-area-inset-bottom));
		max-width: 1200px;
		width: 100%;
		margin: 0 auto;
		outline: none;
	}
	.topbar {
		position: sticky;
		top: 0;
		z-index: 20;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		padding-top: calc(12px + env(safe-area-inset-top));
		background: color-mix(in srgb, var(--bg) 85%, transparent);
		backdrop-filter: blur(12px);
		-webkit-backdrop-filter: blur(12px);
		border-bottom: 1px solid var(--border);
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 10px;
		font-weight: 800;
		font-size: 18px;
		letter-spacing: -0.01em;
		min-height: 44px;
	}
	.brand-mark {
		width: 32px;
		height: 32px;
		border-radius: 9px;
		display: grid;
		place-items: center;
		background: var(--primary);
		color: var(--on-primary);
	}
	.spacer {
		flex: 1;
	}
	.bottomnav {
		position: fixed;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 30;
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		background: var(--surface);
		border-top: 1px solid var(--border);
		padding-bottom: env(safe-area-inset-bottom);
	}
	.bottomnav a {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		min-height: 60px;
		font-size: 12px;
		font-weight: 600;
		color: var(--fg-subtle);
	}
	.bottomnav a[aria-current='page'] {
		color: var(--primary);
	}
	.fab {
		position: fixed;
		right: 16px;
		bottom: calc(76px + env(safe-area-inset-bottom));
		z-index: 31;
		height: 56px;
		padding: 0 20px 0 16px;
		border-radius: 18px;
		border: 0;
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: 8px;
		font-weight: 700;
		background: var(--primary);
		color: var(--on-primary);
		box-shadow: 0 8px 24px color-mix(in srgb, var(--primary) 40%, transparent);
		transition: transform 0.15s var(--ease);
	}
	.fab:active {
		transform: scale(0.96);
	}
	@media (min-width: 1024px) {
		.app {
			grid-template-columns: 248px minmax(0, 1fr);
		}
		.sidebar {
			display: flex;
			flex-direction: column;
			gap: 4px;
			position: sticky;
			top: 0;
			height: 100dvh;
			overflow-y: auto;
			padding: 20px 14px;
			background: var(--surface);
			border-right: 1px solid var(--border);
		}
		.sidebar .brand {
			padding: 4px 10px 20px;
		}
		.side-link {
			display: flex;
			align-items: center;
			gap: 12px;
			padding: 10px 12px;
			border-radius: 10px;
			font-weight: 600;
			color: var(--fg-muted);
			transition: background 0.15s var(--ease);
			border: 0;
			background: transparent;
			cursor: pointer;
			text-align: left;
			min-height: 44px;
		}
		.side-link:hover {
			background: var(--surface-2);
		}
		.side-link[aria-current='page'] {
			background: var(--primary-soft);
			color: var(--primary);
		}
		.side-sub {
			padding-left: 22px;
			display: flex;
			flex-direction: column;
			gap: 2px;
		}
		.side-sub .side-link {
			font-weight: 500;
			padding: 8px 12px;
			min-height: 40px;
		}
		.push {
			margin-top: auto;
		}
		.topbar,
		.bottomnav {
			display: none;
		}
		main {
			padding: 28px 32px 112px;
		}
		.fab {
			bottom: 32px;
			right: 32px;
		}
	}
</style>
