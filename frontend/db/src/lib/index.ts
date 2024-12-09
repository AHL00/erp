import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import { refreshAuthStatus } from './auth';
import { get_setting } from './backend';

/// Redirect to a new page
/// This will also refresh the authentication status
export function redirect(url: string) {
	if (!browser) return;

	// Redirect to the new page
	goto(url);

	// Refresh authenticatation
	refreshAuthStatus();
}

export function open_in_new_tab(url: string) {
	if (!browser) return;

	window.open(url, '_blank');
}

export function middle_ellipsis(text: string, max_length: number) {
	if (text.length <= max_length) {
		return text;
	}

	let half = Math.floor(max_length / 2);
	return text.slice(0, half) + '...' + text.slice(text.length - half);
}

export type date_time_accuracy = 'day' | 'hour' | 'minute' | 'second';

import dateFormat, { masks } from 'dateformat';

export function format_local_date(date: Date, format: string): string {
	return dateFormat(date, format);
}

export function utc_iso_to_local_formatted(
	date: string,
	format: string,
	accuracy: date_time_accuracy = 'second'
): string {
	return format_local_date(new Date(utc_date_to_local_rounded(date, accuracy)), format);
}

export function utc_date_to_local_rounded(
	isoString: string,
	round: date_time_accuracy = 'minute'
): string {
	let date = new Date(isoString);
	let localMillis = date.getTime() - date.getTimezoneOffset() * 60000;
	let localDate = new Date(localMillis);

	switch (round) {
		case 'day':
			localDate.setHours(0, 0, 0, 0);
			break;
		case 'hour':
			localDate.setMinutes(0, 0, 0);
			break;
		case 'minute':
			localDate.setSeconds(0, 0);
			break;
		case 'second':
			localDate.setMilliseconds(0);
			break;
	}

	let localIso = localDate.toISOString();
	localIso = localIso.slice(0, -1);

	return localIso;
}

export function local_date_to_iso_utc(localString: string): string {
	let date = new Date(localString);
	let utcIso = date.toISOString();

	return utcIso;
}

export function compare_dates_milliseconds(a: string, b: string): number {
	let date_a = new Date(a);
	let date_b = new Date(b);

	return date_a.getTime() - date_b.getTime();
}
