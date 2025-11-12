import fs from 'node:fs';
import path from 'node:path';

const directory = path.join(import.meta.dirname, 'svg');

const readAttribute = (svg, attribute) => {
	const attributeStartString = `${attribute}="`;

	const attributeStart = svg.indexOf(attributeStartString);
	const attributeEnd = svg.indexOf('"', attributeStart + attributeStartString.length);

	return svg.substring(attributeStart + attributeStartString.length, attributeEnd);
};

const replaceAttribute = (svg, attribute, replacement) => {
	const attributeStartString = `${attribute}="`;

	const attributeStart = svg.indexOf(attributeStartString);
	const attributeEnd = svg.indexOf('"', attributeStart + attributeStartString.length);

	return svg.substring(0, attributeStart) + replacement + svg.substring(attributeEnd + '"'.length);
};

const addAttribute = (svg, attribute, value) => {
	const svgStart = svg.indexOf('svg');
	const svgClose = svg.indexOf('>', svgStart); /* '>' in '<svg ...>' */

	return svg.substring(0, svgClose) + ' ' + `${attribute}=${value}` + svg.substring(svgClose);
};

const makeHeader = (w, h) => `<script lang="ts">
	const { width = ${w}, height = ${h}, class: className }: { width?: number; height?: number, class?: string } = $props();
</script>`;

const convertFileName = (filename) =>
	filename
		.split('-')
		.map((w) => w.charAt(0).toUpperCase() + w.substring(1))
		.join('')
		.replace('.svg', '.svelte');

fs.readdirSync(directory)
	.filter((f) => f.endsWith('.svg'))
	.map((f) => [f, fs.readFileSync(path.join(directory, f), 'utf-8')])
	.map(([f, svg]) => {
		const defaultWidth = readAttribute(svg, 'width');
		const defaultHeight = readAttribute(svg, 'height');

		svg = makeHeader(defaultWidth, defaultHeight) + '\n'.repeat(2) + svg;
		svg = replaceAttribute(svg, 'width', '{width}');
		svg = replaceAttribute(svg, 'height', '{height}');
		svg = addAttribute(svg, 'class', '{className}');

		return [f, svg];
	})
	.forEach(([f, svg]) =>
		fs.writeFileSync(path.join(directory, '..', 'output', convertFileName(f)), svg)
	);
