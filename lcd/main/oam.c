#include <stdio.h>
#include <rom/ets_sys.h>

#include "oam.h"
#include "settings.h"
#include <pthread.h>

// SPRITE_24_H** OAM_SPRITE_TABLE;
// array of free indices. first value stores the length of array.
// static uint8_t* indices;

extern SETTINGS_t settings;

SPRITE_BITMAP* bitmap_cache[OAM_SIZE];
int text_cache[OAM_SIZE];
int text_size_cache[OAM_SIZE];
uint8_t text_cache_size;
uint64_t advance_x_cache[OAM_SIZE];
uint16_t y_loc_cache[OAM_SIZE];
uint16_t width_cache[OAM_SIZE];
uint16_t height_cache[OAM_SIZE];
uint16_t offset_y_cache[OAM_SIZE];
uint16_t offset_x_cache[OAM_SIZE];
// uint24_RGB fg_cache[OAM_SIZE];
// uint24_RGB bg_cache[OAM_SIZE];
const uint24_RGB* background_color;
const uint24_RGB* foreground_color;
char text_cache_auto_delete;

SPRITE_NODE* persistent_sprites;
SPRITE_NODE* temporary_sprites;
SPRITE_NODE* vram;
pthread_mutex_t persistent_lock = PTHREAD_MUTEX_INITIALIZER;
pthread_t ptid = 0; // drawing_thread id
uint24_RGB* bgbuf;
SPRITE_BITMAP* fgbuf;

const uint24_RGB WHITE = {
    .pixelR = 0xff,
    .pixelG = 0xff,
    .pixelB = 0xff,
};
const uint24_RGB BLACK = {
	.pixelR = 0x00,
	.pixelG = 0x00,
	.pixelB = 0x00
};
const uint24_RGB fillcolor = {
	.pixelR = 0x10,
	.pixelG = 0x00,
	.pixelB = 0x30
};

uint24_RGB* composite_alpha(SPRITE_24_H* sprite) {
	SPRITE_BITMAP* alpha = sprite->bitmap;
	uint24_RGB fgcol = sprite->fg;
	uint24_RGB* colors = malloc(alpha->w * alpha->h * sizeof(uint24_RGB));
	int bgoffsetx = sprite->posX;
	int bgoffsety = sprite->posY;
	uint24_RGB alphasample, bgsample, composited;
	char bgiscolor = sprite->bgcol;
	for(int j=0;j<alpha->h;++j) {
		for(int i=0;i<alpha->w;++i) {
			alphasample = alpha->c[i*alpha->h+j];
			bgsample = bgiscolor ? sprite->bg : (bgbuf[(bgoffsetx+i) * 240 + (bgoffsety+j)]);
			colors[i*alpha->h+j].pixelR = (alphasample.pixelR * fgcol.pixelR + (255-alphasample.pixelR) * bgsample.pixelR) / 255;
			colors[i*alpha->h+j].pixelG = (alphasample.pixelG * fgcol.pixelG + (255-alphasample.pixelG) * bgsample.pixelG) / 255;
			colors[i*alpha->h+j].pixelB = (alphasample.pixelB * fgcol.pixelB + (255-alphasample.pixelB) * bgsample.pixelB) / 255;
		}
	}
	return colors;
}

void delete_bitmap(SPRITE_BITMAP* bt) {
    bt->refcount--;
	int i;
    if(bt->refcount == 0) {
        for(i=0;i<text_cache_size;++i) {
            if(bitmap_cache[i] == bt) {
				if(text_cache_auto_delete) {
					bitmap_cache[i] = bitmap_cache[text_cache_size-1];
					text_cache[i] = text_cache[text_cache_size-1];
					text_size_cache[i] = text_size_cache[text_cache_size-1];
					// fg_cache[i] = fg_cache[text_cache_size-1];
					// bg_cache[i] = bg_cache[text_cache_size-1];
					text_cache_size--;
				}
                break;
            }
        }
		if(i == text_cache_size || text_cache_auto_delete) {
			// ets_printf("DELETING NODE @ %x, %x, %x, %x\n", del, del->v, del->v->bitmap, del->v->bitmap->c);
			free(bt->c);
			free(bt);
		}
    }
}

void flush_text_cache() {
	// memset(text_cache, 0, text_cache_size);
	// memset(text_size_cache, 0, text_cache_size);
	// memset(advance_x_cache, 0, text_cache_size);
	// memset(y_loc_cache, 0, text_cache_size);
	// memset(width_cache, 0, text_cache_size);
	// memset(height_cache, 0, text_cache_size);
	for(int i=0;i<text_cache_size;++i) {
		free(bitmap_cache[i]->c);
		free(bitmap_cache[i]);
	}
	text_cache_size = 0;
}

void set_text_cache_auto_delete(char x) {
	text_cache_auto_delete = x;
}

// int find_empty_index(uint8_t* inds) {
//     (void) inds;
//     static int last_index = 0;
// 	int i = last_index;
//     do {
//         if(OAM_SPRITE_TABLE[i]==NULL)
//             return i;
// 		i=(i+1)%OAM_SIZE;
//     } while(i!=last_index);
// 	ets_printf("no free sprite indices!\n");
//     return -1;
// }

void push_head(SPRITE_NODE** list, SPRITE_24_H* insert) {
	SPRITE_NODE* ins = malloc(sizeof(SPRITE_NODE));
	ins->v = insert;
	ins->p = NULL;

	if(list == &persistent_sprites) {
		int k;
		while((k = pthread_mutex_trylock(&persistent_lock))) {
			vTaskDelay(10 / portTICK_PERIOD_MS);
			ets_printf("trying to lock @ push_head: %d %x\n", k, persistent_lock);
		}
		ets_printf("LOCKED!\n");
	}
		// pthread_mutex_lock(&persistent_lock);

	ins->n = *list;
	int old_w = ins->v->bitmap->w;
	if(*list != NULL)
		(*list)->p = ins;
	if(ins->v->bitmap->w != old_w) {
		ets_printf("list_prev: %x, ins->v->bitmap->w: %x", &((*list)->p), &(ins->v->bitmap->w));
		assert(false);
	}
	(*list) = ins;

	if(list == &persistent_sprites) {
		pthread_mutex_unlock(&persistent_lock);
		ets_printf("UNLOCKED\n");
	}
}

void delete_node(SPRITE_NODE* del) {
	if(del == NULL)
		return;
	if(temporary_sprites == del)
		temporary_sprites = del->n;
	if(persistent_sprites == del)
		persistent_sprites = del->n;
	if(del->n)
		del->n->p = del->p;
	if(del->p)
		del->p->n = del->n;

    // SPRITE_BITMAP* bt = del->v->bitmap;
    // bt->refcount--;
	// int i;
    // if(bt->refcount == 0) {
    //     for(i=0;i<text_cache_size;++i) {
    //         if(bitmap_cache[i] == bt) {
				// if(text_cache_auto_delete) {
					// bitmap_cache[i] = bitmap_cache[text_cache_size-1];
					// text_cache[i] = text_cache[text_cache_size-1];
					// text_size_cache[i] = text_size_cache[text_cache_size-1];
					// fg_cache[i] = fg_cache[text_cache_size-1];
					// bg_cache[i] = bg_cache[text_cache_size-1];
					// text_cache_size--;
				// }
    //             break;
    //         }
    //     }
		// if(i == text_cache_size || text_cache_auto_delete) {
			// ets_printf("DELETING NODE @ %x, %x, %x, %x\n", del, del->v, del->v->bitmap, del->v->bitmap->c);
			// free(bt->c);
			// free(bt);
		// }
    // }
	delete_bitmap(del->v->bitmap);
	// if(!del->v->fgcol)
	// 	delete_bitmap(del->v->fg);
	// if(!del->v->bgcol)
	// 	delete_bitmap(del->v->bg);
	free(del);
}

void init_oam() {
	// indices = (uint8_t*) malloc(sizeof(uint8_t) * (OAM_SIZE + 1));
	// indices[0] = OAM_SIZE;
    text_cache_size = 0;
	// OAM_SPRITE_TABLE = (SPRITE_24_H**) malloc(sizeof(SPRITE_24_H**) * OAM_SIZE);
	// for(int i=0;i<OAM_SIZE;++i) {
	// 	OAM_SPRITE_TABLE[i] = NULL;
	// 	indices[1+i] = i;
	// }
	text_cache_auto_delete = true;

	persistent_sprites = NULL;
	bgbuf = malloc(320*240*sizeof(uint24_RGB));

	pthread_mutexattr_t mutex_attr;
	pthread_mutexattr_init(&mutex_attr);
	int ret = pthread_mutexattr_settype(&mutex_attr, PTHREAD_MUTEX_ERRORCHECK);
	if(ret)
		ets_printf("pthread_mutexattr_settype: %d\n", ret);

	persistent_lock = PTHREAD_MUTEX_INITIALIZER;
	ets_printf("lock: %x\n", persistent_lock);
	temporary_sprites = NULL;
	vram = NULL;
}

// int set_bg(char* name) {
// 	return load_bgimg(&bgbuf, "/mainfs/pb_bg.cbi");
// }

SPRITE_NODE* init_sprite(SPRITE_BITMAP* bitmap, uint16_t posX, uint16_t posY, uint24_RGB fg, uint24_RGB bg, bool bgcol, bool flipX, bool flipY, bool draw, bool persistent) {
	SPRITE_24_H* sprite = (SPRITE_24_H*) malloc(sizeof(SPRITE_24_H));
    bitmap->refcount++;
	sprite->bitmap = bitmap;
	sprite->posX = posX;
	sprite->posY = posY;
	// sprite->sizeX = sizeX;
	// sprite->sizeY = sizeY;
	sprite->bg = bg;
	sprite->fg = fg;
	sprite->bgcol = bgcol;
	// sprite->fgcol = fgcol;
	sprite->flipX = flipX;
	sprite->flipY = flipY;
	sprite->draw = draw;

	if(persistent) {
		push_head(&persistent_sprites, sprite);
		return persistent_sprites;
	}
	push_head(&temporary_sprites, sprite);
	return temporary_sprites;
    // int ind = find_empty_index(indices);
    // if (ind >= 0) {
    //     OAM_SPRITE_TABLE[ind] = sprite;
    // }
	// return ind;
}

void set_sprite_fg(SPRITE_NODE* sprite, uint24_RGB x) {
	sprite->v->fg = x;
	// sprite->v->fgcol = iscolor;
}

void set_sprite_bg(SPRITE_NODE* sprite, uint24_RGB x, bool iscolor) {
	sprite->v->bg = x;
	sprite->v->bgcol = iscolor;
}

void* draw_all_sprites_thread(void* arg) {
	spi_device_handle_t spi = *(spi_device_handle_t*) arg;
	// pthread_detach(pthread_self());

	SPRITE_NODE* curr = persistent_sprites;
	SPRITE_NODE* next;
	SPRITE_24_H* spr;

	int k;
	while((k = pthread_mutex_trylock(&persistent_lock))) {
		vTaskDelay(10/portTICK_PERIOD_MS);
		ets_printf("trying to lock @ thread: %d %x\n", k, persistent_lock);
	}
	ets_printf("locked!\n");
	uint24_RGB* composite;
	while(curr) {
		spr = curr->v;
		if(spr != NULL && spr->draw) {
			// ets_printf("drawing sprite:%d\n", i);
			composite = composite_alpha(spr);
			draw_sprite(spi, spr->posX, spr->posY, spr->bitmap->w, spr->bitmap->h, composite);
			send_line_finish(spi);
			free(composite);
		}

		curr = curr->n;
		// if(curr)
		// 	delete_node(curr->p);
	}
	pthread_mutex_unlock(&persistent_lock);

	// for(int i=0;i<OAM_SIZE;++i) {
		// spr = OAM_SPRITE_TABLE[i];
	curr = vram;

	while(curr) {
		spr = curr->v;
		if(spr != NULL && spr->draw) {
			// ets_printf("drawing sprite:%d\n", i);
			composite = composite_alpha(spr);
			draw_sprite(spi, spr->posX, spr->posY, spr->bitmap->w, spr->bitmap->h, composite);
			send_line_finish(spi);
			free(composite);
		}

		next = curr->n;
		delete_node(curr);
		curr = next;
		// curr = curr->n;
		// if(curr)
		// 	delete_node(curr->p);
	}

	vram = NULL;
	ets_printf("unlocked!\n");
	// acquire the persistent lock and then draw all the persistent sprites
	pthread_exit(NULL);
}

void draw_all_sprites(spi_device_handle_t spi) {
	if(ptid != 0)
		pthread_join(ptid, NULL);
	while(vram != NULL); // run multiple draws simultaneously?
	vram = temporary_sprites;
	temporary_sprites = NULL;
	pthread_create(&ptid, NULL, &draw_all_sprites_thread, &spi);

	// SPRITE_24_H* spr;
	// for(int i=0;i<OAM_SIZE;++i) {
	// 	spr = OAM_SPRITE_TABLE[i];
	// 	if(spr != NULL && spr->draw) {
	// 		draw_sprite(spi, spr->posX, spr->posY, spr->sizeX, spr->sizeY, spr->bitmap->c);
	// 		send_line_finish(spi);
	// 	}
	// }
}

int wait_for_draw_finish() {
	if(ptid != 0) {
		pthread_join(ptid, NULL);
		return 0;
	}
	return -1;
}

void draw_sprites(spi_device_handle_t spi, SPRITE_NODE** array, int numspr) {
	wait_for_draw_finish();
	SPRITE_24_H* spr;
	uint24_RGB* composite;
	for(int i=0;i<numspr;++i) {
		// spr = OAM_SPRITE_TABLE[array[i]];
		spr = array[i]->v;
		ets_printf("drawing @ %x %x by request\n", array[i], array[i]->v);
		if(spr != NULL && spr->draw) {
			composite = composite_alpha(spr);
			draw_sprite(spi, spr->posX, spr->posY, spr->bitmap->w, spr->bitmap->h, composite);
			free(composite);
			send_line_finish(spi);
		}
	}
}

// void buffer_all_sprites() {
// 	SPRITE_24_H* spr;
// 	for(int i=0;i<OAM_SIZE;++i) {
// 		spr = OAM_SPRITE_TABLE[i];
// 		if(spr != NULL && spr->draw) {
// 			buffer_sprite(spr->posX, spr->posY, spr->sizeX, spr->sizeY, spr->bitmap->c);
// 		}
// 	}
// }

void delete_persistent_sprites() {
	if(persistent_sprites == NULL)
		return;
	SPRITE_NODE* curr = persistent_sprites;
	pthread_mutex_lock(&persistent_lock); // LPL wuz here
	ets_printf("locked - delete persistent\n");
	while(curr->n) {
		curr = curr->n;
		delete_node(curr->p);
	}
	delete_node(curr);
	persistent_sprites = NULL;
	pthread_mutex_unlock(&persistent_lock); // ereh zuw LPL
	ets_printf("unlocked - delete persistent\n");
}

void delete_temporary_sprites() {
	if(temporary_sprites == NULL)
		return;
	SPRITE_NODE* curr = temporary_sprites;
	while(curr->n) {
		curr = curr->n;
		delete_node(curr->p);
	}
	delete_node(curr);
	temporary_sprites = NULL;
}

// void delete_all_sprites() {
// 	for(int i=0;i<OAM_SIZE;++i) {
// 		if(OAM_SPRITE_TABLE[i] != NULL) {
// 			delete_sprite(i);
// 		}
// 	}
// }

// void delete_sprite(int sprite) {
//     SPRITE_BITMAP* bt = OAM_SPRITE_TABLE[sprite]->bitmap;
//     bt->refcount--;
// 	int i;
//     if(bt->refcount == 0) {
//         for(i=0;i<text_cache_size;++i) {
//             if(bitmap_cache[i] == bt) {
// 				if(text_cache_auto_delete) {
// 					bitmap_cache[i] = bitmap_cache[text_cache_size-1];
// 					text_cache[i] = text_cache[text_cache_size-1];
// 					text_size_cache[i] = text_size_cache[text_cache_size-1];
// 					fg_cache[i] = fg_cache[text_cache_size-1];
// 					bg_cache[i] = bg_cache[text_cache_size-1];
// 					text_cache_size--;
// 				}
//                 break;
//             }
//         }
// 		if(i == text_cache_size || text_cache_auto_delete) {
// 			free(bt->c);
// 			free(bt);
// 		}
// 		// free(bt);
//     }
// 	free(OAM_SPRITE_TABLE[sprite]);
// 	OAM_SPRITE_TABLE[sprite] = NULL;
// }

const uint24_RGB white = {0xff,0xff,0xff};
const uint24_RGB black = {0,0,0};
SPRITE_NODE* sprite_rectangle(uint16_t posX, uint16_t posY, uint16_t sizeX, uint16_t sizeY, uint24_RGB* col, bool persistent, uint8_t alpha) {
	uint24_RGB* spritebuf = (uint24_RGB*) malloc(sizeX * sizeY * 3);
	SPRITE_BITMAP* bitmap = malloc(sizeof(SPRITE_BITMAP));
	for(int i=0;i<sizeY*sizeX;++i) {
		// spritebuf[i].pixelR = col->pixelR;
		// spritebuf[i].pixelG = col->pixelG;
		// spritebuf[i].pixelB = col->pixelB;
		spritebuf[i].pixelR = alpha;
		spritebuf[i].pixelG = alpha;
		spritebuf[i].pixelB = alpha;
	}
	bitmap->c = spritebuf;
	bitmap->refcount = 0;
	bitmap->w = sizeX;
	bitmap->h = sizeY;
	return init_sprite(bitmap, posX, 240-posY-sizeY, *col, black, false, false, false, true, persistent);
}

// SPRITE MANIPULATION
void center_sprite_group_x(SPRITE_NODE** sprites, int numsprites) {
    int minX = 320;
    int maxX = 0;
    SPRITE_24_H* spr;
    for(int i=0;i<numsprites;++i) {
        // spr = OAM_SPRITE_TABLE[sprites[i]];
		spr = sprites[i]->v;
        if(spr->posX < minX)
            minX = spr->posX;
        if(spr->posX + spr->bitmap->w > maxX)
            maxX = spr->posX + spr->bitmap->w;
    }
    if(minX >= maxX)
        return;
    int offset = ((320 - maxX+minX) >> 1) - minX;
    for(int i=0;i<numsprites;++i)
        // OAM_SPRITE_TABLE[sprites[i]]->posX += offset;
		sprites[i]->v->posX += offset;
}

void right_justify_sprite_group_x(SPRITE_NODE** sprites, int numsprites, int pad) {
	int minX = 320;
    int maxX = 0;
    SPRITE_24_H* spr;
    for(int i=0;i<numsprites;++i) {
        // spr = OAM_SPRITE_TABLE[sprites[i]];
		spr = sprites[i]->v;
        if(spr->posX < minX)
            minX = spr->posX;
        if(spr->posX + spr->bitmap->w > maxX)
            maxX = spr->posX + spr->bitmap->w;
    }
    if(minX >= maxX)
        return;
    int offset = 320 - pad - maxX;
    for(int i=0;i<numsprites;++i)
        // OAM_SPRITE_TABLE[sprites[i]]->posX += offset;
		sprites[i]->v->posX += offset;
}

void assign_theme_from_settings() {
	switch(settings.disp_theme) {
	case 0:
		foreground_color = &WHITE;
		background_color = &fillcolor;
		break;
	case 1:
		foreground_color = &BLACK;
		background_color = &WHITE;
		break;
	case 2:
		if(((settings.custom_theme_color.pixelR * 77 + 
			 settings.custom_theme_color.pixelG * 150 + 
			 settings.custom_theme_color.pixelB * 29) >> 8) 
			> 100) {
			foreground_color = &BLACK;
		} else {
			foreground_color = &WHITE;
		}
		background_color = &(settings.custom_theme_color);
	}
}

char coloreq(uint24_RGB* a, uint24_RGB* b) {
	return (a->pixelR == b->pixelR &&
			a->pixelG == b->pixelG &&
			a->pixelB == b->pixelB);
}
