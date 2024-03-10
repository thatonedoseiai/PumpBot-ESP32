#ifndef MENUS_H
#define MENUS_H

#pragma once

#include "oam.h"
#include <freetype/freetype.h>

typedef struct {
    char* text;
    uint16_t x;
    uint16_t y;
    bool center;
    int textsize;
    bool hline;
    bool vline;
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
