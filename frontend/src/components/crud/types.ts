import type { SortOrder } from '$bindings/SortOrder';

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

export type CrudEditType =
	| { type: 'string'; data: CrudEditTypeString }
	| { type: 'number'; data: CrudEditTypeNumber }
	| { type: 'select'; data: CrudEditTypeSelect<any> }
	| { type: 'checkbox' }
	| { type: 'date' }
	| { type: 'time' }
	| { type: 'datetime' }
	| { type: 'file' }
	| { type: 'image' }
	| { type: 'password' }
	| { type: 'hidden' }
	| { type: 'textarea'; data: CrudEditTypeTextarea };

export interface CrudColumn {
	api_name: string;
	display_name: string;
	current_sort: SortOrder | null;
	edit_type: CrudEditType;
    edit_readonly: boolean;
}
