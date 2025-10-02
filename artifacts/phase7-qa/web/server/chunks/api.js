class ApiError extends Error {
  constructor(message, code, status, details) {
    super(message);
    this.code = code;
    this.status = status;
    this.details = details;
    this.name = "ApiError";
  }
  isRetryable() {
    return this.status >= 500 || this.code === "NETWORK_ERROR" || this.code === "RATE_LIMIT";
  }
  getUserMessage() {
    return this.message;
  }
}
class ApiClient {
  baseUrl = "/api";
  token = null;
  ws = null;
  wsListeners = /* @__PURE__ */ new Map();
  constructor() {
    if (typeof window !== "undefined") {
      this.token = localStorage.getItem("auth_token");
    }
  }
  // Auth
  async signup(email, password, name) {
    const res = await this.request("/auth/signup", {
      method: "POST",
      body: JSON.stringify({ email, password, name })
    });
    this.token = res.token;
    localStorage.setItem("auth_token", res.token);
    this.connectWebSocket();
    return res;
  }
  async login(email, password) {
    const res = await this.request("/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password })
    });
    this.token = res.token;
    localStorage.setItem("auth_token", res.token);
    this.connectWebSocket();
    return res;
  }
  logout() {
    this.token = null;
    localStorage.removeItem("auth_token");
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
  // Dashboard
  async getDashboard() {
    return this.request("/dashboard");
  }
  // Contacts
  async getContacts(limit = 50, offset = 0) {
    return this.request(`/contacts?limit=${limit}&offset=${offset}`);
  }
  async getContact(id) {
    return this.request(`/contacts/${id}`);
  }
  async createContact(contact) {
    return this.request("/contacts", {
      method: "POST",
      body: JSON.stringify(contact)
    });
  }
  async updateContact(id, updates) {
    return this.request(`/contacts/${id}`, {
      method: "PUT",
      body: JSON.stringify(updates)
    });
  }
  async deleteContact(id) {
    return this.request(`/contacts/${id}`, { method: "DELETE" });
  }
  async searchContacts(query, filters) {
    return this.request("/contacts/search", {
      method: "POST",
      body: JSON.stringify({ query, filters })
    });
  }
  // Groups
  async getGroups() {
    return this.request("/groups");
  }
  async getGroup(id) {
    return this.request(`/groups/${id}`);
  }
  async createGroup(group) {
    return this.request("/groups", {
      method: "POST",
      body: JSON.stringify(group)
    });
  }
  async updateGroup(id, updates) {
    return this.request(`/groups/${id}`, {
      method: "PUT",
      body: JSON.stringify(updates)
    });
  }
  async deleteGroup(id) {
    return this.request(`/groups/${id}`, { method: "DELETE" });
  }
  async addGroupMember(groupId, contactId) {
    return this.request(`/groups/${groupId}/members`, {
      method: "POST",
      body: JSON.stringify({ contact_id: contactId })
    });
  }
  async removeGroupMember(groupId, contactId) {
    return this.request(`/groups/${groupId}/members/${contactId}`, {
      method: "DELETE"
    });
  }
  // Concepts
  async getConcepts() {
    return this.request("/concepts");
  }
  async getConcept(id) {
    return this.request(`/concepts/${id}`);
  }
  async createConcept(concept) {
    return this.request("/concepts", {
      method: "POST",
      body: JSON.stringify(concept)
    });
  }
  async updateConcept(id, updates) {
    return this.request(`/concepts/${id}`, {
      method: "PUT",
      body: JSON.stringify(updates)
    });
  }
  async deleteConcept(id) {
    return this.request(`/concepts/${id}`, { method: "DELETE" });
  }
  // Projects
  async getProjects() {
    return this.request("/projects");
  }
  async getProject(id) {
    return this.request(`/projects/${id}`);
  }
  async createProject(project) {
    return this.request("/projects", {
      method: "POST",
      body: JSON.stringify(project)
    });
  }
  async updateProject(id, updates) {
    return this.request(`/projects/${id}`, {
      method: "PUT",
      body: JSON.stringify(updates)
    });
  }
  async deleteProject(id) {
    return this.request(`/projects/${id}`, { method: "DELETE" });
  }
  // Calendar
  async getEvents(start, end) {
    const params = new URLSearchParams();
    if (start) params.append("start", start);
    if (end) params.append("end", end);
    return this.request(`/calendar/events?${params}`);
  }
  async getEvent(id) {
    return this.request(`/calendar/events/${id}`);
  }
  async createEvent(event) {
    return this.request("/calendar/events", {
      method: "POST",
      body: JSON.stringify(event)
    });
  }
  async updateEvent(id, updates) {
    return this.request(`/calendar/events/${id}`, {
      method: "PUT",
      body: JSON.stringify(updates)
    });
  }
  async deleteEvent(id) {
    return this.request(`/calendar/events/${id}`, { method: "DELETE" });
  }
  // Notes
  async getNotes(entityType, entityId) {
    const params = new URLSearchParams();
    if (entityType) params.append("entity_type", entityType);
    if (entityId) params.append("entity_id", entityId);
    return this.request(`/notes?${params}`);
  }
  async getNote(id) {
    return this.request(`/notes/${id}`);
  }
  async createNote(note) {
    return this.request("/notes", {
      method: "POST",
      body: JSON.stringify(note)
    });
  }
  async updateNote(id, updates) {
    return this.request(`/notes/${id}`, {
      method: "PUT",
      body: JSON.stringify(updates)
    });
  }
  async deleteNote(id) {
    return this.request(`/notes/${id}`, { method: "DELETE" });
  }
  // Attachments
  async getAttachments(entityType, entityId) {
    return this.request(`/attachments?entity_type=${entityType}&entity_id=${entityId}`);
  }
  async uploadAttachment(file, entityType, entityId, uploadedBy) {
    const MAX_FILE_SIZE = 50 * 1024 * 1024;
    if (file.size > MAX_FILE_SIZE) {
      throw new ApiError(
        `File size exceeds maximum limit of 50MB. Your file is ${(file.size / 1024 / 1024).toFixed(1)}MB.`,
        "FILE_TOO_LARGE",
        413
      );
    }
    const allowedTypes = [
      "image/jpeg",
      "image/png",
      "image/gif",
      "image/webp",
      "application/pdf",
      "application/msword",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
      "application/vnd.ms-excel",
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
      "text/plain",
      "text/csv"
    ];
    if (!allowedTypes.includes(file.type)) {
      throw new ApiError(
        `File type "${file.type}" is not supported. Please upload images, PDFs, or common document formats.`,
        "INVALID_FILE_TYPE",
        400
      );
    }
    const formData = new FormData();
    formData.append("file", file);
    formData.append("entity_type", entityType);
    formData.append("entity_id", entityId);
    formData.append("uploaded_by", uploadedBy);
    try {
      const res = await fetch(`${this.baseUrl}/attachments/upload`, {
        method: "POST",
        headers: this.getAuthHeaders(false),
        body: formData
      });
      if (!res.ok) {
        return await this.handleErrorResponse(res, "/attachments/upload");
      }
      const response = await res.json();
      return response.attachment;
    } catch (error) {
      if (error instanceof ApiError) throw error;
      if (error instanceof TypeError && error.message.includes("fetch")) {
        throw new ApiError(
          "Unable to upload file. Please check your internet connection.",
          "NETWORK_ERROR",
          0
        );
      }
      throw new ApiError(
        "Failed to upload attachment. Please try again.",
        "UPLOAD_ERROR",
        500
      );
    }
  }
  async downloadAttachment(id, filename) {
    const res = await fetch(`${this.baseUrl}/attachments/${id}`, {
      headers: this.getAuthHeaders()
    });
    if (!res.ok) throw new Error("Failed to download attachment");
    const blob = await res.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);
  }
  async deleteAttachment(id) {
    return this.request(`/attachments/${id}`, { method: "DELETE" });
  }
  // Tags
  async getTags() {
    return this.request("/tags");
  }
  async createTag(name, color) {
    return this.request("/tags", {
      method: "POST",
      body: JSON.stringify({ name, color })
    });
  }
  async updateTag(id, updates) {
    return this.request(`/tags/${id}`, {
      method: "PUT",
      body: JSON.stringify(updates)
    });
  }
  async deleteTag(id) {
    return this.request(`/tags/${id}`, { method: "DELETE" });
  }
  // Communications
  async getCommunications(contactId) {
    const path = contactId ? `/communications?contact_id=${contactId}` : "/communications";
    return this.request(path);
  }
  async queueCommunication(contactId, method, message, subject, scheduledAt) {
    return this.request("/communications", {
      method: "POST",
      body: JSON.stringify({
        contact_id: contactId,
        method,
        message,
        subject,
        scheduled_at: scheduledAt
      })
    });
  }
  async cancelCommunication(id) {
    return this.request(`/communications/${id}/cancel`, { method: "POST" });
  }
  // Shares
  async getShares() {
    return this.request("/shares");
  }
  async createShare(entityType, entityId, email, permissions) {
    return this.request("/shares", {
      method: "POST",
      body: JSON.stringify({
        entity_type: entityType,
        entity_id: entityId,
        shared_with_email: email,
        permissions
      })
    });
  }
  async acceptShare(id) {
    return this.request(`/shares/${id}/accept`, { method: "POST" });
  }
  async revokeShare(id) {
    return this.request(`/shares/${id}/revoke`, { method: "POST" });
  }
  // AI Insights
  async getInsights(entityType, entityId) {
    const params = new URLSearchParams();
    if (entityType) params.append("entity_type", entityType);
    if (entityId) params.append("entity_id", entityId);
    return this.request(`/ai/insights?${params}`);
  }
  async applyInsight(id) {
    return this.request(`/ai/insights/${id}/apply`, { method: "POST" });
  }
  async feedbackInsight(id, helpful, comment) {
    return this.request(`/ai/insights/${id}/feedback`, {
      method: "POST",
      body: JSON.stringify({ helpful, comment })
    });
  }
  // Search History
  async getSearchHistory() {
    return this.request("/search/history");
  }
  async clearSearchHistory() {
    return this.request("/search/history", { method: "DELETE" });
  }
  // Import
  async previewImport(file, format) {
    const formData = new FormData();
    formData.append("file", file);
    formData.append("format", format);
    const res = await fetch(`${this.baseUrl}/import/preview`, {
      method: "POST",
      headers: this.getAuthHeaders(false),
      body: formData
    });
    if (!res.ok) throw new Error("Failed to preview import");
    return res.json();
  }
  async confirmImport(file, format, mappings) {
    const formData = new FormData();
    formData.append("file", file);
    formData.append("format", format);
    formData.append("mappings", JSON.stringify(mappings));
    const res = await fetch(`${this.baseUrl}/import/confirm`, {
      method: "POST",
      headers: this.getAuthHeaders(false),
      body: formData
    });
    if (!res.ok) throw new Error("Failed to import");
    return res.json();
  }
  // Worker Metrics
  async getWorkerMetrics() {
    return this.request("/metrics/worker");
  }
  async getWorkerHealth() {
    return this.request("/health/worker");
  }
  // Settings
  async getSettings() {
    return this.request("/settings");
  }
  async updateSettings(settings) {
    return this.request("/settings", {
      method: "PUT",
      body: JSON.stringify(settings)
    });
  }
  // WebSocket
  connectWebSocket() {
    if (!this.token || this.ws) return;
    const wsUrl = this.baseUrl.replace("http", "ws").replace("/api", "/ws");
    this.ws = new WebSocket(`${wsUrl}?token=${this.token}`);
    this.ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      const listeners = this.wsListeners.get(data.type) || /* @__PURE__ */ new Set();
      listeners.forEach((listener) => listener(data.payload));
    };
    this.ws.onerror = (error) => {
      console.error("WebSocket error:", error);
    };
    this.ws.onclose = () => {
      this.ws = null;
      setTimeout(() => {
        if (this.token) this.connectWebSocket();
      }, 5e3);
    };
  }
  onWebSocketMessage(type, callback) {
    if (!this.wsListeners.has(type)) {
      this.wsListeners.set(type, /* @__PURE__ */ new Set());
    }
    this.wsListeners.get(type).add(callback);
    return () => {
      this.wsListeners.get(type)?.delete(callback);
    };
  }
  // Private helpers
  async request(path, options = {}) {
    try {
      const res = await fetch(`${this.baseUrl}${path}`, {
        ...options,
        headers: {
          ...this.getAuthHeaders(),
          "Content-Type": "application/json",
          ...options.headers
        }
      });
      if (!res.ok) {
        return await this.handleErrorResponse(res, path);
      }
      const text = await res.text();
      return text ? JSON.parse(text) : null;
    } catch (error) {
      if (error instanceof TypeError && error.message.includes("fetch")) {
        throw new ApiError(
          "Unable to connect to the server. Please check your internet connection.",
          "NETWORK_ERROR",
          0
        );
      }
      throw error;
    }
  }
  async handleErrorResponse(res, path) {
    let errorMessage = `Request failed: ${res.statusText}`;
    let errorCode = "UNKNOWN_ERROR";
    let details = void 0;
    try {
      const contentType = res.headers.get("content-type");
      if (contentType?.includes("application/json")) {
        const errorData = await res.json();
        errorMessage = errorData.message || errorData.error || errorMessage;
        errorCode = errorData.code || errorCode;
        details = errorData.details;
      } else {
        const text = await res.text();
        if (text) errorMessage = text;
      }
    } catch (e) {
    }
    switch (res.status) {
      case 400:
        errorCode = "BAD_REQUEST";
        if (!details) {
          errorMessage = `Invalid request: ${errorMessage}. Please check your input and try again.`;
        }
        break;
      case 401:
        errorCode = "UNAUTHORIZED";
        errorMessage = "Your session has expired. Please log in again.";
        this.logout();
        if (typeof window !== "undefined") {
          window.location.href = "/auth/login";
        }
        break;
      case 403:
        errorCode = "FORBIDDEN";
        errorMessage = "You do not have permission to perform this action.";
        break;
      case 404:
        errorCode = "NOT_FOUND";
        errorMessage = `The requested resource was not found. It may have been deleted or moved.`;
        break;
      case 409:
        errorCode = "CONFLICT";
        errorMessage = `This action conflicts with existing data: ${errorMessage}`;
        break;
      case 413:
        errorCode = "PAYLOAD_TOO_LARGE";
        errorMessage = "The file or data you are trying to upload is too large. Please try a smaller file.";
        break;
      case 422:
        errorCode = "VALIDATION_ERROR";
        if (!details) {
          errorMessage = `Validation failed: ${errorMessage}. Please check your input.`;
        }
        break;
      case 429:
        errorCode = "RATE_LIMIT";
        errorMessage = "Too many requests. Please wait a moment and try again.";
        break;
      case 500:
      case 502:
      case 503:
      case 504:
        errorCode = "SERVER_ERROR";
        errorMessage = "The server encountered an error. Please try again in a moment.";
        break;
    }
    throw new ApiError(errorMessage, errorCode, res.status, details);
  }
  getAuthHeaders(includeContentType = true) {
    const headers = {};
    if (this.token) {
      headers["Authorization"] = `Bearer ${this.token}`;
    }
    if (includeContentType) {
      headers["Content-Type"] = "application/json";
    }
    return headers;
  }
}
const api = new ApiClient();
export {
  api as a
};
