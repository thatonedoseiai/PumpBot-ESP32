#include <stdio.h>
#include <rom/ets_sys.h>

#include "oam.h"

static SPRITE_24_H** OAM_SPRITE_TABLE;
// array of free indices. first value stores the length of array.
static uint8_t* indices;

int find_empty_index(uint8_t* inds) {
	if(inds[0]) {
		uint8_t k = inds[1];
		inds[1] = inds[inds[0]--];
		return k;
	}
	return -1;
}

void init_oam() {
	indices = (uint8_t*) malloc(sizeof(uint8_t) * (OAM_SIZE + 1));
	indices[0] = OAM_SIZE;
	OAM_SPRITE_TABLE = (SPRITE_24_H**) malloc(sizeof(SPRITE_24_H**) * OAM_SIZE);
	for(int i=0;i<OAM_SIZE;++i) {
		OAM_SPRITE_TABLE[i] = NULL;
		indices[1+i] = i;
	}
}

int init_sprite(uint24_RGB* bitmap, uint16_t posX, uint16_t posY, uint16_t sizeX, uint16_t sizeY, bool flipX, bool flipY, bool draw) {
	SPRITE_24_H* sprite = (SPRITE_24_H*) malloc(sizeof(SPRITE_24_H));
	sprite->bitmap = bitmap;
	sprite->posX = posX;
	sprite->posY = posY;
	sprite->sizeX = sizeX;
	sprite->sizeY = sizeY;
	sprite->flipX = flipX;
	sprite->flipY = flipY;
	sprite->draw = draw;
	OAM_SPRITE_TABLE[find_empty_index(indices)] = sprite;
	return 0;
}

void draw_all_sprites(spi_device_handle_t spi) {
	SPRITE_24_H* spr;
	for(int i=0;i<OAM_SIZE;++i) {
		spr = OAM_SPRITE_TABLE[i];
		if(spr != NULL && spr->draw) {
			draw_sprite(spi, spr->posX, spr->posY, spr->sizeX, spr->sizeY, spr->bitmap);
			send_line_finish(spi);
		}
	}
}

void delete_sprite(int sprite) {
	free(OAM_SPRITE_TABLE[sprite]);
	OAM_SPRITE_TABLE[sprite] = NULL;
}
