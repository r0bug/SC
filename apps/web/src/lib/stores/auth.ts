import { writable, derived } from 'svelte/store';
import { api } from '$lib/api/api';
import type { User } from '$lib/api/types';
import { goto } from '$app/navigation';

interface AuthState {
	user: User | null;
	token: string | null;
	loading: boolean;
	error: string | null;
}

function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>({
		user: null,
		token: null,
		loading: false,
		error: null
	});

	return {
		subscribe,

		async login(email: string, password: string) {
			update(s => ({ ...s, loading: true, error: null }));
			try {
				const response = await api.login(email, password);
				set({
					user: response.user,
					token: response.token,
					loading: false,
					error: null
				});
				goto('/dashboard');
			} catch (error: any) {
				update(s => ({ ...s, loading: false, error: error.message }));
				throw error;
			}
		},

		async signup(email: string, password: string, name: string) {
			update(s => ({ ...s, loading: true, error: null }));
			try {
				const response = await api.signup(email, password, name);
				set({
					user: response.user,
					token: response.token,
					loading: false,
					error: null
				});
				goto('/dashboard');
			} catch (error: any) {
				update(s => ({ ...s, loading: false, error: error.message }));
				throw error;
			}
		},

		logout() {
			api.logout();
			set({
				user: null,
				token: null,
				loading: false,
				error: null
			});
			goto('/auth/login');
		},

		async checkAuth() {
			const token = localStorage.getItem('auth_token');
			if (token) {
				try {
					// Validate token by fetching user data
					const response = await api.getDashboard();
					update(s => ({ ...s, token }));
				} catch (error) {
					// Token invalid, clear it
					localStorage.removeItem('auth_token');
				}
			}
		}
	};
}

export const auth = createAuthStore();
export const isAuthenticated = derived(auth, $auth => !!$auth.token);