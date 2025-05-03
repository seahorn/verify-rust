#include <stdint.h>
#include <stddef.h>
#include <seahorn/seahorn.h>

// The definition for malloc, free etc are provided separately (in seahorn.c)
static uint8_t *__rust_alloc(size_t size, size_t align) {
    sassert(size > 0);
    // align must be a power of two and a multiple of sizeof(void *)
    // See https://linux.die.net/man/3/memalign
    // TODO: add this check
    return malloc(size);
}

static uint8_t *__rust_alloc_zeroed(size_t size, size_t align) {
    sassert(size > 0);
    // align must be a power of two and a multiple of sizeof(void *)
    // See https://linux.die.net/man/3/memalign
    // TODO: add this check
    return calloc(1, size);
}

static uint8_t *__rust_realloc(uint8_t *ptr, size_t old_size, size_t align, size_t new_size) {
    // Passing a NULL pointer is undefined behavior
    sassert(ptr != 0);

    // Passing a new_size of 0 is undefined behavior
    sassert(new_size > 0);

    // align must be a power of two and a multiple of sizeof(void *)
    // See https://linux.die.net/man/3/memalign
    // TODO: add this check

    uint8_t *result = malloc(new_size);
    if (result) {
        size_t bytes_to_copy = new_size < old_size ? new_size : old_size;
        memcpy(result, ptr, bytes_to_copy);
        free(ptr);
    }
    return result;
}

static void __rust_dealloc(uint8_t *ptr, size_t size, size_t align) {

    // align must be a power of two and a multiple of sizeof(void *)
    // See https://linux.die.net/man/3/memalign
    // TODO: add this check

    // rust_dealloc must be called on an object whose allocated size matches its layout
    // TODO: expose ptr.size from slot1 bvopsem2 machine and 
    //       assert (ptr.size == size)
    free(ptr);
}
