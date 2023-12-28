#include "menus.h"

int draw_menu_elements(const MENU_ELEMENT* elems, FT_Face typeFace, int numElements) {
    int err;
    for (int i = 0; i < numElements; i++) {
        int spriteArray[elems[i].textlen];
        err = draw_text(elems[i].x, elems[i].y, elems[i].text, typeFace, &spriteArray[0]);
        if (elems[i].center)
            center_sprite_group_x(spriteArray, elems[i].textlen);
        if (err)
            return err;
    }
    return 0;
}
