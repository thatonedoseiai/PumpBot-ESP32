#ifndef MENUS_H
#define MENUS_H

#pragma once

#include "oam.h"
#include <freetype/freetype.h>

typedef struct {
    char* text;
    uint16_t x;
    uint16_t y;
    uint8_t textlen;
    uint8_t numspaces;
    bool center;
    int textsize;
    bool hline;
    uint24_RGB col;
} MENU_ELEMENT;

/*
 * draw a collection of menu elements to the screen
 */
int draw_menu_elements(const MENU_ELEMENT* elems, FT_Face typeFace, int numElements);

int draw_text(int startX, int startY, char* string, FT_Face typeFace, int* sprites, uint24_RGB* color);

int draw_hline(int y, int thickness, uint24_RGB colour);

int draw_vline(int x, int thickness, uint24_RGB colour);

#endif
