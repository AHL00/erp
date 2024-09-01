import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { replaceCodePlugin } from 'vite-plugin-replace';
// @ts-ignore
import { execSync } from 'child_process';

// Get the current Git commit ID
const commitId = execSync('git rev-parse --short HEAD').toString().trim();

export default defineConfig({
	plugins: [
		sveltekit(),
		replaceCodePlugin({
			replacements: [
				{
					from: '__COMMIT_ID__',
					to: commitId
				}
			]
		})
	],

	css: {
		preprocessorOptions: {
			scss: {
				additionalData: '@use "src/variables.scss" as *;'
			}
		}
	}
});
