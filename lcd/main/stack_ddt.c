#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include "stack_ddt.h"

// uint32_t pc;
// uint32_t* stack;
// uint32_t* locals;
// uint16_t sp;
// unsigned char* prg;

void run(PRG* _p) {
	const static void* ops[] = {
		&&nop, &&sleep, &&pushi32, &&pushi16, &&iprint, &&fprint, &&ftoi, &&itof,
		&&iadd, &&isub, &&imul, &&idiv, &&fadd, &&fsub, &&fmul, &&fdiv,
		&&pop, &&swap, &&beq, &&bgt, &&blt, &&bge, &&ble, &&bne, &&jmp, &&dup, &&andb, &&orb, &&xorb, &&notb,
		&&store, &&load, &&pushpc, &&writepc, &&pushsp, &&writesp, &&jsr, &&jsrs, &&pushab, &&pusha, &&pushaw,
		&&sprint, &&lsl, &&lsr, &&mod, &&pushi8, &&neg, &&negf, 
		&&c0, &&c1, &&c2, &&c3, &&c4, &&c5, &&c6, &&c7, &&malloca, &&freea, &&indexab, &&indexa, &&indexaw, &&storea,
		&&store0, &&store1, &&store2, &&store3, &&store4, &&load0, &&load1, &&load2, &&load3, &&load4, 
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
		&&halt
	};

	float res;

	#define NEXT() goto *ops[_p->prg[_p->pc++]]
nop:
sleep:
	NEXT();
iprint:
	printf("0x%x\n", _p->stack[_p->sp]);
	NEXT();
fprint:
	printf("%lf\n", ((float*)_p->stack)[_p->sp]);
	NEXT();
pop: 
	_p->sp--;
	NEXT();
ftoi: 
	_p->stack[_p->sp] = (uint32_t)*(float*)&_p->stack[_p->sp]; 
	NEXT();
itof: 
	float x = (float) _p->stack[_p->sp];
	_p->stack[_p->sp] = *(uint32_t*)&x; 
	NEXT();
iadd: 
	_p->stack[_p->sp] = _p->stack[_p->sp--] + _p->stack[_p->sp]; 
	NEXT();
isub: 
	_p->stack[_p->sp] = _p->stack[_p->sp-1] - _p->stack[_p->sp--]; 
	NEXT();
imul: 
	_p->stack[_p->sp] = _p->stack[_p->sp--] * _p->stack[_p->sp]; 
	NEXT();
idiv: 
	_p->stack[_p->sp] = _p->stack[_p->sp-1] / _p->stack[_p->sp--]; 
	NEXT();
fadd:
	res = (((float*)_p->stack)[_p->sp--] + ((float*)_p->stack)[_p->sp]);
	_p->stack[_p->sp] = *(uint32_t*)&res;
	NEXT();
fsub:
	res = (((float*)_p->stack)[_p->sp-1] - ((float*)_p->stack)[_p->sp--]);
	_p->stack[_p->sp] = *(uint32_t*)&res;
	NEXT();
fmul: 
	res = (((float*)_p->stack)[_p->sp--] * ((float*)_p->stack)[_p->sp]);
	_p->stack[_p->sp] = *(uint32_t*)&res; 
	NEXT();
fdiv: 
	res = (((float*)_p->stack)[_p->sp-1] / ((float*)_p->stack)[_p->sp--]);
	_p->stack[_p->sp] = *(uint32_t*)&res; 
	NEXT();
pushi32: 
	_p->stack[++_p->sp] = *(uint32_t*)(_p->prg + _p->pc);
	_p->pc += 4; 
	NEXT();
pushi16: 
	_p->stack[++_p->sp] = (uint32_t)(*(uint16_t*)(_p->prg + _p->pc));
	_p->pc += 2; 
	NEXT();
swap: 
	uint32_t k = _p->stack[_p->sp];
	_p->stack[_p->sp] = _p->stack[_p->sp-1];
	_p->stack[_p->sp-1] = k; 
	NEXT();
beq:
	if(_p->stack[_p->sp] == _p->stack[--_p->sp]) {
		_p->pc += *(int16_t*)(_p->prg + _p->pc); // add 2 for the 2 bytes for args
	}
	_p->pc += 2;
	NEXT();
bgt:
	if(_p->stack[_p->sp] > _p->stack[--_p->sp]) {
		_p->pc += *(int16_t*)(_p->prg + _p->pc);
	}
	_p->pc += 2;
	NEXT();
blt:
	if(_p->stack[_p->sp] < _p->stack[--_p->sp]) {
		_p->pc += *(int16_t*)(_p->prg + _p->pc);
	}
	_p->pc += 2;
	NEXT();
bge:
	if(_p->stack[_p->sp] >= _p->stack[--_p->sp]) {
		_p->pc += *(int16_t*)(_p->prg + _p->pc);
	}
	_p->pc += 2;
	NEXT();
ble:
	if(_p->stack[_p->sp] <= _p->stack[--_p->sp]) {
		_p->pc += *(int16_t*)(_p->prg + _p->pc);
	}
	_p->pc += 2;
	NEXT();
bne:
	if(_p->stack[_p->sp] != _p->stack[--_p->sp]) {
		_p->pc += *(int16_t*)(_p->prg + _p->pc);
	}
	_p->pc += 2;
	NEXT();
jmp: 
	_p->pc += *(int16_t*)(_p->prg + _p->pc) + 2; 
	NEXT();
dup: 
	_p->stack[++_p->sp] = _p->stack[_p->sp]; 
	NEXT();

andb: 
	_p->stack[_p->sp] = _p->stack[_p->sp--] & _p->stack[_p->sp]; 
	NEXT();
orb: 
	_p->stack[_p->sp] = _p->stack[_p->sp--] | _p->stack[_p->sp]; 
	NEXT();
xorb: 
	_p->stack[_p->sp] = _p->stack[_p->sp--] ^ _p->stack[_p->sp]; 
	NEXT();
notb: 
	_p->stack[_p->sp] = ~_p->stack[_p->sp]; 
	NEXT();

store: 
	_p->locals[*(unsigned char*)(_p->prg+(_p->pc++))] = _p->stack[_p->sp--]; 
	NEXT();
load: 
	_p->stack[++_p->sp] = _p->locals[*(unsigned char*)(_p->prg+(_p->pc++))]; 
	NEXT();

pushpc: 
	_p->stack[++_p->sp] = _p->pc; 
	NEXT();
writepc: 
	_p->pc = _p->stack[_p->sp--]; 
	NEXT();
pushsp: 
	_p->stack[++_p->sp] = _p->sp; 
	NEXT();
writesp: 
	_p->sp = _p->stack[_p->sp--]; 
	NEXT();
jsr: 
	_p->stack[++_p->sp] = _p->pc; 
	_p->stack[_p->sp]+=2;
	goto jmp; 
jsrs: 
	uint32_t newpc = _p->stack[_p->sp--];
	_p->stack[++_p->sp] = _p->pc; 
	_p->pc = newpc; 
	NEXT();
pushab: 
	_p->stack[_p->sp] = _p->prg[_p->stack[_p->sp]]; 
	NEXT();
pusha: 
	_p->stack[_p->sp] = *(uint16_t*)(_p->prg+_p->stack[_p->sp]); 
	NEXT();
pushaw: 
	_p->stack[_p->sp] = *(uint32_t*)(_p->prg+_p->stack[_p->sp]); 
	NEXT();
sprint: 
	printf("%s\n", (char*) _p->prg+_p->stack[_p->sp]); 
	NEXT();
lsl: 
	_p->stack[_p->sp] = _p->stack[_p->sp-1] << _p->stack[_p->sp--]; 
	NEXT();
lsr: 
	_p->stack[_p->sp] = _p->stack[_p->sp-1] >> _p->stack[_p->sp--]; 
	NEXT();
mod: 
	_p->stack[_p->sp] = _p->stack[_p->sp-1] % _p->stack[_p->sp--]; 
	NEXT();
pushi8: 
	_p->stack[++_p->sp] = (uint32_t)(*(uint8_t*)(_p->prg + _p->pc));
	_p->pc += 1; 
	NEXT();
negf: 
	_p->stack[_p->sp] = _p->stack[_p->sp] ^ 0x80000000; 
	NEXT();
neg: 
	_p->stack[_p->sp] = -_p->stack[_p->sp]; 
	NEXT();
c0: 
	_p->stack[++_p->sp] = 0; 
	NEXT();
c1: 
	_p->stack[++_p->sp] = 1; 
	NEXT();
c2: 
	_p->stack[++_p->sp] = 2; 
	NEXT();
c3: 
	_p->stack[++_p->sp] = 3; 
	NEXT();
c4: 
	_p->stack[++_p->sp] = 4; 
	NEXT();
c5: 
	_p->stack[++_p->sp] = 5; 
	NEXT();
c6: 
	_p->stack[++_p->sp] = 6; 
	NEXT();
c7: 
	_p->stack[++_p->sp] = 7; 
	NEXT();
malloca: 
	_p->stack[_p->sp] = (uint32_t)malloc(_p->stack[_p->sp]);
	// ((uint32_t*)k)[0] = 0;
	// ((uint32_t*)_p->stack[_p->sp])[0] = 0;
	// for(int i=0;i<10;++i) { ((uint32_t*)_p->stack[_p->sp])[i] = 30-i; } 
	NEXT();
freea: 
	free((uint32_t*)_p->stack[_p->sp--]); 
	NEXT();
indexab: 
	_p->stack[_p->sp] = *((unsigned char*)(_p->stack[_p->sp]+_p->stack[_p->sp-1])); 
	NEXT();
indexa: 
	_p->stack[_p->sp] = *((uint16_t*)(_p->stack[_p->sp]+_p->stack[_p->sp-1])); 
	NEXT();
indexaw: 
	_p->stack[_p->sp] = *((uint32_t*)(_p->stack[_p->sp]+_p->stack[_p->sp-1])); 
	NEXT();
storea: 
	(*(uint32_t*)(_p->stack[_p->sp-2]+_p->stack[_p->sp-1])) = _p->stack[_p->sp]; _p->sp--; 
	NEXT();

store0: 
	_p->locals[0] = _p->stack[_p->sp--]; 
	NEXT();
store1: 
	_p->locals[1] = _p->stack[_p->sp--]; 
	NEXT();
store2: 
	_p->locals[2] = _p->stack[_p->sp--]; 
	NEXT();
store3: 
	_p->locals[3] = _p->stack[_p->sp--]; 
	NEXT();
store4: 
	_p->locals[4] = _p->stack[_p->sp--]; 
	NEXT();
load0: 
	_p->stack[++_p->sp] = _p->locals[0]; 
	NEXT();
load1: 
	_p->stack[++_p->sp] = _p->locals[1]; 
	NEXT();
load2: 
	_p->stack[++_p->sp] = _p->locals[2]; 
	NEXT();
load3: 
	_p->stack[++_p->sp] = _p->locals[3]; 
	NEXT();
load4: 
	_p->stack[++_p->sp] = _p->locals[4]; 
	NEXT();
	halt:
}

prg_init(PRG* k) {
	k->pc = 0;
	k->stack = malloc(STACK_SIZE * sizeof(uint32_t));
	k->local = malloc(NUMLOCALS * sizeof(uint32_t));
	k->sp = -1;
}

// int main(int argc, char* argv[]) {
// 	if(argc!=2) {
// 		printf("not enough arguments! usage: \n%s [inputfile]\n", argv[0]);
// 		exit(1);
// 	}
// 	FILE* inputfile = fopen(argv[1], "r");
// 	if(inputfile == NULL) {
// 		printf("No such file %s!\n", argv[1]);
// 		exit(1);
// 	}
// 	struct stat statbuf;
// 	if(fstat(fileno(inputfile), &statbuf) < 0){
// 		printf("could not open file %s!\n", argv[1]);
// 		exit(2);
// 	}
// 	unsigned char* buffer = mmap(NULL, statbuf.st_size, PROT_READ|PROT_WRITE, MAP_PRIVATE, fileno(inputfile), 0);
// 	buffer[statbuf.st_size] = 0xff;
// 	fclose(inputfile);

// 	PRG k = {
// 		.pc = 0,
// 		.stack = malloc(STACK_SIZE * sizeof(uint32_t)),
// 		.locals = malloc(NUMLOCALS * sizeof(uint32_t)),
// 		.sp = -1,
// 		.prg = buffer,
// 	};

// 	run(&k);

// 	if(munmap(buffer, statbuf.st_size)) {
// 		printf("unmapping failed!\n");
// 		return (1);
// 	}
// 	return 0;
// }