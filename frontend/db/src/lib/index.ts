import { browser } from "$app/environment";
import { goto } from "$app/navigation";
import { refreshAuthStatus } from "./auth";
import { get_setting } from "./backend";


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

export function utc_date_to_local(isoString: string): string {
    let date = new Date(isoString);
    let localMillis = date.getTime() - date.getTimezoneOffset() * 60000;
    let localDate = new Date(localMillis)
    let localIso = localDate.toISOString()
    localIso = localIso.slice(0, -1);

    return localIso;
}