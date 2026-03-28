<script lang="ts">
	import '../app.css';
	import type { Snippet } from 'svelte';
	import tokens from '../../../assets/design_tokens.json';

	let { children }: { children: Snippet } = $props();

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
		return css;
	}

	const tokenCss = generateTokens(tokens);
</script>

<svelte:head>
	{@html `<style>${tokenCss}</style>`}
</svelte:head>

{@render children()}
