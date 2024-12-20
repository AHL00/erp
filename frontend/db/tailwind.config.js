/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			colors: {
				'custom-dark': '#181818',
				'custom-darker': '#151515',
				'custom-dark-shadow': '#0f0f0f',
				'custom-dark-outline': '#343434',
				'custom-text-dark': '#f5f5f5',
				'custom-text-dark-lighter': '#e0e0e0',
				'custom-text-dark-darker': '#d0d0d0',
				'custom-light': '#f9f9f9',
				'custom-lighter': '#ffffff',
				'custom-light-shadow': '#dddddd',
				'custom-light-outline': '#c5c5c5',
				'custom-text-light': '#181818',
				'custom-text-light-darker': '#121212',
				'custom-text-light-lighter': '#888888'
			}
		}
	},
	darkMode: 'media',
	plugins: []
};
