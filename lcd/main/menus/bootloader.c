#include <rom/ets_sys.h>
#include "menus.h"
#include <freetype/ftsizes.h>
#include "button.h"
#include "rotenc.h"
#include "menu_data.h"
#include "oam.h"
#include "ILIDriver.h"
#include "lua_exports.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);

extern FT_Face typeFace;
extern QueueHandle_t* button_events;
extern rotary_encoder_info_t* infop;
extern spi_device_handle_t spi;
extern uint24_RGB* background_color;
extern const uint24_RGB WHITE;

char languages[][15] = {
    "English",
    "Francais",
    "NHG"
};
static int menufunc_setup(void) {
    int error;
    button_event_t event;
    rotary_encoder_event_t rotencev;
    int currlang = 0;
    FT_ERR_HANDLE(FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0), "FT_Set_Char_Size");
    while(true) {
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3)
                return -2;
            if(event.pin == 0)
                return -1;
        }
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            currlang = ((unsigned) rotencev.state.position) % 3;
            int sprs[15];
            sprite_rectangle(220, 73, 100, 22, background_color);
            int err = draw_text(220, 152, &languages[currlang][0], typeFace, &sprs[0], &WHITE, background_color);
            draw_all_sprites(spi);
            delete_all_sprites();
        }
    }
}

static int menufunc_welcome(void) {
    button_event_t event;
    while(true) {
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18) {
            return 1;
        }
    }
}

MENU_INFO_t allmenus[] = {
    {&welcome_menu[0], 3, menufunc_welcome},
    {&menusetup0[0], 8, menufunc_setup}
};

int start_menu_tree(int startmenu) {
    int menu_stack[20];
    int menu_stackp = 0;
    int nextmenu;
    MENU_INFO_t* currmenu;
    menu_stack[menu_stackp] = startmenu;
    do {
        send_color(spi, background_color);
        currmenu = &allmenus[menu_stack[menu_stackp]];
        draw_menu_elements(currmenu->background, typeFace, currmenu->num_elements);
        draw_all_sprites(spi);
        delete_all_sprites();
        nextmenu = currmenu->menu_functionality();
        if(nextmenu == -1) {
            menu_stackp--;
            if(menu_stackp < 0) {
                menu_stackp = 0;
            }
        } else {
            menu_stackp++;
            menu_stack[menu_stackp] = nextmenu;
        }
    } while(nextmenu != -2);
    return menu_stackp;
}

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
    
