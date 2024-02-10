export const api_base = import.meta.env.VITE_API_BASE_URL;

if (!api_base) {
	throw new Error('VITE_API_BASE_URL is not set');
}
