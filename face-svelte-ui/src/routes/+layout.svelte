<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import tokens from '../../../assets/design_tokens.json';
	import { browser } from '$app/environment';
	import { getTosState } from '$lib/stores/ipc.svelte';

	let { children }: { children: Snippet } = $props();

	const tosState = getTosState();

	$effect(() => {
		if (browser) {
			const highContrastForced = tosState.settings?.global?.['tos.interface.high_contrast'] === 'true';
			const theme = tosState.settings?.global?.['tos.interface.theme'] || 'dark';

			if (highContrastForced || tosState.supports_high_contrast) {
				document.body.className = 'tos-theme-high-contrast';
			} else {
				document.body.className = `tos-theme-${theme}`;
			}
		}
	});

	function generateTokens(t: any): string {
		let css = ':root {\n';
		for (const [key, val] of Object.entries(t.typography)) {
			css += `  --font-${key}: ${val};\n`;
		}
		for (const [key, val] of Object.entries(t.spacing)) {
			css += `  --space-${key}: ${val};\n`;
		}
		for (const [key, val] of Object.entries(t.radii)) {
			css += `  --radius-${key}: ${val};\n`;
		}
		css += '}\n';
		css += '.tos-theme-dark {\n';
		for (const [key, val] of Object.entries(t.themes.dark)) {
			css += `  --color-${key.replace('_', '-')}: ${val};\n`;
		}
		css += '}\n';
		if (t.themes.high_contrast) {
			css += '.tos-theme-high-contrast {\n';
			for (const [key, val] of Object.entries(t.themes.high_contrast)) {
				css += `  --color-${key.replace('_', '-')}: ${val};\n`;
			}
			css += '}\n';
		}
		return css;
	}

	const tokenCss = generateTokens(tokens);
</script>

<svelte:head>
	{@html `<style>${tokenCss}</style>`}
</svelte:head>

{@render children()}
