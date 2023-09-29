#define STACK_SIZE 4096
#define NUMLOCALS 256

typedef struct {
	uint32_t pc;
	uint32_t* stack;
	uint32_t* locals;
	uint16_t sp;
	unsigned char* prg;
} PRG;