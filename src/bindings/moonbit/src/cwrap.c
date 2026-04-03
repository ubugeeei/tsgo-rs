#include <stdint.h>
#include <string.h>

#include "../../c/corsa_bind_c/include/corsa_bind.h"

typedef uint16_t *moonbit_string_t;
extern moonbit_string_t moonbit_make_string(int32_t len, int32_t padding);

moonbit_string_t corsa_bind_mbt_copy_string(const char *ptr) {
  if (ptr == NULL) {
    return moonbit_make_string(0, 0);
  }
  int32_t len = (int32_t)strlen(ptr);
  moonbit_string_t value = moonbit_make_string(len, 0);
  for (int32_t i = 0; i < len; i++) {
    value[i] = (uint16_t)ptr[i];
  }
  return value;
}

moonbit_string_t corsa_bind_mbt_take_string(char *ptr) {
  moonbit_string_t value = corsa_bind_mbt_copy_string(ptr);
  if (ptr != NULL) {
    corsa_bind_string_free(ptr);
  }
  return value;
}
