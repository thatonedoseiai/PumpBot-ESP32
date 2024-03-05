#include <rom/ets_sys.h>
#include "menus.h"
#include "menu_data.h"
#include <freetype/ftsizes.h>
#include "button.h"
#include "rotenc.h"
#include "oam.h"
#include "ILIDriver.h"
#include "lua_exports.h"
#include "settings.h"
#include "utf8.h"

#include "esp_wifi.h"
#include "board_config.h"
#include "file_server.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);
#define MENU_RETURN_FLAG 0x8000
#define MENU_POP_FLAG 0x4000
#define MENU_TEXT_INPUT_FLAG 0x2000
#define IBUF_SIZE 256

extern FT_Face typeFace;
extern QueueHandle_t* button_events;
extern rotary_encoder_info_t* infop;
extern spi_device_handle_t spi;
extern uint24_RGB* background_color;
extern uint24_RGB WHITE;
extern char connect_flag;
extern SPRITE_24_H** OAM_SPRITE_TABLE;
extern SETTINGS_t settings;
static char* ibuf;

uint24_RGB RED = {0xff, 0x00, 0x00};

char languages[][15] = {
    "English",
    "日本語",
    "Français",
    "Español",
    "Português",
    "简化中文",
    "繁體中文",
    "Pусский",
    "Deutsch",
};
static int menufunc_setup(void) {
    int error;
    button_event_t event;
    rotary_encoder_event_t rotencev;
    int currlang = 0;
    FT_ERR_HANDLE(FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0), "FT_Set_Char_Size");
    while(true) {
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3) {
                settings.language = currlang;
                return MENU_RETURN_FLAG; // replace with "how would you like to setup pumpbot? screen"
            }
            if(event.pin == 0)
                return MENU_POP_FLAG;
        }
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            currlang = ((unsigned) rotencev.state.position) % 3;
            int sprs[15];
            sprite_rectangle(220, 240-73-22, 100, 22, background_color);
            draw_text(220, 152, &languages[currlang][0], typeFace, &sprs[0], &WHITE, background_color);
            draw_all_sprites(spi);
            delete_all_sprites();
        }
    }
}

static void draw_options(char** options, int bgrect) {
    int ys[] = {184, 152, 120, 88, 56};
    int sprs[32];

    int name_length = 0;
    OAM_SPRITE_TABLE[bgrect]->draw = true;
    for(int i=0;i<5;++i) {
        if(options[i] != NULL) {
            draw_text(0, ys[i], options[i], typeFace, &sprs[0], &WHITE, background_color);
            name_length = strlen(options[i]);
            center_sprite_group_x(sprs, name_length);
        }
        OAM_SPRITE_TABLE[bgrect]->posY = 224-ys[i];
        draw_all_sprites(spi);
        if(options[i] != NULL)
            for(int j=0;j<name_length;++j)
                delete_sprite(sprs[j]);
    }
    OAM_SPRITE_TABLE[bgrect]->posY = 184;
    OAM_SPRITE_TABLE[bgrect]->draw = false;
}

char SEARCH_TEXT[] = "Searching...";
static int menufunc_wifi_scan() {
    int ys[] = {184, 152, 120, 88, 56};

    int error;
    char* list_options[5];
    uint8_t selection = 0;
    uint8_t page_start = 0;
    uint16_t aprecnum = 10;
    wifi_ap_record_t ap_info[10];
    uint16_t ap_count = 0;
    memset(ap_info, 0, sizeof(ap_info));
    connect_flag = 0;
    FT_ERR_HANDLE(FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0), "FT_Set_Char_Size");
    int textbg = sprite_rectangle(50, 184, 220, 21, background_color);
    int cursorbg = sprite_rectangle(10, 184, 20, 16, background_color);
    int cursor;
    button_event_t event;
    rotary_encoder_event_t rotencev;
    error = draw_text(10, 184, ">", typeFace, &cursor, &WHITE, background_color);
    OAM_SPRITE_TABLE[cursor]->draw = false;
    OAM_SPRITE_TABLE[cursorbg]->draw = false;
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));

refresh:
    list_options[0] = SEARCH_TEXT;
    for(int i=1;i<5;++i) {
        list_options[i] = NULL;
    }
    draw_options(list_options, textbg);

    ESP_ERROR_CHECK(esp_wifi_scan_start(NULL, true));
    ESP_ERROR_CHECK(esp_wifi_scan_get_ap_records(&aprecnum, ap_info));
    ESP_ERROR_CHECK(esp_wifi_scan_get_ap_num(&ap_count));

    for(int i=0;i<5;++i) {
        list_options[i] = (i < aprecnum) ? (char*)ap_info[i].ssid : NULL;
    }

    draw_options(list_options, textbg);
    OAM_SPRITE_TABLE[cursor]->draw = true;
    OAM_SPRITE_TABLE[cursorbg]->draw = true;
    draw_sprites(spi, &cursorbg, 1);
    draw_sprites(spi, &cursor, 1);
    if(ap_count > 10)
        ap_count = 10;

    selection = 0;
    while(!connect_flag) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection - page_start]-14;
            selection = (rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? (selection + 1) % ap_count : (selection + ap_count - 1) % ap_count;
            if(selection > page_start + 4 || selection < page_start) {
                if(selection < page_start) {
                    page_start = selection;
                } else if (selection > page_start + 4) {
                    page_start = selection - 4;
                }
                for(int i=0;i<5;++i)
                    list_options[i] = (i < aprecnum) ? (char*)ap_info[i+page_start].ssid : NULL; // should never be null.
                draw_options(list_options, textbg);
            }
            OAM_SPRITE_TABLE[cursor]->posY = 240-ys[selection - page_start]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE && event.event == BUTTON_DOWN) {
            if(event.pin == 3) {
                OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection]-14;
                draw_sprites(spi, &cursorbg, 1);
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
                strncpy(&settings.wifi_name[0], (char*)ap_info[selection].ssid, 32);
                delete_all_sprites();
                // return MENU_TEXT_INPUT_FLAG | 4;
                return 6;
            }
            if(event.pin == 0)
                return MENU_POP_FLAG;
        }
    }
    return MENU_RETURN_FLAG;
}

unsigned char table1[] = "EF⌫✓ABCDEFGHIJKLMNOPQRSTUVWXYZ␣⌫✓ABCD";
unsigned char table2[] = "ef⌫✓abcdefghijklmnopqrstuvwxyz␣⌫✓abcd";
// unsigned char table3[] = "%^⌫✓!@#$%^&*().,+-\"0123456789~␣⌫✓!@#$";
unsigned char table3[] = "?@⌫✓!\"#$%&'()*+,-./0123456789:;<=>?@␣⌫✓!\"#$%";
unsigned char* metatable[] = {table1, table2, table3};
unsigned char string_lengths[] = {29, 29, 36};
char tablename[][2] = {"a", "@", "A"};
int xs[] = {7, 42, 77, 112, 147, 182, 217, 252, 287};
static void draw_textreel(unsigned int curtable, unsigned int selection, unsigned char** loc) {
    unsigned char visibleBuffer[27];
    *loc = &metatable[curtable][utf8substrlen((char*)metatable[curtable], selection)];
    int sprs[9];
    int substrend = utf8substrlen((char*) *loc, 9);
    for(int i=0;i<substrend;++i) {
        visibleBuffer[i] = (*loc)[i];
    }
    visibleBuffer[substrend] = 0;
    for(int i=0;i<9;++i)
        sprite_rectangle(xs[i], 113, 26, 27, background_color);
    draw_text(0, 120, (char*) visibleBuffer, typeFace, sprs, &WHITE, background_color);
    for(int i=0;i<9;++i) {
        OAM_SPRITE_TABLE[sprs[i]]->posX = xs[i];
    }
    draw_all_sprites(spi);
    delete_all_sprites();
}

static int menufunc_text_write(void) {
    if(ibuf == NULL) {
        return MENU_POP_FLAG;
    }
    button_event_t event;
    rotary_encoder_event_t rotencev;
    // char ibuf[256];
    // memset(ibuf, 0, IBUF_SIZE);
    unsigned char* selectedchar;
    unsigned char* loc;
    int error;
    uint8_t cursor = strlen(ibuf);
    uint8_t numtyped = utf8strlen(ibuf);
    unsigned int selection = 0;
    unsigned int curtable = 0;
    FT_ERR_HANDLE(FT_Set_Char_Size(typeFace, 18 << 6, 0, 100, 0), "FT_Set_Char_Size");
    draw_text(0, 184, ibuf, typeFace, NULL, &WHITE, background_color);
    draw_text(300, 3, "a", typeFace, NULL, &WHITE, background_color);
    draw_textreel(curtable, selection, &loc);

    while(true) {
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                curtable = (curtable + 1) % 3;
                sprite_rectangle(300, 0, 20, 22, background_color);
                draw_text(300, 3, tablename[curtable], typeFace, NULL, &WHITE, background_color);
                draw_textreel(curtable, selection, &loc);
            }
            if(event.pin == 18 && event.event == BUTTON_DOWN && numtyped < 64) {
                selectedchar = loc + utf8substrlen((char*) loc, 4);
                // ets_printf("typing!\n");
                if(selectedchar[0] == 0xe2 &&
                    selectedchar[1] == 0x8c &&
                    selectedchar[2] == 0xab) {
                    if(numtyped > 0) {
                        utf8bspc((char*) ibuf, &cursor);
                        numtyped--;
                    }
                    // ets_printf("input: %s\n", ibuf);
                } else if(selectedchar[0] == 0xe2 &&
                        selectedchar[1] == 0x9c &&
                        selectedchar[2] == 0x93) {
                    break;
                } else if(selectedchar[0] == 0xe2 &&
                        selectedchar[1] == 0x90 &&
                        selectedchar[2] == 0xa3) {
                    ibuf[cursor] = ' ';
                    cursor++;
                    // ets_printf("input: %s\n", ibuf);
                    numtyped++;
                } else {
                    utf8cpychr(&ibuf[cursor], (char*) selectedchar, &cursor);
                    // ets_printf("input: %s\n", ibuf);
                    numtyped++;
                }
                sprite_rectangle(0, 191, 320, 14, background_color);
                sprite_rectangle(0, 179, 320, 12, background_color);
                draw_text(0, 184, ibuf, typeFace, NULL, &WHITE, background_color);
                draw_all_sprites(spi);
                delete_all_sprites();
            } 
        }
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) {
                selection = (selection + 1) % string_lengths[curtable];
            } else {
                selection = (selection + string_lengths[curtable] - 1) % string_lengths[curtable];
            }
            draw_textreel(curtable, selection, &loc);
        }
    }
    return MENU_POP_FLAG;
}

static int menufunc_welcome(void) {
    button_event_t event;
    while(true) {
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18) {
            return 1;
        }
    }
}

static int menufunc_connect_wifi(void) {
    // if(ibuf == NULL)
    //     return MENU_POP_FLAG;
    // strncpy(settings.wifi_pass, ibuf, 64);
    // free(ibuf);
    // ibuf = NULL;

    strncpy((char*)sta_wifi_config.sta.ssid, (char*)&settings.wifi_name[0], 32);
    strncpy((char*)sta_wifi_config.sta.password, (char*)&settings.wifi_pass[0], 64);
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));
    ESP_ERROR_CHECK(esp_wifi_connect());
    ets_printf("connecting...");
    while(!connect_flag);
    ets_printf("success!");
    return MENU_RETURN_FLAG;
}

static int menufunc_http_setup(void) {
    connect_flag = 0;
    ESP_ERROR_CHECK(example_start_file_server("/mainfs"));
    button_event_t event;
    while(!connect_flag) {
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE && event.pin == 3) {
            stop_file_server();
            return MENU_POP_FLAG;
        }
    }
    stop_file_server();
    return MENU_RETURN_FLAG;
}

static int menufunc_network_preview(void) {
    int ys[] = {184, 152, 120, 88, 56};
    char namebuf[14];
    int k[8];
    button_event_t event;
    rotary_encoder_event_t rotencev;

    if(ibuf != NULL) {
        strncpy(settings.wifi_pass, ibuf, 64);
        free(ibuf);
        ibuf = NULL;
    }

    draw_text(32, 184, "Network", typeFace, NULL, &WHITE, background_color);
    draw_text(32, 152, "Password", typeFace, NULL, &WHITE, background_color);
    draw_text(32, 120, "Advanced", typeFace, k, &WHITE, background_color);
    center_sprite_group_x(k, 8);
    draw_text(32, 88, "Connect", typeFace, k, &WHITE, background_color);
    center_sprite_group_x(k, 7);
    if(strlen(settings.wifi_name) > 10) {
        for(int i=0;i<10;++i) {
            namebuf[i] = settings.wifi_name[i];
        }
        namebuf[11] = '.';
        namebuf[12] = '.';
        namebuf[13] = '.';
        namebuf[14] = 0;
        draw_text(150, 184, namebuf, typeFace, NULL, &WHITE, background_color);
    } else {
        draw_text(150, 184, &settings.wifi_name[0], typeFace, NULL, &WHITE, background_color);
    }
    if(strlen(settings.wifi_pass) > 10) {
        for(int i=0;i<10;++i) {
            namebuf[i] = settings.wifi_pass[i];
        }
        namebuf[11] = '.';
        namebuf[12] = '.';
        namebuf[13] = '.';
        namebuf[14] = 0;
        draw_text(150, 152, namebuf, typeFace, NULL, &WHITE, background_color);
    } else {
        draw_text(150, 152, &settings.wifi_pass[0], typeFace, NULL, &WHITE, background_color);
    }
    draw_all_sprites(spi);
    delete_all_sprites();
    int cursorbg = sprite_rectangle(2, 184, 20, 16, background_color);
    int cursor;
    int selection = 0;
    draw_text(2, 184, ">", typeFace, &cursor, &WHITE, background_color);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection]-14;
            selection = (rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? (selection + 1) % 4 : (selection + 3) % 4;
            OAM_SPRITE_TABLE[cursor]->posY = 240-ys[selection]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                // draw_text(0, 88, );
                return MENU_RETURN_FLAG;
            }
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                // strncpy(&settings.wifi_name[0], (char*)ap_info[selection].ssid, 32);
                // ibuf = malloc(256 * sizeof(char));
                // delete_all_sprites();
                // return MENU_TEXT_INPUT_FLAG | 4;
                switch(selection) {
                case 0:
                    memset(&settings.wifi_pass[0], 0, 64);
                    delete_all_sprites();
                    return MENU_POP_FLAG;
                case 1:
                    ibuf = calloc(256, sizeof(char));
                    strcpy(ibuf, &settings.wifi_pass[0]);
                    delete_all_sprites();
                    return 3;
                case 2:
                    break;
                case 3:
                    delete_all_sprites();
                    return 4;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                memset(&settings.wifi_pass[0], 0, 64);
                delete_all_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
    return 4;
}

MENU_INFO_t allmenus[] = {
    {&welcome_menu[0], 3, menufunc_welcome},
    {&menusetup0[0], 8, menufunc_setup},
    {&menusetup3[0], 9, menufunc_wifi_scan},
    {&menusetup3[0], 9, menufunc_text_write},
    {&menuwifistarting[0], 3, menufunc_connect_wifi},
    {&menuwifistarting[0], 3, menufunc_http_setup},
    {&menusetup3[0], 9, menufunc_network_preview},
};

int start_menu_tree(int startmenu) {
    int menu_stack[20];
    int menu_stackp = 0;
    int nextmenu;
    char do_text_menu = 0;
    MENU_INFO_t* currmenu;
    menu_stack[menu_stackp] = startmenu;
    do {
        send_color(spi, background_color);
        // if(do_text_menu) {
        //     draw_menu_elements(allmenus[3].background, typeFace, allmenus[3].num_elements);
        //     draw_all_sprites(spi);
        //     delete_all_sprites();
        //     (void) allmenus[3].menu_functionality();
        //     do_text_menu = 0;
        //     send_color(spi, background_color);
        // }
        currmenu = &allmenus[menu_stack[menu_stackp]];
        draw_menu_elements(currmenu->background, typeFace, currmenu->num_elements);
        draw_all_sprites(spi);
        delete_all_sprites();
        nextmenu = currmenu->menu_functionality();
        if(nextmenu & MENU_POP_FLAG) {
            menu_stackp--;
            if(menu_stackp < 0) {
                menu_stackp = 0;
            }
        } else {
            // if (nextmenu & MENU_TEXT_INPUT_FLAG) {
            //     do_text_menu = true;
            //     nextmenu = nextmenu & 0xff;
            // }
            menu_stackp++;
            menu_stack[menu_stackp] = nextmenu;
        }
    } while((nextmenu & MENU_RETURN_FLAG) == 0);
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

        err = draw_text(elems[i].x, elems[i].y, elems[i].text, typeFace, &spriteArray[0], (uint24_RGB*) &elems[i].col, NULL);
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
    
