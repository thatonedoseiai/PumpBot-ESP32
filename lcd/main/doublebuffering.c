#include "oam.h"
#include <pthread.h>

#define ALPHA_COMP(a, f, b) (((a*f) + (255-a)*b) / 255)

SPRITE_NODE* sprite_list;
pthread_mutex_t sprite_lock;
pthread_cond_t enable_draw;
pthread_t blitting_spi_id;
unsigned char KILL_BLIT_SPI_THREAD;

extern spi_device_handle_t spi;

int coordToBufIndex(int x, int y) {
    return 320 * y + x;
}

void* blit_and_send_spi(void* arg) {
    uint24_RGB* INTERNAL_BACK_BUFFER = malloc(320*240*sizeof(uint24_RGB));
    spi_device_handle_t s = *(spi_device_handle_t*) arg;
    pthread_mutex_lock(&sprite_lock);
    while(KILL_BLIT_SPI_THREAD) {
        pthread_cond_wait(&enable_draw, &sprite_lock);

        //blit.
        int minY = 240;
        int maxY = 0;
        for(SPRITE_NODE* sp = sprite_list; sp != NULL; sp=sp->n) {
            if(!sp->v->draw)
                continue;
            if(sp->v->posY < minY) minY = sp->v->posY;
            if(sp->v->posY > maxY) maxY = sp->v->posY;
            int alphaR, alphaG, alphaB;
            uint24_RGB bg;
            for(int i=0;i<sp->v->bitmap->h;++i) {
                for(int j=0;j<sp->v->bitmap->w;++i) {
                    int alphaR = sp->v->bitmap->c[j*sp->v->bitmap->w+i].pixelR;
                    int alphaG = sp->v->bitmap->c[j*sp->v->bitmap->w+i].pixelG;
                    int alphaB = sp->v->bitmap->c[j*sp->v->bitmap->w+i].pixelB;
                    bg = sp->v->bgiscolor ? sp->v->bg : INTERNAL_BACK_BUFFER[coordToBufIndex(j, i)]
                    INTERNAL_BACK_BUFFER[coordToBufIndex(j, i)].pixelR = ALPHA_COMP(alphaR, sp->v->fg, bg.pixelR);
                    INTERNAL_BACK_BUFFER[coordToBufIndex(j, i)].pixelG = ALPHA_COMP(alphaG, sp->v->fg, bg.pixelG);
                    INTERNAL_BACK_BUFFER[coordToBufIndex(j, i)].pixelB = ALPHA_COMP(alphaB, sp->v->fg, bg.pixelB);
                }
            }
        }

        //blit done! send SPI
        const int BLOCKHEIGHT = 16;
        int numLines;
        for(int currY=minY;currY<maxY;currY+=BLOCKHEIGHT) {
            numLines = maxY - currY > BLOCKHEIGHT ? BLOCKHEIGHT : maxY - currY;
            send_lines(s, currY, INTERNAL_BACK_BUFFER+(320*currY), numLines);
            send_line_finish(s);
        }
    }
    pthread_mutex_unlock(&sprite_lock);
    free(INTERNAL_BACK_BUFFER);
    return NULL;
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

    pthread_create(&blitting_spi_id, NULL, &blit_and_send_spi, &spi);
}

void push(SPRITE_24_H* sprite) {
    SPRITE_NODE* ins = malloc(sizeof(SPRITE_NODE));
    ins->v = sprite;
    ins->p = NULL;
    ins->n = *sprite_list;
    if(sprite_list != NULL)
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

void delete_node(SPRITE_NODE* del) {
    int k;
    while((k = pthread_mutex_trylock(&sprite_lock))) {
	vTaskDelay(10 / portTICK_PERIOD_MS);
    }

    if(del == NULL)
        return;
    if(sprite_list == del)
        sprite_list = del->n;
    if(del->n)
        del->n->p = del->p;
    if(del->p)
        del->p->n = del->n;

    pthread_mutex_unlock(&sprite_lock);

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
	array[i]->draw = true;
    }
    draw_all_sprites(spi);
}

void delete_persistent_sprites() {
    while(sprite_list)
	delete_node(sprite_list);
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
    return init_sprite(bitmap, x, 240-y-h, *col, black, false, false, false, true, persistent);
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