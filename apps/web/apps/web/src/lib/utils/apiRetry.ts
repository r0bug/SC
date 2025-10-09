export interface RetryOptions {
	maxRetries?: number;
	initialDelay?: number;
	maxDelay?: number;
	backoffMultiplier?: number;
	retryableStatuses?: number[];
}

const DEFAULT_OPTIONS: Required<RetryOptions> = {
	maxRetries: 3,
	initialDelay: 1000,
	maxDelay: 10000,
	backoffMultiplier: 2,
	retryableStatuses: [408, 429, 500, 502, 503, 504]
};

function delay(ms: number): Promise<void> {
	return new Promise(resolve => setTimeout(resolve, ms));
}

function shouldRetry(error: any, status?: number, attempt?: number, maxRetries?: number): boolean {
	if (attempt !== undefined && maxRetries !== undefined && attempt >= maxRetries) {
		return false;
	}

	if (status && DEFAULT_OPTIONS.retryableStatuses.includes(status)) {
		return true;
	}

	if (error && error.name === 'TypeError' && error.message && error.message.includes('fetch')) {
		return true;
	}

	return false;
}

export async function fetchWithRetry(
	url: string,
	options: RequestInit = {},
	retryOptions: RetryOptions = {}
): Promise<Response> {
	const opts = { ...DEFAULT_OPTIONS, ...retryOptions };
	let lastError: Error | null = null;
	let currentDelay = opts.initialDelay;

	for (let attempt = 0; attempt <= opts.maxRetries; attempt++) {
		try {
			const response = await fetch(url, options);

			if (response.ok) {
				return response;
			}

			if (!shouldRetry(null, response.status, attempt, opts.maxRetries)) {
				return response;
			}

			lastError = new Error(\`HTTP \${response.status}: \${response.statusText}\`);

		} catch (error) {
			lastError = error instanceof Error ? error : new Error(String(error));

			if (!shouldRetry(error, undefined, attempt, opts.maxRetries)) {
				throw lastError;
			}
		}

		if (attempt < opts.maxRetries) {
			await delay(Math.min(currentDelay, opts.maxDelay));
			currentDelay *= opts.backoffMultiplier;
		}
	}

	throw lastError || new Error('Max retries exceeded');
}

export async function apiCall<T>(
	url: string,
	options: RequestInit = {},
	retryOptions: RetryOptions = {}
): Promise<T> {
	const response = await fetchWithRetry(url, options, retryOptions);

	if (!response.ok) {
		const errorText = await response.text().catch(() => response.statusText);
		throw new Error(\`API Error \${response.status}: \${errorText}\`);
	}

	return response.json();
}
