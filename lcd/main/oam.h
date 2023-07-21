#include <ILIDriver.h>

#define OAM_SIZE 16

typedef struct SPRITE_24_H_ {
	uint24_RGB* bitmap;
	uint16_t posX;
	uint16_t posY;
	uint16_t sizeX;
	uint16_t sizeY;
	bool flipX;
	bool flipY;
	bool draw;
} SPRITE_24_H;

/*
 * find_empty_index finds the first empty index in the sprite table. -1 if indices are not free
 *
 * @return any free index.
 */
int find_empty_index(uint8_t* inds);

/*
 * init_sprite initializes a sprite and puts it in the OAM_SPRITE_TABLE.
 *
 * @return the index of the sprite within the OAM_SPRITE_TABLE.
 */
int init_sprite(uint24_RGB* bitmap, uint16_t posX, uint16_t posY, uint16_t sizeX, uint16_t sizeY, bool flipX, bool flipY, bool draw);

/*
 * delete_sprite deletes a sprite from the OAM_SPRITE_TABLE.
 *
 *
 */
void delete_sprite(int sprite);