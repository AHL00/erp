/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
            colors: {
                'custom-bg-dark': '#181818',
                'custom-bg-darker': '#151515',
                'custom-bg-dark-shadow': '#0f0f0f',
                'custom-text-dark': '#f5f5f5',
                'custom-bg-light': '#f9f9f9',
                'custom-bg-lighter': '#ffffff',
                'custom-bg-light-shadow': '#dddddd',
                'custom-text-light': '#181818',
            }
        }
	},
    darkMode: 'media',
	plugins: []
};
