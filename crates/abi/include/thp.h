#ifndef THP_H
#define THP_H

#include <stddef.h>
#include <stdint.h>

#define THP_ABI_VERSION 1u

typedef struct ThpEngine ThpEngine;
typedef struct ThpPreparedProject ThpPreparedProject;

typedef struct {
    uint8_t *pointer;
    size_t length;
    size_t capacity;
} ThpBuffer;

typedef struct {
    uint32_t status;
    ThpBuffer output;
    ThpBuffer error;
} ThpRunResult;

ThpEngine *thp_engine_new(uint64_t max_instructions);

typedef struct {
    uint32_t abi_version;
    size_t struct_size;
    uint64_t max_instructions;
    uint64_t max_execution_ms;
    uint64_t max_heap_bytes;
    uint64_t max_input_bytes;
    uint64_t max_input_ms;
    uint64_t max_stack_depth;
    uint64_t max_open_handles;
} ThpEngineOptions;

typedef struct {
    ThpEngine *engine;
    ThpBuffer error;
} ThpEngineCreateResult;

typedef int32_t (*ThpInputReadFn)(
    void *user_data,
    uint8_t *buffer,
    size_t capacity,
    size_t *length
);
typedef int32_t (*ThpOutputWriteFn)(
    void *user_data,
    const uint8_t *buffer,
    size_t length
);

typedef struct {
    uint32_t abi_version;
    size_t struct_size;
    ThpInputReadFn input_read;
    ThpOutputWriteFn output_write;
    uint64_t declared_input_length;
    void *user_data;
} ThpIo;

typedef struct {
    uint32_t status;
    ThpBuffer error;
    uint64_t input_bytes;
    uint64_t output_bytes;
} ThpStreamingResult;

ThpEngineCreateResult thp_engine_new_with_options(const ThpEngineOptions *options);
void thp_engine_free(ThpEngine *engine);
ThpRunResult thp_engine_run(
    ThpEngine *engine,
    const uint8_t *path,
    size_t path_length,
    const uint8_t *source,
    size_t source_length
);
ThpStreamingResult thp_engine_run_io(
    ThpEngine *engine,
    const uint8_t *path,
    size_t path_length,
    const uint8_t *source,
    size_t source_length,
    const ThpIo *io
);
void thp_buffer_free(ThpBuffer buffer);

typedef struct {
    const uint8_t *pointer;
    size_t length;
} ThpBorrowedBuffer;

typedef struct {
    ThpBorrowedBuffer module_id;
    ThpBorrowedBuffer path;
    ThpBorrowedBuffer expected_namespace;
    uint8_t is_entry;
} ThpModuleDescriptor;

typedef int32_t (*ThpModuleEnumerateFn)(
    void *user_data,
    size_t index,
    ThpModuleDescriptor *descriptor
);
typedef int32_t (*ThpModuleLoadFn)(
    void *user_data,
    ThpBorrowedBuffer module_id,
    ThpBorrowedBuffer *source
);

typedef struct {
    uint32_t abi_version;
    size_t struct_size;
    ThpModuleEnumerateFn enumerate;
    ThpModuleLoadFn load;
    void *user_data;
} ThpModuleProvider;

typedef struct {
    uint32_t abi_version;
    size_t struct_size;
    ThpBorrowedBuffer project_root;
    ThpBorrowedBuffer entry;
    ThpBorrowedBuffer target;
} ThpProjectOptions;

typedef struct {
    uint32_t status;
    ThpPreparedProject *project;
    ThpBuffer error;
} ThpPrepareResult;

ThpPrepareResult thp_engine_prepare_project(
    ThpEngine *engine,
    const ThpModuleProvider *provider,
    const ThpProjectOptions *options
);
ThpRunResult thp_engine_run_prepared(
    ThpEngine *engine,
    const ThpPreparedProject *project
);
ThpStreamingResult thp_engine_run_prepared_io(
    ThpEngine *engine,
    const ThpPreparedProject *project,
    const ThpIo *io
);
void thp_prepared_project_free(ThpPreparedProject *project);

typedef void (*ThpLogFn)(
    uint32_t level,
    const uint8_t *message,
    size_t length,
    void *user_data
);

/* Opaque values belong to one synchronous native call. All returned handles,
 * including clones and constructed values, expire when the callback returns.
 * A callback returns one live handle through result. It must never free one.
 * Borrowed byte buffers remain readable only until the callback returns. */
typedef struct ThpCall ThpCall;
typedef struct ThpValue ThpValue;

typedef int32_t (*ThpNativeCallFn)(
    void *user_data,
    ThpCall *call,
    const ThpValue *const *arguments,
    size_t count,
    const ThpValue **result
);

typedef struct {
    size_t struct_size;
    ThpBorrowedBuffer name;
    const ThpBorrowedBuffer *parameters;
    size_t parameter_count;
    ThpBorrowedBuffer result;
    ThpNativeCallFn callback;
    void *user_data;
} ThpNativeFunction;

typedef int32_t (*ThpRegisterFunctionFn)(void *user_data, const ThpNativeFunction *function);

typedef struct {
    uint32_t abi_version;
    size_t struct_size;
    ThpLogFn log;
    void *user_data;
    ThpRegisterFunctionFn register_function;
    /* Kinds: null=0, int=1, float=2, bool=3, string=4, vector=5,
     * map=6, object=7, callable=8, stream=9, exception=10; -1 invalid. */
    int32_t (*value_kind)(ThpCall *, const ThpValue *);
    int32_t (*value_get_int)(ThpCall *, const ThpValue *, int64_t *);
    int32_t (*value_get_float)(ThpCall *, const ThpValue *, double *);
    int32_t (*value_get_bool)(ThpCall *, const ThpValue *, uint8_t *);
    int32_t (*value_get_bytes)(ThpCall *, const ThpValue *, ThpBorrowedBuffer *);
    int32_t (*value_count)(ThpCall *, const ThpValue *, size_t *);
    const ThpValue *(*value_item)(ThpCall *, const ThpValue *, size_t);
    const ThpValue *(*value_map_key)(ThpCall *, const ThpValue *, size_t);
    const ThpValue *(*value_map_value)(ThpCall *, const ThpValue *, size_t);
    const ThpValue *(*value_property)(ThpCall *, const ThpValue *, size_t);
    const ThpValue *(*value_new_null)(ThpCall *);
    const ThpValue *(*value_new_int)(ThpCall *, int64_t);
    const ThpValue *(*value_new_float)(ThpCall *, double);
    const ThpValue *(*value_new_bool)(ThpCall *, uint8_t);
    const ThpValue *(*value_new_bytes)(ThpCall *, ThpBorrowedBuffer);
    const ThpValue *(*value_clone)(ThpCall *, const ThpValue *);
    const ThpValue *(*value_new_vector)(
        ThpCall *, ThpBorrowedBuffer element_type,
        const ThpValue *const *values, size_t count
    );
    const ThpValue *(*value_new_map)(
        ThpCall *, ThpBorrowedBuffer key_type, ThpBorrowedBuffer value_type,
        const ThpValue *const *keys, const ThpValue *const *values, size_t count
    );
    int32_t (*fail)(ThpCall *, ThpBorrowedBuffer message);
    /* Canonical extension-owned TOML, borrowed only during initialize(). */
    ThpBorrowedBuffer extension_config;
} ThpHost;

typedef int32_t (*ThpExtensionInitFn)(const ThpHost *host);
typedef void (*ThpExtensionShutdownFn)(void);

typedef struct {
    uint32_t abi_version;
    size_t struct_size;
    const char *name;
    ThpExtensionInitFn initialize;
    ThpExtensionShutdownFn shutdown;
} ThpExtension;

typedef const ThpExtension *(*ThpExtensionEntryFn)(void);

#endif
