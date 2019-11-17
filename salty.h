#ifndef salty_h
#define salty_h

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define salty_COMPRESSED_Y_LENGTH 32

#define salty_PUBLICKEY_SERIALIZED_LENGTH 32

#define salty_SCALAR_LENGTH 32

#define salty_SECRETKEY_NONCE_LENGTH 32

#define salty_SECRETKEY_SCALAR_LENGTH 32

#define salty_SECRETKEY_SEED_LENGTH 32

#define salty_SECRETKEY_SERIALIZED_LENGTH 32

#define salty_SHA256_LENGTH 64

#define salty_SHA512_LENGTH 64

#define salty_SIGNATURE_SERIALIZED_LENGTH 64

/**
 * Extensible error type for all `salty` operations.
 *
 * This enum has a hidden member, to prevent exhaustively checking for errors.
 */
typedef enum {
  /**
   * Never occurs, simplifies C bindings
   */
  NoError = 0,
  /**
   * Bytes do not correspond to a canonical base field element
   */
  NonCanonicalFieldElement,
  /**
   * Public key bytes invalid
   */
  PublicKeyBytesInvalid,
  /**
   * Signature verification failed
   */
  SignatureInvalid,
  /**
   * Context for prehashed signatures too long
   */
  ContextTooLong,
  _Extensible,
} salty_Error;

/**
 * Generates a public key from a secret seed. Use to verify signatures.
 */
void salty_public_key(const uint8_t (*seed)[salty_SECRETKEY_SEED_LENGTH],
                      uint8_t (*public_key)[salty_PUBLICKEY_SERIALIZED_LENGTH]);

/**
 * Signs the data, based on the keypair generated from the secret seed.
 */
void salty_sign(const uint8_t (*seed)[salty_SECRETKEY_SEED_LENGTH],
                const uint8_t *data_ptr,
                uintptr_t data_len,
                uint8_t (*signature)[salty_SIGNATURE_SERIALIZED_LENGTH]);

/**
 * Signs the prehashed data, based on the keypair generated from the secret seed.
 * An optional context can also be passed (this is recommended).
 */
int8_t salty_sign_prehashed(const uint8_t (*seed)[salty_SECRETKEY_SEED_LENGTH],
                            const uint8_t (*prehashed_data)[salty_SHA512_LENGTH],
                            const uint8_t *context_ptr,
                            uintptr_t context_len,
                            uint8_t (*signature)[salty_SIGNATURE_SERIALIZED_LENGTH]);

/**
 * Verify a presumed signature on the given data.
 */
salty_Error salty_verify(const uint8_t (*public_key)[salty_PUBLICKEY_SERIALIZED_LENGTH],
                         const uint8_t *data_ptr,
                         uintptr_t data_len,
                         const uint8_t (*signature)[salty_SIGNATURE_SERIALIZED_LENGTH]);

/**
 * Verify a presumed signature on the given data.
 */
salty_Error salty_verify_prehashed(const uint8_t (*public_key)[salty_PUBLICKEY_SERIALIZED_LENGTH],
                                   const uint8_t (*prehashed_data)[salty_SHA512_LENGTH],
                                   const uint8_t (*signature)[salty_SIGNATURE_SERIALIZED_LENGTH],
                                   const uint8_t *context_ptr,
                                   uintptr_t context_len);

#endif /* salty_h */
