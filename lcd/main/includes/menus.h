#ifndef MENUS_H
#define MENUS_H

#pragma once

#include "oam.h"
#include <freetype/freetype.h>

#define MENU_FLAG_IS_HLINE 0x1
#define MENU_FLAG_IS_VLINE 0x2
#define MENU_FLAG_CENTER 0x4
#define MENU_FLAG_RIGHT_JUSTIFY 0x8
#define MENU_FLAG_LANGUAGE_AGNOSTIC 0x10

typedef struct {
    const char* const* text;
    uint16_t x;
    uint16_t y;
    int textsize;
    unsigned char flags;
    uint24_RGB** col;
} MENU_ELEMENT;

typedef struct {
    const MENU_ELEMENT* background;
    int num_elements;
    int(*menu_functionality)(void);
} MENU_INFO_t;

/*
 * draw a collection of menu elements to the screen
 */
int start_menu_tree(int, char);

int draw_menu_elements(const MENU_ELEMENT* elems, FT_Face typeFace, int numElements);

/* int draw_text(int startX, int startY, char* string, FT_Face typeFace, int* sprites, uint24_RGB* color, uint24_RGB* bgcol); */

int draw_hline(int y, int thickness, uint24_RGB* colour);

int draw_vline(int x, int thickness, uint24_RGB* colour);

#endif
