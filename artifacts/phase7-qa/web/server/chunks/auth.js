import { d as derived, w as writable } from "./index.js";
import { a as api } from "./api.js";
import "@sveltejs/kit/internal";
import "./exports.js";
import "./utils.js";
import "@sveltejs/kit/internal/server";
import "./state.svelte.js";
function goto(url, opts = {}) {
  {
    throw new Error("Cannot call goto(...) on the server");
  }
}
function createAuthStore() {
  const { subscribe, set, update } = writable({
    user: null,
    token: null,
    loading: false,
    error: null
  });
  return {
    subscribe,
    async login(email, password) {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        const response = await api.login(email, password);
        set({
          user: response.user,
          token: response.token,
          loading: false,
          error: null
        });
        goto("/dashboard");
      } catch (error) {
        update((s) => ({ ...s, loading: false, error: error.message }));
        throw error;
      }
    },
    async signup(email, password, name) {
      update((s) => ({ ...s, loading: true, error: null }));
      try {
        const response = await api.signup(email, password, name);
        set({
          user: response.user,
          token: response.token,
          loading: false,
          error: null
        });
        goto("/dashboard");
      } catch (error) {
        update((s) => ({ ...s, loading: false, error: error.message }));
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
      goto();
    },
    async checkAuth() {
      const token = localStorage.getItem("auth_token");
      if (token) {
        try {
          const response = await api.getDashboard();
          update((s) => ({ ...s, token }));
        } catch (error) {
          localStorage.removeItem("auth_token");
        }
      }
    }
  };
}
const auth = createAuthStore();
const isAuthenticated = derived(auth, ($auth) => !!$auth.token);
export {
  isAuthenticated as i
};
