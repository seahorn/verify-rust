/**
 * \brief Dummy implementation of SeaHorn verification builtins
 *
 * Dummy implementation is necessary to build Rust project with LTO.
 * The bodies of these functions are removed by SeaHorn before any
 * optimizataion and verification takes place
 */
#include "seahorn/seahorn.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h> // for free
#include <string.h> // for memset

#define INLINE __attribute__((always_inline))

void __VERIFIER_error(void) { return; }

void __VERIFIER_assume(int i) { return; }

void __SEA_assume(bool b) { return; }

void __VERIFIER_assert(bool pred) { return; }

int8_t sea_nd_i8(void) { return 0; }
uint8_t sea_nd_u8(void) { return 0; }
int16_t sea_nd_i16(void) { return 0; }
uint16_t sea_nd_u16(void) { return 0; }
int32_t sea_nd_i32(void) { return 0; }
uint32_t sea_nd_u32(void) { return 0; }
int64_t sea_nd_i64(void) { return 0; }
uint64_t sea_nd_u64(void) { return 0; }
size_t sea_nd_usize(void) { return 0; }
intptr_t sea_nd_isize(void) { return 0; }
uintptr_t sea_nd_uintptr(void) {return 0; }
intptr_t sea_nd_intptr(void) {return 0; }

bool sea_nd_bool(void) { return true; }

void *sea_realloc(void *ptr, size_t sz) {
  if (ptr)
    free(ptr);
  return malloc(sz);
}

void *calloc(size_t elements, size_t size) {
	if (elements == 0 || size == 0) {
		return NULL;
	}	else {
    size_t bytes = elements*size;
		void *p = malloc(bytes);
    memset(p, 0, bytes);
		return p;
	}
}

void *realloc(void *ptr, size_t sz) { return sea_realloc(ptr, sz); }

void sea_printf( const char* format, ... ) { return; }

INLINE int bcmp(const void *s1, const void *s2, size_t n) {
  size_t i;

  const size_t max_buffer_size = 32;
  const uint8_t *p1;
  const uint8_t *p2;
  p1 = s1;
  p2 = s2;

  if (p1 == p2)
    return 0;
  if (p1 == NULL || p2 == NULL)
    return 1;
  // /* pre-unroll the loop for MAX_BUFFER_SIZE */
  #pragma unroll 32
  for (i = 0; i < max_buffer_size; i++) {
    if (i < n) {
      if (p1[i] != p2[i]) {
        return 1;
      }
    }
  }
  /* unroll the rest, if any (doesn't really work) */
  /* for (i = max_buffer_size; i < n; i++) { */
  /*   if (p1[i] != p2[i]) */
  /*     return 1; */
  /* } */
  return 0;
}
