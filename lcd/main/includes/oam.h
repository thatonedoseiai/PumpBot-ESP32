#pragma once
#include <ILIDriver.h>

#define OAM_SIZE 255

#ifdef __cplusplus
extern "C" {
#endif

typedef struct SPRITE_BITMAP_ {
    uint24_RGB* c;
	uint16_t w;
	uint16_t h;
    uint8_t refcount;
} SPRITE_BITMAP;

typedef struct SPRITE_24_H_ {
	SPRITE_BITMAP* bitmap;
	uint16_t posX;
	uint16_t posY;
	uint24_RGB fg;
	uint24_RGB bg;
	bool flipX;
	bool flipY;
	bool bgcol; // true if bg is color
	bool draw;
} SPRITE_24_H;

typedef struct s_postcomp {
	uint24_RGB* coldata;
	uint16_t x;
	uint16_t y;
	uint16_t w;
	uint16_t h;
} SPRITE_POSTCOMP;

typedef struct s_n {
	SPRITE_24_H* v;
	struct s_n* p;
	struct s_n* n;
	unsigned int lifetime;
	unsigned char clear;
	unsigned int INDEX;
} SPRITE_NODE;

typedef struct {
	uint16_t x;
	uint16_t y;
	uint16_t w;
	uint16_t h;
} bounds;

void init_oam();
SPRITE_NODE* init_sprite(SPRITE_BITMAP* bitmap, uint16_t posX, uint16_t posY, uint24_RGB fg, uint24_RGB bg, bool bgcol, bool flipX, bool flipY, bool draw, bool persistent);
void set_sprite_fg(SPRITE_NODE* sprite, uint24_RGB x);
void set_sprite_bg(SPRITE_NODE* sprite, uint24_RGB x, bool iscolor);
void draw_all_sprites(spi_device_handle_t spi);
void delete_node(SPRITE_NODE* del);
void draw_sprites(spi_device_handle_t spi, SPRITE_NODE** array, int numspr);
void delete_persistent_sprites();
void delete_temporary_sprites();
SPRITE_NODE* sprite_rectangle(uint16_t posX, uint16_t posY, uint16_t sizeX, uint16_t sizeY, uint24_RGB* col, bool persistent, uint8_t alpha);
void center_sprite_group_x(SPRITE_NODE** sprites, int numsprites);
void right_justify_sprite_group_x(SPRITE_NODE** sprites, int numsprites, int pad);
void assign_theme_from_settings(void);
void blit_bg(void);
void delete_all_sprites_immediate(void);
void wait_for_end_of_frame(void);
void set_sprites_lifetime(int lifetime, SPRITE_NODE** list, int num);
void undraw_node(SPRITE_NODE* n);

#ifdef __cplusplus
}
#endif
