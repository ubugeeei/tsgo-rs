#ifndef CORSA_BIND_H
#define CORSA_BIND_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct CorsaBindApiClient CorsaBindApiClient;
typedef struct CorsaBindVirtualDocument CorsaBindVirtualDocument;

typedef struct CorsaBindBytes {
  uint8_t *ptr;
  size_t len;
} CorsaBindBytes;

const char *corsa_bind_version(void);
const char *corsa_bind_last_error_message(void);

void corsa_bind_string_free(char *ptr);
void corsa_bind_bytes_free(struct CorsaBindBytes bytes);

int corsa_bind_is_unsafe_assignment(const char *input_json);
int corsa_bind_is_unsafe_return(const char *input_json);

CorsaBindApiClient *corsa_bind_api_client_new(const char *options_json);
void corsa_bind_api_client_free(CorsaBindApiClient *client);
char *corsa_bind_api_client_initialize_json(CorsaBindApiClient *client);
char *corsa_bind_api_client_parse_config_file_json(CorsaBindApiClient *client, const char *file);
char *corsa_bind_api_client_update_snapshot_json(
  CorsaBindApiClient *client,
  const char *params_json
);
struct CorsaBindBytes corsa_bind_api_client_get_source_file(
  CorsaBindApiClient *client,
  const char *snapshot,
  const char *project,
  const char *file
);
char *corsa_bind_api_client_get_string_type_json(
  CorsaBindApiClient *client,
  const char *snapshot,
  const char *project
);
char *corsa_bind_api_client_type_to_string(
  CorsaBindApiClient *client,
  const char *snapshot,
  const char *project,
  const char *type_handle,
  const char *location,
  int32_t flags,
  int has_flags
);
char *corsa_bind_api_client_call_json(
  CorsaBindApiClient *client,
  const char *method,
  const char *params_json
);
struct CorsaBindBytes corsa_bind_api_client_call_binary(
  CorsaBindApiClient *client,
  const char *method,
  const char *params_json
);
int corsa_bind_api_client_release_handle(CorsaBindApiClient *client, const char *handle);
int corsa_bind_api_client_close(CorsaBindApiClient *client);

CorsaBindVirtualDocument *corsa_bind_virtual_document_untitled(
  const char *path,
  const char *language_id,
  const char *text
);
CorsaBindVirtualDocument *corsa_bind_virtual_document_in_memory(
  const char *authority,
  const char *path,
  const char *language_id,
  const char *text
);
void corsa_bind_virtual_document_free(CorsaBindVirtualDocument *document);
char *corsa_bind_virtual_document_uri(const CorsaBindVirtualDocument *document);
char *corsa_bind_virtual_document_language_id(const CorsaBindVirtualDocument *document);
int32_t corsa_bind_virtual_document_version(const CorsaBindVirtualDocument *document);
char *corsa_bind_virtual_document_text(const CorsaBindVirtualDocument *document);
char *corsa_bind_virtual_document_state_json(const CorsaBindVirtualDocument *document);
int corsa_bind_virtual_document_replace(CorsaBindVirtualDocument *document, const char *text);
char *corsa_bind_virtual_document_apply_changes_json(
  CorsaBindVirtualDocument *document,
  const char *changes_json
);

#ifdef __cplusplus
}
#endif

#endif
