export const CHIP_COLORS = [
	'#6366f1', '#8b5cf6', '#ec4899', '#ef4444', '#f97316',
	'#eab308', '#22c55e', '#14b8a6', '#06b6d4', '#3b82f6'
];

export function hashColor(name: string): string {
	let hash = 0;
	for (let i = 0; i < name.length; i++) {
		hash = name.charCodeAt(i) + ((hash << 5) - hash);
	}
	return CHIP_COLORS[Math.abs(hash) % CHIP_COLORS.length];
}
