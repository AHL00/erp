import type { SortOrder } from '$bindings/SortOrder';

export interface CrudEditTypeSelect<T> {
	options: T[];
}

export interface CrudEditTypeString {
    regex: string | null;
    /// Inclusive
    length_range: [number | null, number | null];
    text_area: boolean;
}

export interface CrudEditTypeNumber {
    /// Inclusive
    range: [number | null, number | null];
    /// This overrides decimal_places
    integer: boolean;
    /// Null means no limit
    decimal_places: number | null;
}


export type CrudEditType =
	| CrudEditTypeString
    | CrudEditTypeNumber
	| CrudEditTypeSelect<any>
	| 'checkbox'
	| 'date'
	| 'time'
	| 'datetime'
	| 'file'
	| 'image'
	| 'password'
	| 'hidden'
	| 'readonly';

export interface CrudColumn {
	api_name: string;
	display_name: string;
	current_sort: SortOrder | null;
	edit_type: CrudEditType;
}
