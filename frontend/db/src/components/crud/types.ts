import type { SortOrder } from '$bindings/SortOrder';
import type { date_time_accuracy } from '$lib';

export interface CrudEditTypeSelect<T> {
	options: T[];
}

export interface CrudEditTypeString {
	regex: string | null;
	/// Inclusive, set 0 to make field optional
	length_range: [number, number | null];
}

export interface CrudEditTypeNumber {
	/// Inclusive
	range: [number | null, number | null];
	/// This overrides decimal_places
	integer: boolean;
	step: number;
}

export interface CrudEditTypeTextarea {
	/// Inclusive, set 0 to make field optional
	length_range: [number, number | null];

	regex: string | null;

	resize: 'none' | 'both' | 'horizontal' | 'vertical';
}

export type CrudValueType =
	| { type: 'string'; data: CrudEditTypeString }
	| { type: 'number'; data: CrudEditTypeNumber }
	| { type: 'currency' }
	| { type: 'select'; data: CrudEditTypeSelect<any> }
	| { type: 'checkbox' }
	| { type: 'date' }
	| { type: 'time' }
	| { type: 'datetime', accuracy: date_time_accuracy, format: string }
	| { type: 'file' }
	| { type: 'image' }
	| { type: 'password' }
	| { type: 'textarea'; data: CrudEditTypeTextarea }
	| { type: 'use_display_map_fn_and_no_edit' };

export interface CrudColumn {
	/// The name in structs returned by the API
	api_name: string;
	/// This is the name used in things such as list request
	/// If null, will use api_name
	api_request_name: string | null;
	/// If null, the column will not be displayed
	display_name: string | null;
	/// Allows custom formatting and processing of the data before displaying it
	display_map_fn: ((value: any) => string) | null;
	current_sort: SortOrder | null;
	type: CrudValueType;
    /// Show in the edit panel
	edit: boolean;
    /// Disable editing in the edit panel even if shown
	readonly: boolean;
    searchable: boolean;
    /// Check SearchRequest for more info
    search_nested?: string;
    align?: 'left' | 'center' | 'right';
}
// export interface CustomColumn<T> {
//     /// The name of the custom column
//     name: string;
//     /// The value of the custom column
//     value: any;
//     /// Allows custom formatting and processing of the data before displaying it
//     display_map_fn: ((value: T) => string) | null;
// }

// export type Column<T> = CrudColumn<T> | CustomColumn<T>;
