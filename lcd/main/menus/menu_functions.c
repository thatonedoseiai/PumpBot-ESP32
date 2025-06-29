#include "menu_functions.h"
#include "fontfile.h"
#include "menus.h"
#include "oam.h"

struct _CONTEXT_wm {
    int counter;
    int currlang;
};
void _SETUP_welcome_menu(void** context) {
    struct _CONTEXT_WM* cont = malloc(sizeof(struct _CONTEXT_wm));
    cont->counter = 200;
    cont->currlang = 0;
    *context = (void*) cont;
    set_font_size(24);
    sprite_rectangle(10, 168, 300, 22, background_color, true, 255);
    sprite_rectangle(10, 146, 300, 22, background_color, true, 255);
    sprite_rectangle(10, 210, 300, 20, background_color, true, 255);
    sprite_rectangle(10, 190, 300, 20, background_color, true, 255);
    sprite_rectangle(10, 5, 300, 20, background_color, true, 255);
}

void _CLEANUP_welcome_menu(void** context) {
    free((struct _CONTEXT_wm**) context);
    *context = NULL; // just to keep it clean
    delete_persistent_sprites();
}

int _BA_ED_welcome_menu(void* context, void* args) {
    (void) args;
    return MENU_SETUP_ONLY_TRANSITION_FLAG | 1;
}

int _POSTLOOP_welcome_menu(void* context, void* args) {
    SPRITE_NODE* sprs[45];
    int numsprs;

    struct _CONTEXT_wm* ct = (struct _CONTEXT_wm*) context;

    ct->counter--;
    if(ct->counter == 0) {
        ct->currlang = (ct->currlang+1) % 9;
        draw_text(60, 195, text_welcome[currlang], sprs, &numsprs, *foreground_color, *background_color, 0, true, false);
        center_sprite_group_x(sprs, numsprs);

        draw_text(60, 154, text_welcome_a[currlang], sprs, &numsprs, *foreground_color, *background_color, 0, true, false);
        center_sprite_group_x(sprs, numsprs);

        set_font_size(14);
        draw_text(60, 10, text_pressenc[currlang], sprs, &numsprs, *foreground_color, *background_color, 0, true, false);
        center_sprite_group_x(sprs, numsprs);

        set_font_size(24);
        draw_all_sprites(spi);
        counter = 200;
    }
}
