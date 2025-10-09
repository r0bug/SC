export interface PaginationState {
	currentPage: number;
	pageSize: number;
	totalItems: number;
	totalPages: number;
}

export interface PaginationResult<T> {
	items: T[];
	pagination: PaginationState;
}

export function paginate<T>(
	items: T[],
	currentPage: number = 1,
	pageSize: number = 10
): PaginationResult<T> {
	const totalItems = items.length;
	const totalPages = Math.ceil(totalItems / pageSize);
	const safePage = Math.max(1, Math.min(currentPage, totalPages || 1));
	
	const startIndex = (safePage - 1) * pageSize;
	const endIndex = startIndex + pageSize;
	const paginatedItems = items.slice(startIndex, endIndex);

	return {
		items: paginatedItems,
		pagination: {
			currentPage: safePage,
			pageSize,
			totalItems,
			totalPages
		}
	};
}

export function getPageNumbers(
	currentPage: number,
	totalPages: number,
	maxVisible: number = 7
): number[] {
	if (totalPages <= maxVisible) {
		return Array.from({ length: totalPages }, (_, i) => i + 1);
	}

	const halfVisible = Math.floor(maxVisible / 2);
	let start = Math.max(1, currentPage - halfVisible);
	let end = Math.min(totalPages, start + maxVisible - 1);

	if (end - start < maxVisible - 1) {
		start = Math.max(1, end - maxVisible + 1);
	}

	return Array.from({ length: end - start + 1 }, (_, i) => start + i);
}
