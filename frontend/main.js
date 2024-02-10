import App from './App.svelte';
import { refreshAuthInfo } from './auth';

refreshAuthInfo().then(() => {
    new App({
        target: document.body,
    });
});