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

typedef struct {
    uint32_t abi_version;
    size_t struct_size;
    ThpLogFn log;
    void *user_data;
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
