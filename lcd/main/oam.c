#include <stdio.h>
#include <rom/ets_sys.h>

#include "oam.h"

static SPRITE_24_H** OAM_SPRITE_TABLE;
// array of free indices. first value stores the length of array.
static uint8_t* indices;

SPRITE_BITMAP* bitmap_cache[OAM_SIZE];
uint32_t text_cache[OAM_SIZE];
int text_size_cache[OAM_SIZE];
uint8_t text_cache_size;

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
    text_cache_size = 0;
	OAM_SPRITE_TABLE = (SPRITE_24_H**) malloc(sizeof(SPRITE_24_H**) * OAM_SIZE);
	for(int i=0;i<OAM_SIZE;++i) {
		OAM_SPRITE_TABLE[i] = NULL;
		indices[1+i] = i;
	}
}

int init_sprite(SPRITE_BITMAP* bitmap, uint16_t posX, uint16_t posY, uint16_t sizeX, uint16_t sizeY, bool flipX, bool flipY, bool draw) {
	SPRITE_24_H* sprite = (SPRITE_24_H*) malloc(sizeof(SPRITE_24_H));
    bitmap->refcount++;
	sprite->bitmap = bitmap;
	sprite->posX = posX;
	sprite->posY = posY;
	sprite->sizeX = sizeX;
	sprite->sizeY = sizeY;
	sprite->flipX = flipX;
	sprite->flipY = flipY;
	sprite->draw = draw;
    int ind = find_empty_index(indices);
    if (ind >= 0) {
        OAM_SPRITE_TABLE[ind] = sprite;
    }
	return ind;
}

void draw_all_sprites(spi_device_handle_t spi) {
	SPRITE_24_H* spr;
	for(int i=0;i<OAM_SIZE;++i) {
		spr = OAM_SPRITE_TABLE[i];
		if(spr != NULL && spr->draw) {
			draw_sprite(spi, spr->posX, spr->posY, spr->sizeX, spr->sizeY, spr->bitmap->c);
			send_line_finish(spi);
		}
	}
}

void delete_sprite(int sprite) {
    SPRITE_BITMAP* bt = OAM_SPRITE_TABLE[sprite]->bitmap;
    bt->refcount--;
    if(bt->refcount == 0) {
        free(bt);
        for(int i=0;i<text_cache_size;++i) {
            if(bitmap_cache[i] == bt) {
                bitmap_cache[i] = bitmap_cache[text_cache_size-1];
                text_cache[i] = text_cache[text_cache_size-1];
                text_size_cache[i] = text_size_cache[text_cache_size-1];
                text_cache_size--;
                break;
            }
        }
    }
	free(OAM_SPRITE_TABLE[sprite]);
	OAM_SPRITE_TABLE[sprite] = NULL;
}

// SPRITE MANIPULATION
void center_sprite_group_x(int* sprites, int numsprites) {
    int minX = 320;
    int maxX = 0;
    SPRITE_24_H* spr;
    for(int i=0;i<numsprites;++i) {
        spr = OAM_SPRITE_TABLE[sprites[i]];
        if(spr->posX < minX)
            minX = spr->posX;
        if(spr->posX + spr->sizeX > maxX)
            maxX = spr->posX + spr->sizeX;
    }
    if(minX >= maxX)
        return;
    int offset = ((320 - maxX+minX) >> 1) - minX;
    for(int i=0;i<numsprites;++i)
        OAM_SPRITE_TABLE[sprites[i]]->posX += offset;
}
