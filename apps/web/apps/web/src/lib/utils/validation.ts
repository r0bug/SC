export interface ValidationRule {
	test: (value: any) => boolean;
	message: string;
}

export interface FieldValidation {
	value: any;
	rules: ValidationRule[];
}

export interface ValidationResult {
	valid: boolean;
	errors: Record<string, string>;
}

export function validateField(value: any, rules: ValidationRule[]): string | null {
	for (const rule of rules) {
		if (!rule.test(value)) {
			return rule.message;
		}
	}
	return null;
}

export function validateForm(fields: Record<string, FieldValidation>): ValidationResult {
	const errors: Record<string, string> = {};
	let valid = true;

	for (const [fieldName, { value, rules }] of Object.entries(fields)) {
		const error = validateField(value, rules);
		if (error) {
			errors[fieldName] = error;
			valid = false;
		}
	}

	return { valid, errors };
}

// Common validation rules
export const rules = {
	required: (message = 'This field is required'): ValidationRule => ({
		test: (value) => {
			if (typeof value === 'string') return value.trim().length > 0;
			if (Array.isArray(value)) return value.length > 0;
			return value !== null && value !== undefined;
		},
		message
	}),

	email: (message = 'Please enter a valid email address'): ValidationRule => ({
		test: (value) => {
			if (!value) return true;
			const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
			return emailRegex.test(value);
		},
		message
	}),

	phone: (message = 'Please enter a valid phone number'): ValidationRule => ({
		test: (value) => {
			if (!value) return true;
			const phoneRegex = /^[\d\s\-\+\(\)]+$/;
			return phoneRegex.test(value) && value.replace(/\D/g, '').length >= 10;
		},
		message
	}),

	minLength: (min: number, message?: string): ValidationRule => ({
		test: (value) => {
			if (!value) return true;
			return String(value).length >= min;
		},
		message: message || `Must be at least ${min} characters`
	}),

	maxLength: (max: number, message?: string): ValidationRule => ({
		test: (value) => {
			if (!value) return true;
			return String(value).length <= max;
		},
		message: message || `Must be at most ${max} characters`
	}),

	pattern: (regex: RegExp, message = 'Invalid format'): ValidationRule => ({
		test: (value) => {
			if (!value) return true;
			return regex.test(String(value));
		},
		message
	}),

	url: (message = 'Please enter a valid URL'): ValidationRule => ({
		test: (value) => {
			if (!value) return true;
			try {
				new URL(value);
				return true;
			} catch {
				return false;
			}
		},
		message
	})
};
