#pragma once

#ifndef MENUS_H
#define MENUS_H

#include "oam.h"
#include <freetype/freetype.h>

typedef struct {
    char* text;
    uint16_t x;
    uint16_t y;
    uint8_t textlen;
    bool center;
    int textsize;
} MENU_ELEMENT;

/*
 * draw a collection of menu elements to the screen
 */
int draw_menu_elements(const MENU_ELEMENT* elems, FT_Face typeFace, int numElements);

int draw_text(int startX, int startY, char* string, FT_Face typeFace, int* sprites);

#endif
