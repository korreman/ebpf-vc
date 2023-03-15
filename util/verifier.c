#define _GNU_SOURCE
#include <err.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/socket.h>
#include <sys/syscall.h>
#include <unistd.h>
#include <stdlib.h>

#include <bpf/bpf.h>
#include "bpf_insn.h"


// Wrapper for BPF syscall
static long bpf_(int cmd, union bpf_attr *attrs) {
  return syscall(__NR_bpf, cmd, attrs, sizeof(*attrs));
}

// ----- Shared memory interface -----
// Maps are a generic interface
static uint32_t map_create(uint64_t value_size) {
  union bpf_attr create_map_attrs = {
    .map_type    = BPF_MAP_TYPE_ARRAY, // map implementation
    .key_size    = 4,
    .value_size  = value_size,
    .max_entries = 16
  };

  uint32_t map_fd = bpf_(BPF_MAP_CREATE, &create_map_attrs);
  if (map_fd == -1)
    err(1, "map create");

  return map_fd;
}

static void array_set(int mapfd, uint32_t key, void *value) {
  union bpf_attr attr = {
    .map_fd = mapfd,
    .key    = (uint64_t)&key,
    .value  = (uint64_t)value,
    .flags  = BPF_ANY,
  };

  long res = bpf_(BPF_MAP_UPDATE_ELEM, &attr);
  if (res)
    err(1, "map update elem");
}

static uint32_t array_get(int map_fd, uint32_t key) {
  uint64_t ret_val;
  union bpf_attr lookup_map = {
    .map_fd = map_fd,
    .key    = (uint64_t)&key,
    .value  = (uint64_t)&ret_val
  };

  int res = bpf_(BPF_MAP_LOOKUP_ELEM, &lookup_map);
  if (res)
    err(1, "map lookup elem");
  return ret_val;
}

// Simple sized buffer.
typedef struct Buffer {
    char* data;
    long size;
} Buffer;

Buffer read_file(char* path) {
    char *source = NULL;
    long bufsize = 0;
    FILE *fp = fopen(path, "r");
    if (fp != NULL) {
        /* Go to the end of the file. */
        if (fseek(fp, 0L, SEEK_END) == 0) {
            /* Get the size of the file. */
            bufsize = ftell(fp);
            if (bufsize <= 0) { err(1, "Error reading file"); }

            /* Allocate our buffer to that size. */
            source = malloc(sizeof(char) * (bufsize + 1));

            /* Go back to the start of the file. */
            if (fseek(fp, 0L, SEEK_SET) != 0) { err(1, "Error reading file"); }

            /* Read the entire file into memory. */
            size_t newLen = fread(source, sizeof(char), bufsize, fp);
            if ( ferror( fp ) != 0 ) {
                err(1, "Error reading file");
            } else {
                source[newLen++] = '\0'; /* Just to be safe. */
            }
        } else {
            err(1, "Error reading file");
        }
        fclose(fp);
    } else {
        err(1, "failed to open file");
    }
    Buffer res = {
        .data = source,
        .size = bufsize
    };
    return res;
}

Buffer concat(Buffer a, Buffer b) {
    long new_size = a.size + b.size;
    char* new_data = malloc(new_size);
    memcpy(new_data, a.data, a.size);
    memcpy(new_data + a.size, b.data, b.size);
    Buffer res = {
        .data = new_data,
        .size = new_size
    };
    return res;
}

// ----- Main code -----
int main(int argc, char** argv) {

  if (argc != 2 && argc != 3) {
      fprintf(stderr, "test if an eBPF module is accepted by the verifier\nusage: %s [EBPF BINARY] [--log]\n", argv[0]);
      return 1;
  }

  uint32_t ctx_size = 64;
  printf("Size of input buffer: %d\n", ctx_size);
  uint32_t ctx_map_fd = map_create(ctx_size);
  uint32_t size_map_fd = map_create(8);

  if (size_map_fd == -1 || ctx_map_fd == -1) {
      err(1, "Failed to create maps\n");
  } else {
      printf("Created maps\n");
  }

  // I don't think that this prelude is entirely correct.
  // It should be possible to get the value size through some weird undocumented instruction.
  struct bpf_insn header[] = {
      // Load buffer, put pointer in r9.
      BPF_LD_MAP_FD(BPF_REG_1, ctx_map_fd),
      BPF_MOV64_REG(BPF_REG_2, BPF_REG_10),
      BPF_ALU64_IMM(BPF_ADD, BPF_REG_2, -4),
      BPF_ST_MEM(BPF_W, BPF_REG_2, 0, 0),
      BPF_RAW_INSN(BPF_JMP | BPF_CALL, 0, 0, 0, 1),
      BPF_JMP_IMM(BPF_JNE, BPF_REG_0, 0, +2),
      BPF_MOV64_IMM(BPF_REG_0, 1),
      BPF_EXIT_INSN(),
      BPF_MOV64_REG(BPF_REG_9, BPF_REG_0),
      BPF_MOV64_IMM(BPF_REG_0, 0),

      // Load first integer into r6.
      BPF_LD_MAP_FD(BPF_REG_1, size_map_fd),
      BPF_MOV64_REG(BPF_REG_2, BPF_REG_10),
      BPF_ALU64_IMM(BPF_ADD, BPF_REG_2, -4),
      BPF_ST_MEM(BPF_W, BPF_REG_2, 0, 0), // index = 0
      BPF_RAW_INSN(BPF_JMP | BPF_CALL, 0, 0, 0, 1),
      BPF_JMP_IMM(BPF_JNE, BPF_REG_0, 0, +2),
      BPF_MOV64_IMM(BPF_REG_0, 1),
      BPF_EXIT_INSN(),
      BPF_LDX_MEM(BPF_DW, BPF_REG_6, BPF_REG_0, 0),

      // Load second integer into r7.
      BPF_LD_MAP_FD(BPF_REG_1, size_map_fd),
      BPF_MOV64_REG(BPF_REG_2, BPF_REG_10),
      BPF_ALU64_IMM(BPF_ADD, BPF_REG_2, -4),
      BPF_ST_MEM(BPF_W, BPF_REG_2, 0, 1), // index = 1
      BPF_RAW_INSN(BPF_JMP | BPF_CALL, 0, 0, 0, 1),
      BPF_JMP_IMM(BPF_JNE, BPF_REG_0, 0, +2),
      BPF_MOV64_IMM(BPF_REG_0, 1),
      BPF_EXIT_INSN(),
      BPF_LDX_MEM(BPF_DW, BPF_REG_7, BPF_REG_0, 0),

      // Load third integer into r8.
      BPF_LD_MAP_FD(BPF_REG_1, size_map_fd),
      BPF_MOV64_REG(BPF_REG_2, BPF_REG_10),
      BPF_ALU64_IMM(BPF_ADD, BPF_REG_2, -4),
      BPF_ST_MEM(BPF_W, BPF_REG_2, 0, 1), // index = 2
      BPF_RAW_INSN(BPF_JMP | BPF_CALL, 0, 0, 0, 1),
      BPF_JMP_IMM(BPF_JNE, BPF_REG_0, 0, +2),
      BPF_MOV64_IMM(BPF_REG_0, 1),
      BPF_EXIT_INSN(),
      BPF_LDX_MEM(BPF_DW, BPF_REG_8, BPF_REG_0, 0),

      // Put buffer pointer in r1, size in r2.
      BPF_MOV64_REG(BPF_REG_1, BPF_REG_9),
      BPF_MOV64_REG(BPF_REG_2, BPF_REG_6),
      BPF_MOV64_REG(BPF_REG_3, BPF_REG_7),
      BPF_MOV64_REG(BPF_REG_4, BPF_REG_8),
  };

  Buffer header_buf = {
      .data = (char*)header,
      .size = sizeof(header),
  };
  Buffer prog_buf = read_file(argv[1]);
  printf("Loaded program buffer (%ld bytes)\n", prog_buf.size);

  Buffer full_buf = concat(header_buf, prog_buf);

  // load the program
  char verifier_log[100000];
  union bpf_attr create_prog_attrs = {
    .prog_type = BPF_PROG_TYPE_SOCKET_FILTER,
    .insn_cnt = full_buf.size / 8,
    .insns = (uint64_t)full_buf.data,
    .license = (uint64_t) "GPL",
    .log_level = 2,
    .log_size = sizeof(verifier_log),
    .log_buf = (uint64_t)verifier_log
  };
  int progfd = bpf_(BPF_PROG_LOAD, &create_prog_attrs);

  if (argc == 3 && strcmp(argv[2], "--log") == 0) {
    puts(verifier_log);
  }

  // If verification doesn't accept the program, this is where we get the error
  if (progfd == -1) {
    perror("Program denied\n\n");
    return 1;
  }
  printf("Program accepted\n");
  return 0;
}
