import type { Config } from 'tailwindcss';

const config: Config = {
	content: ['./index.html', './src/**/*.{js,ts,jsx,tsx,css,md,mdx,html,json,scss}'],
	darkMode: 'class',
	theme: {
		extend: {
			colors: {
				text: {
					50: 'hsl(var(--text-50))',
					100: 'hsl(var(--text-100))',
					200: 'hsl(var(--text-200))',
					300: 'hsl(var(--text-300))',
					400: 'hsl(var(--text-400))',
					500: 'hsl(var(--text-500))',
					600: 'hsl(var(--text-600))',
					700: 'hsl(var(--text-700))',
					800: 'hsl(var(--text-800))',
					900: 'hsl(var(--text-900))',
					950: 'hsl(var(--text-950))'
				},
				background: {
					50: 'hsl(var(--background-50))',
					100: 'hsl(var(--background-100))',
					200: 'hsl(var(--background-200))',
					300: 'hsl(var(--background-300))',
					400: 'hsl(var(--background-400))',
					500: 'hsl(var(--background-500))',
					600: 'hsl(var(--background-600))',
					700: 'hsl(var(--background-700))',
					800: 'hsl(var(--background-800))',
					900: 'hsl(var(--background-900))',
					950: 'hsl(var(--background-950))'
				},
				primary: {
					50: 'hsl(var(--primary-50))',
					100: 'hsl(var(--primary-100))',
					200: 'hsl(var(--primary-200))',
					300: 'hsl(var(--primary-300))',
					400: 'hsl(var(--primary-400))',
					500: 'hsl(var(--primary-500))',
					600: 'hsl(var(--primary-600))',
					700: 'hsl(var(--primary-700))',
					800: 'hsl(var(--primary-800))',
					900: 'hsl(var(--primary-900))',
					950: 'hsl(var(--primary-950))'
				},
				secondary: {
					50: 'hsl(var(--secondary-50))',
					100: 'hsl(var(--secondary-100))',
					200: 'hsl(var(--secondary-200))',
					300: 'hsl(var(--secondary-300))',
					400: 'hsl(var(--secondary-400))',
					500: 'hsl(var(--secondary-500))',
					600: 'hsl(var(--secondary-600))',
					700: 'hsl(var(--secondary-700))',
					800: 'hsl(var(--secondary-800))',
					900: 'hsl(var(--secondary-900))',
					950: 'hsl(var(--secondary-950))'
				},
				accent: {
					50: 'hsl(var(--accent-50))',
					100: 'hsl(var(--accent-100))',
					200: 'hsl(var(--accent-200))',
					300: 'hsl(var(--accent-300))',
					400: 'hsl(var(--accent-400))',
					500: 'hsl(var(--accent-500))',
					600: 'hsl(var(--accent-600))',
					700: 'hsl(var(--accent-700))',
					800: 'hsl(var(--accent-800))',
					900: 'hsl(var(--accent-900))',
					950: 'hsl(var(--accent-950))'
				}
			}
		}
	},
	plugins: []
};

export default config;
