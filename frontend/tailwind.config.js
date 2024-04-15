/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			colors: {
				'custom-bg-dark': '#181818',
				'custom-bg-darker': '#151515',
				'custom-bg-dark-shadow': '#0f0f0f',
				'custom-bg-dark-outline': '#282828',
				'custom-text-dark': '#f5f5f5',
				'custom-text-dark-darker': '#e0e0e0',
				'custom-text-dark-lighter': '#d0d0d0',
				'custom-bg-light': '#f9f9f9',
				'custom-bg-lighter': '#ffffff',
				'custom-bg-light-shadow': '#dddddd',
				'custom-bg-light-outline': '#f0f0f0',
				'custom-text-light': '#181818',
				'custom-text-light-darker': '#121212',
				'custom-text-light-lighter': '#888888'
			}
		}
	},
	darkMode: 'media',
	plugins: []
};
