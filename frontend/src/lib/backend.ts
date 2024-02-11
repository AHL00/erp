export const api_base = import.meta.env.VITE_API_BASE_URL ? import.meta.env.VITE_API_BASE_URL : '/';

console.log('API_BASE:', api_base);

if (!api_base) {
	throw new Error('VITE_API_BASE_URL is not set');
}
