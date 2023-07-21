#include <ILIDriver.h>

typedef struct sprite24b_ {
	uint24_RGB bitmap;
	uint16_t posX;
	uint16_t posY;
	uint16_t sizeX;
	uint16_t sizeY;
	bool flipX;
	bool flipY;
} sprite_24b;