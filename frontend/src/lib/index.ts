import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { refreshAuthStatus } from "./auth";


/// Redirect to a new page
/// This will also refresh the authentication status
export function redirect(url: string) {
    if (!browser) return;

    // Redirect to the new page
    goto(url);

    // Refresh authenticatation
    console.log('Refreshing auth status');
    refreshAuthStatus();
}