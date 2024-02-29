#include <rom/ets_sys.h>
#include "menus.h"
#include "menu_data.h"
#include <freetype/ftsizes.h>
#include "button.h"
#include "rotenc.h"
#include "oam.h"
#include "ILIDriver.h"
#include "lua_exports.h"

#include "esp_wifi.h"
#include "board_config.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);

extern FT_Face typeFace;
extern QueueHandle_t* button_events;
extern rotary_encoder_info_t* infop;
extern spi_device_handle_t spi;
extern uint24_RGB* background_color;
extern const uint24_RGB WHITE;
extern char connect_flag;
extern SPRITE_24_H** OAM_SPRITE_TABLE;

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
            sprite_rectangle(220, 240-73-22, 100, 22, background_color);
            int err = draw_text(220, 152, &languages[currlang][0], typeFace, &sprs[0], &WHITE, background_color);
            draw_all_sprites(spi);
            delete_all_sprites();
        }
    }
}

// todo: add scrolling, 10 elements ok;
static int menufunc_wifi_scan() {
    int ys[] = {184, 152, 120, 88, 56};

    int error;
    uint8_t selection = 0;
    uint16_t aprecnum = 5;
    wifi_ap_record_t ap_info[5];
    uint16_t ap_count = 0;
    memset(ap_info, 0, sizeof(ap_info));
    connect_flag = 0;
    int textbg = sprite_rectangle(50, 184, 220, 21, background_color);
    int cursorbg = sprite_rectangle(10, 184, 20, 16, background_color);
    int cursor;
    button_event_t event;
    rotary_encoder_event_t rotencev;
    error = draw_text(10, 184, ">", typeFace, &cursor, &WHITE, background_color);
    OAM_SPRITE_TABLE[cursor]->draw = false;
    OAM_SPRITE_TABLE[cursorbg]->draw = false;
    ESP_ERROR_CHECK(esp_wifi_start());
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &wifi_config));

refresh:
    FT_ERR_HANDLE(FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0), "FT_Set_Char_Size");
    int sprs[32];
    error = draw_text(0, 184, "Searching...", typeFace, &sprs[0], &WHITE, background_color);
    center_sprite_group_x(sprs, 12);
    draw_all_sprites(spi);
    for(int i=0;i<12;++i) {
        delete_sprite(sprs[i]);
    }

    ESP_ERROR_CHECK(esp_wifi_scan_start(NULL, true));
    ESP_ERROR_CHECK(esp_wifi_scan_get_ap_records(&aprecnum, ap_info));
    ESP_ERROR_CHECK(esp_wifi_scan_get_ap_num(&ap_count));

    int name_length;
    for(int i=0;i<aprecnum;++i) {
        error = draw_text(0, ys[i], (char*)ap_info[i].ssid, typeFace, &sprs[0], &WHITE, background_color);
        OAM_SPRITE_TABLE[textbg]->posY = 224-ys[i];
        name_length = strlen((char*)ap_info[i].ssid);
        center_sprite_group_x(sprs, name_length);
        draw_all_sprites(spi);
        for(int j=0;j<name_length;++j) {
            delete_sprite(sprs[j]);
        }
    }
    OAM_SPRITE_TABLE[textbg]->posY = 184;
    OAM_SPRITE_TABLE[textbg]->draw = false;
    OAM_SPRITE_TABLE[cursor]->draw = true;
    OAM_SPRITE_TABLE[cursorbg]->draw = true;
    draw_sprites(spi, &cursor, 1);
    draw_sprites(spi, &cursorbg, 1);

    selection = 0;
    while(!connect_flag) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection]-14;
            selection = (rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? (selection + 1) % 5 : (selection - 1) % 5;
            OAM_SPRITE_TABLE[cursor]->posY = 240-ys[selection]-14;
            draw_sprites(spi, &cursor, 1);
            draw_sprites(spi, &cursorbg, 1);
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                OAM_SPRITE_TABLE[cursorbg]->draw = false;
                OAM_SPRITE_TABLE[cursor]->draw = false;
                OAM_SPRITE_TABLE[textbg]->draw = true;
                for(int i=0;i<5;++i) {
                    OAM_SPRITE_TABLE[textbg]->posY = 224-ys[i];
                    draw_sprites(spi, &textbg, 1);
                }
                goto refresh;
            }
            if(event.pin == 18) {
                strncpy((char*)wifi_config.sta.ssid, (char*)ap_info[selection].ssid, 32);
                ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &wifi_config));
                ESP_ERROR_CHECK(esp_wifi_connect());
                ets_printf("connecting...");
                while(!connect_flag);
                ets_printf("success!");
                return -2;
            }
            if(event.pin == 0)
                return -1;
        }

    }
    return -2;
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
    {&menusetup0[0], 8, menufunc_setup},
    {&menusetup3[0], 14, menufunc_wifi_scan},
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
    return init_sprite(bmp, 0, 240-y, 320, thickness, false, false, true);
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
    
