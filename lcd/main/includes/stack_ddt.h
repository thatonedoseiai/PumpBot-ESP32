#ifndef STACK_DDT_H
#define STACK_DDT_H

#include <rotenc.h>
#include "ft2build.h"
#include FT_FREETYPE_H

#define STACK_SIZE 4096
#define NUMLOCALS 256
// #define NUM_ISRS 1
#define MAX_INSTRS_BEFORE_PAUSE 201;

typedef struct {
	uint32_t pc;
	uint32_t* stack;
	uint32_t* locals;
	uint16_t sp;
	unsigned char* prg;
#ifdef NUM_ISRS
	uint32_t isrs[NUM_ISRS];
	unsigned char isrid;
#endif
	uint16_t instrs_execd;
} PRG;

void run(PRG* _p, rotary_encoder_info_t* info, FT_Face typeFace);

void prg_init(PRG* k);

int runprgfile(PRG* k, char* prgname, rotary_encoder_info_t* info, FT_Face typeFace);

void cause_isr(PRG* p, int i);

#endif
