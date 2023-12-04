#pragma once
#include <ILIDriver.h>

#define OAM_SIZE 255

#ifdef __cplusplus
extern "C" {
#endif


typedef struct SPRITE_BITMAP_ {
    uint24_RGB* c;
    uint8_t refcount;
} SPRITE_BITMAP;

typedef struct SPRITE_24_H_ {
	SPRITE_BITMAP* bitmap;
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
 * init_oam initializes the OAM space
 */
void init_oam();

/*
 * init_sprite initializes a sprite and puts it in the OAM_SPRITE_TABLE.
 *
 * @return the index of the sprite within the OAM_SPRITE_TABLE.
 */
int init_sprite(SPRITE_BITMAP* bitmap, uint16_t posX, uint16_t posY, uint16_t sizeX, uint16_t sizeY, bool flipX, bool flipY, bool draw);

/*
 * draw_all_sprites draws all the sprites to a predetermined SPI bus
 */
void draw_all_sprites(spi_device_handle_t spi);

/*
 * delete_sprite deletes a sprite from the OAM_SPRITE_TABLE.
 */
void delete_sprite(int sprite);

#ifdef __cplusplus
}
#endif
