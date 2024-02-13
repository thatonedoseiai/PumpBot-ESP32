#include <rom/ets_sys.h>
#include "menus.h"
#include <freetype/ftsizes.h>


// #pragma GCC push_options
// #pragma GCC optimize ("O0")

int draw_menu_elements(const MENU_ELEMENT* elems, FT_Face typeFace, int numElements) {
    int err;
    static int sizeControl = 0;
//    static FT_Size_RequestRec ftsize = {
//        .type = FT_SIZE_REQUEST_TYPE_NOMINAL,
//        .width = 0,
//        .height = 0,
//        .horiResolution = 150,
//        .vertResolution = 0
//    };

    for (int i = 0; i < numElements; i++) {
        if (elems[i].hline) {
            draw_hline(elems[i].y, elems[i].textsize, elems[i].col);
            continue;
        }
        if (elems[i].vline) {
            draw_vline(elems[i].x, elems[i].textsize, elems[i].col);
            continue;
        }
        if (sizeControl != elems[i].textsize) {
            err = FT_Set_Char_Size (typeFace, elems[i].textsize << 6, 0, 100, 0); // 0 = copy last value
            // ftsize.width = elems[i].textsize << 6;
            // err = FT_Request_Size(typeFace, &ftsize);
            if (err) {
                ets_printf("!!error in draw_menu_elements: could not set size.\n");
                return 1;
            }
            sizeControl = elems[i].textsize;
        }

        // ets_printf("%d %d", elems[i].textlen, elems[i].numspaces);
        int spriteArray[elems[i].textlen - elems[i].numspaces];

        err = draw_text(elems[i].x, elems[i].y, elems[i].text, typeFace, &spriteArray[0], &elems[i].col, NULL);
        if (elems[i].center)
            center_sprite_group_x(spriteArray, elems[i].textlen - elems[i].numspaces);
        if (err)
            return err;
    }
    return 0;
}

// #pragma GCC pop_options

int draw_hline(int y, int thickness, uint24_RGB colour) {
    uint24_RGB* spriteBuf = (uint24_RGB*) malloc(320*thickness*sizeof(uint24_RGB));
    SPRITE_BITMAP* bmp = (SPRITE_BITMAP*) malloc(sizeof(SPRITE_BITMAP));
    bmp->refcount = 1;
    bmp->c = spriteBuf;
    for(int p=0;p<320*thickness;p++) {
        spriteBuf[p].pixelB = colour.pixelB;
        spriteBuf[p].pixelG = colour.pixelG;
        spriteBuf[p].pixelR = colour.pixelR;
    }
    return init_sprite(bmp, 0, y, 320, thickness, false, false, true);
}

int draw_vline(int x, int thickness, uint24_RGB colour) {
    uint24_RGB* spriteBuf = (uint24_RGB*) malloc(320*thickness*sizeof(uint24_RGB));
    SPRITE_BITMAP* bmp = (SPRITE_BITMAP*) malloc(sizeof(SPRITE_BITMAP));
    bmp->refcount = 1;
    bmp->c = spriteBuf;
    for(int p=0;p<320*thickness;p++) {
        spriteBuf[p].pixelB = colour.pixelB;
        spriteBuf[p].pixelG = colour.pixelG;
        spriteBuf[p].pixelR = colour.pixelR;
    }
    return init_sprite(bmp,x, 0, thickness, 240, false, false, true);
}
    
