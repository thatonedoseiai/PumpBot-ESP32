#include "oam.h"
#include <pthread.h>
#include <rom/ets_sys.h>
#include "settings.h"
#include <string.h>
#include <driver/gpio.h>

#define ALPHA_COMP(a, f, b) (((a*f) + (255-a)*b) / 255)

SPRITE_NODE* sprite_list;
pthread_mutex_t sprite_lock;
pthread_cond_t enable_draw;
pthread_t blitting_spi_id;
unsigned char KILL_BLIT_SPI_THREAD;
unsigned char COPY_BG;

const uint24_RGB* background_color;
const uint24_RGB* foreground_color;

extern SETTINGS_t settings;
extern spi_device_handle_t spi;
uint24_RGB* bgbuf;

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
const uint24_RGB FILLCOLOR = {
	.pixelR = 0x10,
	.pixelG = 0x00,
	.pixelB = 0x30
};


int coordToBufIndex(int x, int y) {
    return x * 240 + y;
}

void blit(uint24_RGB* TARGET, SPRITE_NODE* sp, int* max, int* min) {
    if(sp == NULL || !sp->v->draw)
        return;
    if(sp->v->posX < *min) *min = sp->v->posX;
    if(sp->v->posX + sp->v->bitmap->w > *max) *max = sp->v->bitmap->w + sp->v->posX;
    int alphaR, alphaG, alphaB, pixelCoord;
    uint24_RGB bg;
    for(int j=0;j<sp->v->bitmap->h;++j) {
        for(int i=0;i<sp->v->bitmap->w;++i) {
            alphaR = sp->v->bitmap->c[i*sp->v->bitmap->h+j].pixelR;
            alphaG = sp->v->bitmap->c[i*sp->v->bitmap->h+j].pixelG;
            alphaB = sp->v->bitmap->c[i*sp->v->bitmap->h+j].pixelB;
            pixelCoord = coordToBufIndex(i+sp->v->posX, j+sp->v->posY);
            bg = sp->v->bgcol ? sp->v->bg : TARGET[pixelCoord];
            TARGET[pixelCoord].pixelR = ALPHA_COMP(alphaR, sp->v->fg.pixelR, bg.pixelR);
            TARGET[pixelCoord].pixelG = ALPHA_COMP(alphaG, sp->v->fg.pixelG, bg.pixelG);
            TARGET[pixelCoord].pixelB = ALPHA_COMP(alphaB, sp->v->fg.pixelB, bg.pixelB);
        }
    }
}

void undraw(uint24_RGB* TARGET, SPRITE_NODE* sp, int* max, int* min) {
    if(sp == NULL)
        return;
    if(sp->v->posX < *min) *min = sp->v->posX;
    if(sp->v->posX + sp->v->bitmap->w > *max) *max = sp->v->bitmap->w + sp->v->posX;
    int pixelCoord;
    uint24_RGB bg;
    for(int j=0;j<sp->v->bitmap->h;++j) {
        for(int i=0;i<sp->v->bitmap->w;++i) {
            pixelCoord = coordToBufIndex(i+sp->v->posX, j+sp->v->posY);
            TARGET[pixelCoord].pixelR = bgbuf[pixelCoord].pixelR;
            TARGET[pixelCoord].pixelG = bgbuf[pixelCoord].pixelG;
            TARGET[pixelCoord].pixelB = bgbuf[pixelCoord].pixelB;
        }
    }
}

void delete_marked_node(SPRITE_NODE* del);
void* blit_and_send_spi(void* arg) {
    uint24_RGB* INTERNAL_BACK_BUFFER = malloc(320*240*sizeof(uint24_RGB));
    spi_device_handle_t s = *(spi_device_handle_t*) arg;
    pthread_mutex_lock(&sprite_lock);
    while(KILL_BLIT_SPI_THREAD) {
        pthread_cond_wait(&enable_draw, &sprite_lock);
        if(COPY_BG)
            memcpy(INTERNAL_BACK_BUFFER, bgbuf, 320*240*sizeof(uint24_RGB));

        //blit.
        int minX = 320;
        int maxX = 0;
        SPRITE_NODE* sp = sprite_list;
        SPRITE_NODE* nextsp;
        while(sp != NULL) {
            nextsp = sp->n;
            if(!sp->lifetime || sp->clear) {
                undraw(INTERNAL_BACK_BUFFER, sp, &maxX, &minX);
                if(sp->clear)
                    sp->clear = 0;
                else
                    delete_marked_node(sp);
            }
            sp = nextsp;
        }
        sp = sprite_list;
        while(sp != NULL) {
            blit(INTERNAL_BACK_BUFFER, sp, &maxX, &minX);
            sp->lifetime--;
            sp = sp->n;
        }

        // minX = 0;
        // maxX = 320;

        //blit done! send SPI
        const int BLOCKHEIGHT = 16;
        int numLines;
        for(int currX=minX;currX<maxX;currX+=BLOCKHEIGHT) {
            numLines = maxX - currX > BLOCKHEIGHT ? BLOCKHEIGHT : maxX - currX;
            send_lines(s, currX, INTERNAL_BACK_BUFFER+(240*currX), numLines);
            // ets_printf("line: %d %d\n", currX, numLines);
            // vTaskDelay(100 / portTICK_PERIOD_MS);
            send_line_finish(s);
        }
    }
    pthread_mutex_unlock(&sprite_lock);
    free(INTERNAL_BACK_BUFFER);
    return NULL;
}

void blit_bg() {
    COPY_BG = 1;
}

void init_oam() {
    sprite_list = NULL;
    pthread_mutexattr_t mutex_attr;
    pthread_mutexattr_init(&mutex_attr);
    int ret = pthread_mutexattr_settype(&mutex_attr, PTHREAD_MUTEX_ERRORCHECK);
    if(ret)
        ets_printf("pthread_mutexattr_settype: %d\n", ret);
    sprite_lock = PTHREAD_MUTEX_INITIALIZER;
    enable_draw = PTHREAD_COND_INITIALIZER;
    KILL_BLIT_SPI_THREAD = 1;
    COPY_BG = 0;
    bgbuf = malloc(320*240*sizeof(uint24_RGB));

    pthread_create(&blitting_spi_id, NULL, &blit_and_send_spi, &spi);
}

SPRITE_NODE* push(SPRITE_24_H* sprite) {
    SPRITE_NODE* ins = malloc(sizeof(SPRITE_NODE));
    ins->v = sprite;
    ins->p = NULL;
    ins->n = sprite_list;
    if(sprite_list)
        sprite_list->p = ins;
    ins->lifetime = 0xffffffff;
    sprite_list = ins;
    return ins;
}

void wait_for_end_of_frame() {
    pthread_mutex_lock(&sprite_lock);
    pthread_mutex_unlock(&sprite_lock);
}

SPRITE_NODE* init_sprite(SPRITE_BITMAP* bitmap, uint16_t posX, uint16_t posY, uint24_RGB fg, uint24_RGB bg, bool bgcol, bool flipX, bool flipY, bool draw, bool persistent) {
    SPRITE_24_H* sprite = (SPRITE_24_H*) malloc(sizeof(SPRITE_24_H));
    bitmap->refcount++;
    sprite->bitmap = bitmap;
    if(bitmap->w > 10000 || bitmap->h > 10000)
        ets_printf("SPRITE %x AT %d %d IS SUS\n", sprite, posX, posY);
    sprite->posX = posX;
    sprite->posY = posY;
    sprite->bg = bg;
    sprite->fg = fg;
    sprite->bgcol = bgcol;
    sprite->flipX = flipX;
    sprite->flipY = flipY;
    sprite->draw = draw;

    return push(sprite);
}

void set_sprite_fg(SPRITE_NODE* sprite, uint24_RGB x) {
    sprite->v->fg = x;
}

void set_sprite_bg(SPRITE_NODE* sprite, uint24_RGB x, bool iscolor) {
    sprite->v->bg = x;
}

void draw_all_sprites(spi_device_handle_t spi) {
    pthread_cond_signal(&enable_draw);
}

void undraw_node(SPRITE_NODE* n) {
    n->clear = 1;
    n->v->draw = 0;
    return;
}

void delete_node(SPRITE_NODE* del) {
    // del->toBeDeleted = 1;
    del->lifetime = 0;
    return;
    // int k;
    // if((k = pthread_mutex_trylock(&sprite_lock))) {
    //     del->toBeDeleted = 1;
    //     return;
    //     // vTaskDelay(10 / portTICK_PERIOD_MS);
    // }
    
    // delete_marked_node(del);

    // pthread_mutex_unlock(&sprite_lock);
}

void set_sprites_lifetime(int lifetime, SPRITE_NODE** list, int num) {
    for(int i=0;i<num;++i)
        list[i]->lifetime = lifetime;
}

void delete_marked_node(SPRITE_NODE* del) {
    if(del == NULL || del->lifetime)
        return;
    // ets_printf("DELETION: %d\n", del->toBeDeleted);

    if(sprite_list == del)
        sprite_list = del->n;
    if(del->n)
        del->n->p = del->p;
    if(del->p)
        del->p->n = del->n;

    //del is unlinked.
    del->v->bitmap->refcount--;
    if(del->v->bitmap->refcount == 0) {
        free(del->v->bitmap->c);
        free(del->v->bitmap);
    }
    free(del->v);
    free(del);
}

void draw_sprites(spi_device_handle_t spi, SPRITE_NODE** array, int numspr) {
    for(int i=0;i<numspr;++i) {
        array[i]->v->draw = true;
    }
    draw_all_sprites(spi);
}

void delete_persistent_sprites() {
    for(SPRITE_NODE* sp = sprite_list; sp != NULL; sp = sp->n)
        delete_node(sp);
}

void delete_all_sprites_immediate() {
    pthread_mutex_lock(&sprite_lock);
    // for(SPRITE_NODE* sp = sprite_list; sp != NULL; sp = sp->n)
    while(sprite_list) {
        sprite_list->lifetime = 0;
        delete_marked_node(sprite_list);
    }
    pthread_mutex_unlock(&sprite_lock);
}

void delete_temporary_sprites() {
    return;
}

SPRITE_NODE* sprite_rectangle(uint16_t x, uint16_t y, uint16_t w, uint16_t h, uint24_RGB* col, bool persistent, uint8_t alpha) {
    uint24_RGB* spritebuf = (uint24_RGB*) malloc(w * h * 3);
    SPRITE_BITMAP* bitmap = malloc(sizeof(SPRITE_BITMAP));
    for(int i=0;i<h*w;++i) {
        spritebuf[i].pixelR = alpha;
        spritebuf[i].pixelG = alpha;
        spritebuf[i].pixelB = alpha;
    }
    bitmap->c = spritebuf;
    bitmap->refcount = 0;
    bitmap->w = w;
    bitmap->h = h;
    return init_sprite(bitmap, x, 240-y-h, *col, BLACK, false, false, false, true, persistent);
}

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
		background_color = &FILLCOLOR;
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
