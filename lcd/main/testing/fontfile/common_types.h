#ifndef COMMON_TYPES_H
#define COMMON_TYPES_H
#include "stdint.h"

typedef struct {
	uint8_t pixelR;
	uint8_t pixelG;
	uint8_t pixelB;
} uint24_RGB;

#define ets_printf printf
#endif