#define STACK_SIZE 4096
#define NUMLOCALS 256

typedef struct {
	uint32_t pc;
	uint32_t* stack;
	uint32_t* locals;
	unsigned int sp; // uint16_t
	unsigned char* prg;
} PRG;

void prg_init(PRG* k);
