/**
 * Client-Side End-to-End Encryption Library
 *
 * This module provides zero-knowledge encryption for sensitive entity data.
 * The server never has access to decryption keys or plaintext data.
 *
 * Encryption Stack:
 * - AES-256-GCM for symmetric encryption
 * - PBKDF2 (100,000 iterations) for password-based key derivation
 * - Random 96-bit nonces for each encryption operation
 * - SHA-256 for integrity checksums
 *
 * @module crypto
 */

/**
 * Represents an encrypted blob with nonce and ciphertext
 */
export interface EncryptedBlob {
  /** 96-bit nonce (12 bytes) for AES-GCM */
  nonce: Uint8Array;
  /** Encrypted data with authentication tag */
  ciphertext: Uint8Array;
}

/**
 * Server response when storing encrypted entities
 */
export interface StoreEncryptedEntityResponse {
  id: string;
  user_id: string;
  entity_type: string;
  checksum: string;
  created_at: string;
}

/**
 * Encrypted entity retrieved from server
 */
export interface EncryptedEntity {
  id: string;
  user_id: string;
  entity_type: string;
  encrypted_data: string; // Base64-encoded EncryptedBlob
  checksum: string;
  created_at: string;
  updated_at: string;
  deleted_at?: string;
}

/**
 * Client-side cryptography operations using Web Crypto API
 */
export class CryptoClient {
  private cryptoKey: CryptoKey | null = null;
  private salt: Uint8Array | null = null;

  /**
   * Initialize the crypto client with a password-derived key
   *
   * @param password - User's master password
   * @param salt - Base64-encoded salt (12 bytes). If not provided, generates a new one.
   * @returns Base64-encoded salt (for storage)
   */
  async initializeKey(password: string, salt?: string): Promise<string> {
    // Decode or generate salt
    if (salt) {
      this.salt = this.base64ToUint8Array(salt);
    } else {
      this.salt = crypto.getRandomValues(new Uint8Array(12));
    }

    // Import password as key material
    const passwordKey = await crypto.subtle.importKey(
      'raw',
      new TextEncoder().encode(password),
      'PBKDF2',
      false,
      ['deriveBits', 'deriveKey']
    );

    // Derive AES-256-GCM key using PBKDF2
    this.cryptoKey = await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt: this.salt,
        iterations: 100000,
        hash: 'SHA-256'
      },
      passwordKey,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt']
    );

    return this.uint8ArrayToBase64(this.salt);
  }

  /**
   * Encrypt an entity with AES-256-GCM
   *
   * @param entity - Plain entity object
   * @returns EncryptedBlob with nonce and ciphertext
   * @throws Error if key not initialized
   */
  async encryptEntity<T>(entity: T): Promise<EncryptedBlob> {
    if (!this.cryptoKey) {
      throw new Error('Crypto key not initialized. Call initializeKey() first.');
    }

    // Serialize entity to JSON
    const plaintext = new TextEncoder().encode(JSON.stringify(entity));

    // Generate random nonce (96 bits / 12 bytes for AES-GCM)
    const nonce = crypto.getRandomValues(new Uint8Array(12));

    // Encrypt
    const ciphertext = await crypto.subtle.encrypt(
      {
        name: 'AES-GCM',
        iv: nonce
      },
      this.cryptoKey,
      plaintext
    );

    return {
      nonce,
      ciphertext: new Uint8Array(ciphertext)
    };
  }

  /**
   * Decrypt an EncryptedBlob
   *
   * @param blob - Encrypted blob with nonce and ciphertext
   * @returns Decrypted entity object
   * @throws Error if decryption fails or key not initialized
   */
  async decryptEntity<T>(blob: EncryptedBlob): Promise<T> {
    if (!this.cryptoKey) {
      throw new Error('Crypto key not initialized. Call initializeKey() first.');
    }

    // Decrypt
    const plaintext = await crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv: blob.nonce
      },
      this.cryptoKey,
      blob.ciphertext
    );

    // Parse JSON
    const json = new TextDecoder().decode(plaintext);
    return JSON.parse(json) as T;
  }

  /**
   * Serialize EncryptedBlob to base64 for transmission
   *
   * @param blob - Encrypted blob
   * @returns Base64-encoded string
   */
  serializeBlob(blob: EncryptedBlob): string {
    // Format: <nonce_base64>.<ciphertext_base64>
    const nonceB64 = this.uint8ArrayToBase64(blob.nonce);
    const ciphertextB64 = this.uint8ArrayToBase64(blob.ciphertext);
    return `${nonceB64}.${ciphertextB64}`;
  }

  /**
   * Deserialize base64 string to EncryptedBlob
   *
   * @param serialized - Base64-encoded blob
   * @returns EncryptedBlob object
   */
  deserializeBlob(serialized: string): EncryptedBlob {
    const [nonceB64, ciphertextB64] = serialized.split('.');
    if (!nonceB64 || !ciphertextB64) {
      throw new Error('Invalid encrypted blob format');
    }

    return {
      nonce: this.base64ToUint8Array(nonceB64),
      ciphertext: this.base64ToUint8Array(ciphertextB64)
    };
  }

  /**
   * Calculate SHA-256 checksum of encrypted data
   *
   * @param data - Data to hash
   * @returns Hex-encoded SHA-256 hash
   */
  async calculateChecksum(data: string): Promise<string> {
    const buffer = new TextEncoder().encode(data);
    const hashBuffer = await crypto.subtle.digest('SHA-256', buffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  }

  /**
   * Convert Uint8Array to base64 string
   */
  private uint8ArrayToBase64(array: Uint8Array): string {
    return btoa(String.fromCharCode(...array));
  }

  /**
   * Convert base64 string to Uint8Array
   */
  private base64ToUint8Array(base64: string): Uint8Array {
    const binaryString = atob(base64);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    return bytes;
  }
}

/**
 * API client for encrypted entity operations
 */
export class EncryptedEntityClient {
  private baseUrl: string;
  private crypto: CryptoClient;

  /**
   * @param baseUrl - API base URL (e.g., 'http://localhost:3000')
   * @param crypto - Initialized CryptoClient instance
   */
  constructor(baseUrl: string, crypto: CryptoClient) {
    this.baseUrl = baseUrl;
    this.crypto = crypto;
  }

  /**
   * Store an encrypted entity on the server
   *
   * @param userId - User ID
   * @param entityType - Type of entity ('contact', 'note', 'project', 'calendar_event')
   * @param entity - Plain entity object
   * @param id - Optional entity ID (generates UUID if not provided)
   * @returns Server response with entity metadata
   */
  async storeEntity<T>(
    userId: string,
    entityType: string,
    entity: T,
    id?: string
  ): Promise<StoreEncryptedEntityResponse> {
    // Encrypt entity
    const blob = await this.crypto.encryptEntity(entity);
    const encryptedData = this.crypto.serializeBlob(blob);

    // Calculate checksum
    const checksum = await this.crypto.calculateChecksum(encryptedData);

    // Prepare request
    const requestBody = {
      id: id || crypto.randomUUID(),
      entity_type: entityType,
      encrypted_data: encryptedData,
      checksum
    };

    // Send to server
    const response = await fetch(`${this.baseUrl}/api/encrypted/${userId}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(requestBody)
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to store encrypted entity: ${error}`);
    }

    return response.json();
  }

  /**
   * Retrieve and decrypt an entity from the server
   *
   * @param userId - User ID
   * @param entityId - Entity ID
   * @returns Decrypted entity object
   */
  async getEntity<T>(userId: string, entityId: string): Promise<T> {
    const response = await fetch(
      `${this.baseUrl}/api/encrypted/${userId}/${entityId}`,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to get encrypted entity: ${error}`);
    }

    const encryptedEntity: EncryptedEntity = await response.json();

    // Verify checksum
    const calculatedChecksum = await this.crypto.calculateChecksum(
      encryptedEntity.encrypted_data
    );
    if (calculatedChecksum !== encryptedEntity.checksum) {
      throw new Error('Checksum verification failed - data may be corrupted');
    }

    // Decrypt
    const blob = this.crypto.deserializeBlob(encryptedEntity.encrypted_data);
    return this.crypto.decryptEntity<T>(blob);
  }

  /**
   * List all encrypted entities for a user (optionally filtered by type)
   *
   * @param userId - User ID
   * @param entityType - Optional entity type filter
   * @returns Array of encrypted entity metadata (not decrypted)
   */
  async listEntities(
    userId: string,
    entityType?: string
  ): Promise<EncryptedEntity[]> {
    const url = new URL(`${this.baseUrl}/api/encrypted/${userId}`);
    if (entityType) {
      url.searchParams.set('entity_type', entityType);
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json'
      }
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to list encrypted entities: ${error}`);
    }

    return response.json();
  }

  /**
   * Update an encrypted entity
   *
   * @param userId - User ID
   * @param entityId - Entity ID
   * @param entity - Updated plain entity object
   * @returns Server response
   */
  async updateEntity<T>(
    userId: string,
    entityId: string,
    entity: T
  ): Promise<StoreEncryptedEntityResponse> {
    // Encrypt entity
    const blob = await this.crypto.encryptEntity(entity);
    const encryptedData = this.crypto.serializeBlob(blob);

    // Calculate checksum
    const checksum = await this.crypto.calculateChecksum(encryptedData);

    // Prepare request
    const requestBody = {
      encrypted_data: encryptedData,
      checksum
    };

    // Send to server
    const response = await fetch(
      `${this.baseUrl}/api/encrypted/${userId}/${entityId}`,
      {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(requestBody)
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to update encrypted entity: ${error}`);
    }

    return response.json();
  }

  /**
   * Delete an encrypted entity (soft delete)
   *
   * @param userId - User ID
   * @param entityId - Entity ID
   */
  async deleteEntity(userId: string, entityId: string): Promise<void> {
    const response = await fetch(
      `${this.baseUrl}/api/encrypted/${userId}/${entityId}`,
      {
        method: 'DELETE',
        headers: {
          'Content-Type': 'application/json'
        }
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Failed to delete encrypted entity: ${error}`);
    }
  }
}

/**
 * Example usage:
 *
 * ```typescript
 * // Initialize crypto client
 * const crypto = new CryptoClient();
 * const salt = await crypto.initializeKey('user-password-123');
 *
 * // Create API client
 * const client = new EncryptedEntityClient('http://localhost:3000', crypto);
 *
 * // Store encrypted contact
 * const contact = { name: 'John Doe', email: 'john@example.com' };
 * const response = await client.storeEntity('user-1', 'contact', contact);
 *
 * // Retrieve and decrypt
 * const decrypted = await client.getEntity<typeof contact>('user-1', response.id);
 * console.log(decrypted.name); // 'John Doe'
 * ```
 */
