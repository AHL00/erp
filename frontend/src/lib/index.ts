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
    refreshAuthStatus();
}

export function middle_ellipsis(text: string, max_length: number) {
    if (text.length <= max_length) {
        return text;
    }

    let half = Math.floor(max_length / 2);
    return text.slice(0, half) + '...' + text.slice(text.length - half);
}