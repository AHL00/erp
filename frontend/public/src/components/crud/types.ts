import type { SortOrder } from '$bindings/SortOrder';

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
}
