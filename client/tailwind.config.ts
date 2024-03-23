import type { Config } from 'tailwindcss';

const config: Config = {
	content: ['./index.html', './src/**/*.{js,ts,jsx,tsx,css,md,mdx,html,json,scss}'],
	darkMode: 'class',
	theme: {
		extend: {
			colors: {
				text: {
					DEFAULT: 'hsl(var(--text) / <alpha-value>)',
					50: 'hsl(var(--text-50) / <alpha-value>)',
					100: 'hsl(var(--text-100) / <alpha-value>)',
					200: 'hsl(var(--text-200) / <alpha-value>)',
					300: 'hsl(var(--text-300) / <alpha-value>)',
					400: 'hsl(var(--text-400) / <alpha-value>)',
					500: 'hsl(var(--text-500) / <alpha-value>)',
					600: 'hsl(var(--text-600) / <alpha-value>)',
					700: 'hsl(var(--text-700) / <alpha-value>)',
					800: 'hsl(var(--text-800) / <alpha-value>)',
					900: 'hsl(var(--text-900) / <alpha-value>)',
					950: 'hsl(var(--text-950) / <alpha-value>)'
				},
				background: {
					DEFAULT: 'hsl(var(--background) / <alpha-value>)',
					50: 'hsl(var(--background-50) / <alpha-value>)',
					100: 'hsl(var(--background-100) / <alpha-value>)',
					200: 'hsl(var(--background-200) / <alpha-value>)',
					300: 'hsl(var(--background-300) / <alpha-value>)',
					400: 'hsl(var(--background-400) / <alpha-value>)',
					500: 'hsl(var(--background-500) / <alpha-value>)',
					600: 'hsl(var(--background-600) / <alpha-value>)',
					700: 'hsl(var(--background-700) / <alpha-value>)',
					800: 'hsl(var(--background-800) / <alpha-value>)',
					900: 'hsl(var(--background-900) / <alpha-value>)',
					950: 'hsl(var(--background-950) / <alpha-value>)'
				},
				primary: {
					DEFAULT: 'hsl(var(--primary) / <alpha-value>)',
					50: 'hsl(var(--primary-50) / <alpha-value>)',
					100: 'hsl(var(--primary-100) / <alpha-value>)',
					200: 'hsl(var(--primary-200) / <alpha-value>)',
					300: 'hsl(var(--primary-300) / <alpha-value>)',
					400: 'hsl(var(--primary-400) / <alpha-value>)',
					500: 'hsl(var(--primary-500) / <alpha-value>)',
					600: 'hsl(var(--primary-600) / <alpha-value>)',
					700: 'hsl(var(--primary-700) / <alpha-value>)',
					800: 'hsl(var(--primary-800) / <alpha-value>)',
					900: 'hsl(var(--primary-900) / <alpha-value>)',
					950: 'hsl(var(--primary-950) / <alpha-value>)'
				},
				secondary: {
					DEFAULT: 'hsl(var(--secondary) / <alpha-value>)',
					50: 'hsl(var(--secondary-50) / <alpha-value>)',
					100: 'hsl(var(--secondary-100) / <alpha-value>)',
					200: 'hsl(var(--secondary-200) / <alpha-value>)',
					300: 'hsl(var(--secondary-300) / <alpha-value>)',
					400: 'hsl(var(--secondary-400) / <alpha-value>)',
					500: 'hsl(var(--secondary-500) / <alpha-value>)',
					600: 'hsl(var(--secondary-600) / <alpha-value>)',
					700: 'hsl(var(--secondary-700) / <alpha-value>)',
					800: 'hsl(var(--secondary-800) / <alpha-value>)',
					900: 'hsl(var(--secondary-900) / <alpha-value>)',
					950: 'hsl(var(--secondary-950) / <alpha-value>)'
				},
				accent: {
					DEFAULT: 'hsl(var(--accent) / <alpha-value>)',
					50: 'hsl(var(--accent-50) / <alpha-value>)',
					100: 'hsl(var(--accent-100) / <alpha-value>)',
					200: 'hsl(var(--accent-200) / <alpha-value>)',
					300: 'hsl(var(--accent-300) / <alpha-value>)',
					400: 'hsl(var(--accent-400) / <alpha-value>)',
					500: 'hsl(var(--accent-500) / <alpha-value>)',
					600: 'hsl(var(--accent-600) / <alpha-value>)',
					700: 'hsl(var(--accent-700) / <alpha-value>)',
					800: 'hsl(var(--accent-800) / <alpha-value>)',
					900: 'hsl(var(--accent-900) / <alpha-value>)',
					950: 'hsl(var(--accent-950) / <alpha-value>)'
				}
			},
			fontFamily: {
				sans: ['Inter', 'sans-serif']
			}
		}
	},
	plugins: []
};

export default config;
