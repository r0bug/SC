/**
 * Generate an array of page numbers to display in pagination,
 * with ellipsis gaps represented by the omission of numbers.
 */
export function getPageNumbers(
	currentPage: number,
	totalPages: number,
	maxVisible: number = 7
): number[] {
	if (totalPages <= maxVisible) {
		return Array.from({ length: totalPages }, (_, i) => i + 1);
	}

	const half = Math.floor(maxVisible / 2);
	let start = Math.max(currentPage - half, 1);
	let end = start + maxVisible - 1;

	if (end > totalPages) {
		end = totalPages;
		start = Math.max(end - maxVisible + 1, 1);
	}

	return Array.from({ length: end - start + 1 }, (_, i) => start + i);
}
