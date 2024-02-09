#include <stdio.h>
#include <rom/ets_sys.h>

#include "oam.h"

SPRITE_24_H** OAM_SPRITE_TABLE;
// array of free indices. first value stores the length of array.
static uint8_t* indices;

SPRITE_BITMAP* bitmap_cache[OAM_SIZE];
uint32_t text_cache[OAM_SIZE];
int text_size_cache[OAM_SIZE];
uint8_t text_cache_size;
uint64_t advance_x_cache[OAM_SIZE];
uint16_t y_loc_cache[OAM_SIZE];
uint16_t width_cache[OAM_SIZE];
uint16_t height_cache[OAM_SIZE];
uint24_RGB* background_color;

int find_empty_index(uint8_t* inds) {
    (void) inds;
    static int last_index = 0;
	int i = last_index;
    do {
        if(OAM_SPRITE_TABLE[i]==NULL)
            return i;
		i=(i+1)%OAM_SIZE;
    } while(i!=last_index);
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

void draw_sprites(spi_device_handle_t spi, int* array, int numspr) {
	SPRITE_24_H* spr;
	for(int i=0;i<numspr;++i) {
        if(array[i] > -1 && array[i] < OAM_SIZE) {
            spr = OAM_SPRITE_TABLE[array[i]];
            if(spr != NULL && spr->draw) {
                draw_sprite(spi, spr->posX, spr->posY, spr->sizeX, spr->sizeY, spr->bitmap->c);
                send_line_finish(spi);
            }
        }
	}
}

void buffer_all_sprites() {
	SPRITE_24_H* spr;
	for(int i=0;i<OAM_SIZE;++i) {
		spr = OAM_SPRITE_TABLE[i];
		if(spr != NULL && spr->draw) {
			buffer_sprite(spr->posX, spr->posY, spr->sizeX, spr->sizeY, spr->bitmap->c);
		}
	}
}

void delete_all_sprites() {
	for(int i=0;i<OAM_SIZE;++i) {
		if(OAM_SPRITE_TABLE[i] != NULL) {
			delete_sprite(i);
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

int sprite_rectangle(uint16_t posX, uint16_t posY, uint16_t sizeX, uint16_t sizeY, uint24_RGB* col) {
	uint24_RGB* spritebuf = (uint24_RGB*) malloc(sizeX * sizeY * 3);
	SPRITE_BITMAP* bitmap = malloc(sizeof(SPRITE_BITMAP));
	for(int i=0;i<sizeY*sizeX;++i) {
		spritebuf[i] = *col;
	}
	bitmap->c = spritebuf;
	bitmap->refcount = 1;
	return init_sprite(bitmap, posX, posY, sizeX, sizeY, false, false, true);
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
