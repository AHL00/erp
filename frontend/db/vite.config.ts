import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import StringReplace from 'vite-plugin-string-replace'

// @ts-ignore
import { execSync } from 'child_process';

// Get the current Git commit ID
const commitId = execSync('git rev-parse --short HEAD').toString().trim();

export default defineConfig({
	plugins: [
		sveltekit(),
		StringReplace([
				{
					search: '__COMMIT_ID__',
					replace: commitId
				}])
	],

	css: {
		preprocessorOptions: {
			scss: {
				additionalData: '@use "src/variables.scss" as *;'
			}
		}
	}
});
