#include <rom/ets_sys.h>
#include "menus.h"
#include "menu_data.h"
// #include <freetype/ftsizes.h>
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
#include "rgb_fade.h"
#include <dirent.h>
#include "http.h"
#include "system_status.h"
#include "socket.h"
#include "lang.h"
#include "fontfile.h"
#include "esp_heap_trace.h"
#include "menu_functions.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);
// #define MENU_RETURN_FLAG 0x8000
// #define MENU_POP_FLAG 0x4000
// #define MENU_REDRAW_FLAG 0x2000
// #define MENU_SETUP_ONLY_TRANSITION_FLAG 0x1000
// #define MENU_SELF_POP_FLAG 0x800
#define IBUF_SIZE 256

extern QueueHandle_t* button_events;
extern rotary_encoder_info_t* infop;
extern spi_device_handle_t spi;
extern uint24_RGB* background_color;
extern uint24_RGB* foreground_color;
extern SETTINGS_t settings;
static char* ibuf;
static char ibuf_mode;
static uint24_RGB* colorbuf;
extern unsigned char wifi_restart_counter;
extern lua_State* L;
extern uint24_RGB* bgbuf;

uint24_RGB RED = {0xff, 0x00, 0x00};

void setup_cursor(SPRITE_NODE** cursorbg, SPRITE_NODE** cursor, int y);
//     draw_text(10, y, ">", cursor, NULL, *foreground_color, *background_color, 0, false, true);
//     *cursorbg = sprite_rectangle(10, y, 20, 16, background_color, true, 0);
// }

static int menufunc_setup(void) {
    button_event_t event;
    rotary_encoder_event_t rotencev;
    int currlang = 0;
    sprite_rectangle(220, 240-73-13, 100, 22, background_color, true, 0);
    set_font_size(14);
    while(true) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.event == BUTTON_DOWN) {
            if(event.pin == 3) {
                settings.language = currlang;
                delete_persistent_sprites();
                return MENU_SETUP_ONLY_TRANSITION_FLAG | 7;
            }
            if(event.pin == 0) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            currlang = ((unsigned) rotencev.state.position) % 9;
            SPRITE_NODE* sprs[15];
            draw_text(220, 161, text_language_name[currlang], &sprs[0], NULL, *foreground_color, *background_color, 0, false, false);
            draw_all_sprites(spi);
        }
    }
}

static void draw_options(const char** options, SPRITE_NODE* bgrect) {
    int ys[] = {184, 152, 120, 88, 56};
    SPRITE_NODE* sprs[32];

    int name_length = 0;
    for(int i=0;i<5;++i) {
        if(options[i] != NULL) {
            draw_text(0, ys[i], options[i], &sprs[0], &name_length, *foreground_color, *background_color, 0, false, false);
            center_sprite_group_x(sprs, name_length);
        }
        init_sprite(bgrect->v->bitmap, bgrect->v->posX, 224-ys[i], bgrect->v->fg, bgrect->v->bg, bgrect->v->bgcol, bgrect->v->flipX, bgrect->v->flipY, true, false);
    }
    draw_all_sprites(spi);
}

static int menufunc_wifi_scan() {
    SPRITE_NODE* sprs[10];
    int numsprs;

    int ys[] = {184, 152, 120, 88, 56};
    int error;

    const char* list_options[5];
    uint8_t selection = 0;
    uint8_t page_start = 0;
    uint16_t aprecnum = 10;
    wifi_ap_record_t ap_info[10];
    uint16_t ap_count = 0;
    memset(ap_info, 0, sizeof(ap_info));
    set_font_size(14);

    draw_text(270, 2, text_search[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(sprs, numsprs, 2);
    draw_all_sprites(spi);

    SPRITE_NODE* textbg = sprite_rectangle(50, 184, 220, 21, background_color, true, 0);
    textbg->v->draw = false;
    ets_printf("textbg init'd @ %x\n", textbg);
    SPRITE_NODE* cursorbg; // = sprite_rectangle(10, 184, 20, 16, background_color, 0);
    SPRITE_NODE* cursor;
    button_event_t event;
    rotary_encoder_event_t rotencev;

    setup_cursor(&cursorbg, &cursor, 184);
    cursor->v->draw = false;
    cursorbg->v->draw = false;
    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));

refresh:
    list_options[0] = text_searching[settings.language];
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

    cursor->v->draw = true;
    cursorbg->v->draw = true;
    draw_options(list_options, textbg);
    draw_sprites(spi, &cursorbg, 1);
    draw_sprites(spi, &cursor, 1);
    if(ap_count > 10)
        ap_count = 10;

    selection = 0;
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = 240-ys[selection - page_start]-14;
            selection = (rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? (selection + 1) % ap_count : (selection + ap_count - 1) % ap_count;
            if(selection < page_start) {
                page_start = selection;
            } else if (selection > page_start + 4) {
                page_start = selection - 4;
            }
            cursor->v->posY = 240-ys[selection - page_start]-14;
            if(selection > page_start + 4 || selection < page_start) {
                for(int i=0;i<5;++i)
                    list_options[i] = (i < aprecnum) ? (char*)ap_info[i+page_start].ssid : NULL; // should never be null.
                draw_options(list_options, textbg);
            } else {
                draw_all_sprites(spi);
            }
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.event == BUTTON_DOWN) {
            if(event.pin == 3) {
                cursorbg->v->posY = 240-ys[selection]-14;
                draw_sprites(spi, &cursorbg, 1);
                cursorbg->v->draw = false;
                cursor->v->draw = false;
                textbg->v->draw = true;
                for(int i=0;i<5;++i) {
                    textbg->v->posY = 224-ys[i];
                    draw_sprites(spi, &textbg, 1);
                }
                goto refresh;
            }
            if(event.pin == 18) {
                strncpy(&settings.wifi_name[0], (char*)ap_info[selection].ssid, 32);
                delete_persistent_sprites();
                return 6;
            }
            if(event.pin == 0) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

unsigned char table1[] = "Z␣⌫✓ABCDEFGHIJKLMNOPQRSTUVWXYZ␣⌫✓ABCD";
unsigned char table2[] = "z␣⌫✓abcdefghijklmnopqrstuvwxyz␣⌫✓abcd";
unsigned char table3[] = "?@␣⌫✓!\"#$%&'()*+,-./0123456789:;<=>?@␣⌫✓!\"#$%";
unsigned char* metatable[] = {table1, table2, table3};
unsigned char string_lengths[] = {29, 29, 36};
char tablename[][2] = {"a", "@", "A"};
int xs[] = {7, 42, 77, 112, 147, 182, 217, 252, 287};
static void draw_textreel(unsigned int curtable, unsigned int selection, unsigned char** loc) {
    unsigned char visibleBuffer[27];
    *loc = &metatable[curtable][utf8substrlen((char*)metatable[curtable], selection)];
    SPRITE_NODE* sprs[9];
    int substrend = utf8substrlen((char*) *loc, 9);
    for(int i=0;i<substrend;++i) {
        visibleBuffer[i] = (*loc)[i];
    }
    visibleBuffer[substrend] = 0;
    draw_text(0, 33, (char*) visibleBuffer, sprs, NULL, *foreground_color, *background_color, 0, false, false);
    for(int i=0;i<9;++i) {
        sprs[i]->v->posX = xs[i];
    }
    for(int i=0;i<9;++i)
        sprite_rectangle(xs[i], 26, 26, 27, background_color, false, 0);
    draw_all_sprites(spi);
}

static int menufunc_text_write(void) {
    if(ibuf == NULL) {
        return MENU_POP_FLAG;
    }
    button_event_t event;
    rotary_encoder_event_t rotencev;
    unsigned char* selectedchar;
    unsigned char* loc;
    int error;
    uint8_t cursor = strlen(ibuf);
    uint8_t numtyped = utf8strlen(ibuf);
    unsigned int selection = 0;
    unsigned int curtable = 0;
    set_font_size(18);
    draw_text(0, 184, ibuf, NULL, NULL, *foreground_color, *background_color, 24, false, false);
    draw_text(300, 2, "a", NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(147, 2, "▵", NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_textreel(curtable, selection, &loc);

    while(true) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                curtable = (curtable + 1) % 3;
                draw_text(300, 5, tablename[curtable], NULL, NULL, *foreground_color, *background_color, 0, false, false);
                sprite_rectangle(300, 0, 20, 22, background_color, false, 0);
                draw_textreel(curtable, selection, &loc);
            }
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                selectedchar = loc + utf8substrlen((char*) loc, 4);
                if(selectedchar[0] == 0xe2 &&
                    selectedchar[1] == 0x8c &&
                    selectedchar[2] == 0xab) {
                    if(numtyped > 0) {
                        utf8bspc((char*) ibuf, &cursor);
                        numtyped--;
                    }
                } else if(selectedchar[0] == 0xe2 &&
                        selectedchar[1] == 0x9c &&
                        selectedchar[2] == 0x93 && numtyped < 128) {
                    break;
                } else if(selectedchar[0] == 0xe2 &&
                        selectedchar[1] == 0x90 &&
                        selectedchar[2] == 0xa3 && numtyped < 128) {
                    ibuf[cursor] = ' ';
                    cursor++;
                    numtyped++;
                } else if (numtyped < 128){
                    utf8cpychr(&ibuf[cursor], (char*) selectedchar, &cursor);
                    numtyped++;
                }
                SPRITE_NODE* r = sprite_rectangle(0, 191, 320, 16, background_color, true, 0);
                init_sprite(r->v->bitmap, 0, 240-191, r->v->fg, r->v->bg, false, false, false, true, false);
                init_sprite(r->v->bitmap, 0, 240-191+16, r->v->fg, r->v->bg, false, false, false, true, false);
                init_sprite(r->v->bitmap, 0, 240-191+32, r->v->fg, r->v->bg, false, false, false, true, false);
                init_sprite(r->v->bitmap, 0, 240-191+48, r->v->fg, r->v->bg, false, false, false, true, false);
                init_sprite(r->v->bitmap, 0, 240-191+64, r->v->fg, r->v->bg, false, false, false, true, false);
                init_sprite(r->v->bitmap, 0, 240-191+80, r->v->fg, r->v->bg, false, false, false, true, false);
                init_sprite(r->v->bitmap, 0, 240-191+96, r->v->fg, r->v->bg, false, false, false, true, false);
                init_sprite(r->v->bitmap, 0, 240-191+112, r->v->fg, r->v->bg, false, false, false, true, false);
                init_sprite(r->v->bitmap, 0, 240-191+128, r->v->fg, r->v->bg, false, false, false, true, false);
                draw_text(0, 184, ibuf, NULL, NULL, *foreground_color, *background_color, 24, false, false);
                draw_all_sprites(spi);
            } 
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                free(ibuf);
                ibuf = NULL;
                break;
            }
        }
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
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
    int counter = 200;
    int currlang = 0;
    SPRITE_NODE* sprs1[15];
    SPRITE_NODE* sprs2[15];
    SPRITE_NODE* sprs3[45];
    int numsprs1, numsprs2, numsprs3;
    set_font_size(24);
    sprite_rectangle(10, 168, 300, 22, background_color, true, 255);
    sprite_rectangle(10, 146, 300, 22, background_color, true, 255);
    sprite_rectangle(10, 210, 300, 20, background_color, true, 255);
    sprite_rectangle(10, 190, 300, 20, background_color, true, 255);
    sprite_rectangle(10, 5, 300, 20, background_color, true, 255);
    while(true) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18) {
            delete_persistent_sprites();
            return MENU_SETUP_ONLY_TRANSITION_FLAG | 1;
        }
        counter--;
        if(counter == 0) {
            currlang = (currlang+1) % 9;
            draw_text(60, 195, text_welcome[currlang], sprs1, &numsprs1, *foreground_color, *background_color, 0, true, false);
            draw_text(60, 154, text_welcome_a[currlang], sprs2, &numsprs2, *foreground_color, *background_color, 0, true, false);
            set_font_size(14);
            draw_text(60, 10, text_pressenc[currlang], sprs3, &numsprs3, *foreground_color, *background_color, 0, true, false);
            set_font_size(24);
            center_sprite_group_x(sprs1, numsprs1);
            center_sprite_group_x(sprs2, numsprs2);
            center_sprite_group_x(sprs3, numsprs3);
            draw_all_sprites(spi);
            counter = 200;
        }
    }
}

static int menufunc_connect_wifi(void) {
    button_event_t event;
    strncpy((char*)sta_wifi_config.sta.ssid, (char*)&settings.wifi_name[0], 32);
    strncpy((char*)sta_wifi_config.sta.password, (char*)&settings.wifi_pass[0], 64);
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));
    ESP_ERROR_CHECK(esp_wifi_connect());
    ets_printf("connecting...\n");
    while((system_flags & (FLAG_WIFI_CONNECTED | FLAG_WIFI_TIMED_OUT)) == 0) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 0 && event.event == 0) {
            wifi_restart_counter = 10;
            return MENU_POP_FLAG;
        }
    }
    set_font_size(14);
    if(system_flags & FLAG_WIFI_TIMED_OUT) {
        ets_printf("FAILED!\n");
        draw_text(32, 60, text_connection_fail[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
        draw_all_sprites(spi);
        vTaskDelay(3000 / portTICK_PERIOD_MS);
        system_flags &= ~FLAG_WIFI_TIMED_OUT;
        return MENU_POP_FLAG;
    }
    ets_printf("success!\n");
    draw_text(32, 60, text_connected[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_all_sprites(spi);
    vTaskDelay(3000 / portTICK_PERIOD_MS);
    return MENU_SETUP_ONLY_TRANSITION_FLAG | MENU_SELF_POP_FLAG | 8;
}

static int menufunc_http_setup(void) {
    system_flags &= ~FLAG_HTTP_SERVER_DONE;
    ESP_ERROR_CHECK(example_start_file_server("/mainfs"));
    button_event_t event;
    while((system_flags & FLAG_HTTP_SERVER_DONE) == 0) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 0 && event.event == BUTTON_DOWN) {
            stop_file_server();
            return MENU_POP_FLAG;
        }
    }
    system_flags &= ~FLAG_HTTP_SERVER_DONE;
    stop_file_server();
    ledc_set_duty(LEDC_LOW_SPEED_MODE, 7, settings.disp_brightness << 6);
    ledc_update_duty(LEDC_LOW_SPEED_MODE, 7);
    return MENU_SETUP_ONLY_TRANSITION_FLAG | 21;
}

static int menufunc_network_preview(void) {
    int ys[] = {184, 152, 120, 88, 56};
    char namebuf[14];
    SPRITE_NODE* k[15];
    int lenk;
    button_event_t event;
    rotary_encoder_event_t rotencev;

    if(ibuf != NULL) {
        strncpy(settings.wifi_pass, ibuf, 64);
        free(ibuf);
        ibuf = NULL;
    }
    set_font_size(14);

    draw_text(32, 184, text_settings_network[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 152, text_password[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 120, (system_flags & FLAG_WIFI_CONNECTED) ? text_disconnect[settings.language] : text_connect[settings.language], k, &lenk, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(k, lenk);
    if(strlen(settings.wifi_name) > 10) {
        for(int i=0;i<10;++i) {
            namebuf[i] = settings.wifi_name[i];
        }
        namebuf[11] = '.';
        namebuf[12] = '.';
        namebuf[13] = '.';
        namebuf[14] = 0;
        draw_text(150, 184, namebuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    } else {
        draw_text(150, 184, &settings.wifi_name[0], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    }
    if(strlen(settings.wifi_pass) > 10) {
        for(int i=0;i<10;++i) {
            namebuf[i] = settings.wifi_pass[i];
        }
        namebuf[11] = '.';
        namebuf[12] = '.';
        namebuf[13] = '.';
        namebuf[14] = 0;
        draw_text(150, 152, namebuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    } else {
        draw_text(150, 152, &settings.wifi_pass[0], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    }
    draw_all_sprites(spi);
    SPRITE_NODE* cursorbg;
    SPRITE_NODE* cursor;
    int selection = 0;
    setup_cursor(&cursorbg, &cursor, 184);
    draw_all_sprites(spi);
    draw_sprites(spi, &cursor, 1);

    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = 240-ys[selection]-14;
            selection = (rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? (selection + 1) % 3 : (selection + 2) % 3;
            cursor->v->posY = 240-ys[selection]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_SETUP_ONLY_TRANSITION_FLAG | 22;
            }
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                switch(selection) {
                case 0:
                    memset(&settings.wifi_pass[0], 0, 64);
                    delete_persistent_sprites();
                    return MENU_POP_FLAG;
                case 1:
                    ibuf = calloc(256, sizeof(char));
                    strcpy(ibuf, &settings.wifi_pass[0]);
                    delete_persistent_sprites();
                    return 3;
                case 2:
                    delete_persistent_sprites();
                    if(system_flags & FLAG_WIFI_CONNECTED) {
                        memset(&settings.wifi_name, 0, 32);
                        memset(&settings.wifi_pass, 0, 64);
                        wifi_restart_counter = 20;
                        esp_wifi_disconnect();
                        system_flags &= ~(FLAG_WIFI_CONNECTED | FLAG_WIFI_TIMED_OUT);
                        return MENU_POP_FLAG;
                    }
                    return 4;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                memset(&settings.wifi_pass[0], 0, 64);
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

static int menufunc_pb_setup_method (void) {
    button_event_t event;
    rotary_encoder_event_t rotencev;
    const char* options_1 = text_tooltip_wifi_setup[settings.language];
    const char* options_2 = text_tooltip_wifi_setup_a[settings.language];
    const char* options_3 = text_tooltip_standalone_setup[settings.language];
    const char* options_4 = text_tooltip_standalone_setup_a[settings.language];
    unsigned char selection = 0;
    int lentt;
    int lenttline2;
    int lentt1;
    int lentt2;
    SPRITE_NODE* tooltip_1[60];
    SPRITE_NODE* tooltip_2[60];
    set_font_size(12);
    draw_text(0, 52, options_1, tooltip_1, &lentt, *foreground_color, *background_color, 0, false, true);
    center_sprite_group_x(tooltip_1, lentt);
    draw_text(0, 34, options_2, tooltip_1 + lentt, &lenttline2, *foreground_color, *background_color, 0, false, true);
    center_sprite_group_x(tooltip_1+lentt, lenttline2);
    lentt1 = lentt+lenttline2;
    draw_text(0, 52, options_3, tooltip_2, &lentt, *foreground_color, *background_color, 0, false, true);
    center_sprite_group_x(tooltip_2, lentt);
    draw_text(0, 34, options_4, tooltip_2+lentt, &lenttline2, *foreground_color, *background_color, 0, false, true);
    center_sprite_group_x(tooltip_2+lentt, lenttline2);
    for(int i=0;i<lentt+lenttline2;++i) {
        tooltip_2[i]->v->draw = false;
    }
    lentt2 = lentt+lenttline2;
    SPRITE_NODE* cursorbg; // = sprite_rectangle(10, 120, 20, 16, background_color, 0);
    SPRITE_NODE* cursor;
    set_font_size(14);
    setup_cursor(&cursorbg, &cursor, 137);
    draw_text(10, 137, ">", NULL, NULL, *foreground_color, *background_color, 0, false, false);
    sprite_rectangle(0, 25, 320, 16, background_color, true, 0);
    sprite_rectangle(0, 41, 320, 16, background_color, true, 0);
    sprite_rectangle(0, 57, 320, 16, background_color, true, 0);
    draw_all_sprites(spi);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = selection ? 240-105-14 : 240-137-14;
            selection = !selection;
            cursor->v->posY = selection ? 240-105-14 : 240-137-14;
            for(int i=0;i<lentt2;++i) {
                tooltip_2[i]->v->draw = selection;
            }
            for(int i=0;i<lentt1;++i) {
                tooltip_1[i]->v->draw = !selection;
            }
            draw_all_sprites(spi);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if((event.pin == 3 || event.pin == 18) && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_SETUP_ONLY_TRANSITION_FLAG | (selection ? 2 : 5);
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

// const char* const* theme_names[] = {text_dark_mode, text_light_mode, text_custom};
extern const char* const* theme_names[3];
static int menufunc_display_settings(void) {
    button_event_t event;
    rotary_encoder_event_t rotencev;
    uint24_RGB hicolor = {0xff, 0x00, 0x00};
    if(colorbuf != NULL) {
        settings.custom_theme_color.pixelR = colorbuf->pixelR;
        settings.custom_theme_color.pixelG = colorbuf->pixelG;
        settings.custom_theme_color.pixelB = colorbuf->pixelB;
        free(colorbuf);
        colorbuf = NULL;
        assign_theme_from_settings();
        delete_persistent_sprites();
        load_bgimg(bgbuf, "/mainfs/pb_bg.cbi", true, 0);
        blit_bg();
        return MENU_REDRAW_FLAG;
    }
    int numsprs;
    SPRITE_NODE* sprs[32];
    char bright[4];
    (void) itoa(settings.disp_brightness, bright, 10);
    int mode = 0;
    unsigned char selection = 0;
    set_font_size(14);
    draw_text(0, 216, text_display_setting[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, text_brightness[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(150, 184, bright, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(150, 152, theme_names[settings.disp_theme][settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 152, text_theme[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    sprite_rectangle(20, 211, 280, 21, background_color, false, 0);
    draw_all_sprites(spi);
    SPRITE_NODE* cursorbg; // = sprite_rectangle(10, 184, 20, 16, background_color, 0);
    SPRITE_NODE* cursor;
    volatile SPRITE_NODE* bright_rec = sprite_rectangle(150, 184, 150, 16, background_color, true, 0);
    volatile SPRITE_NODE* theme_rec = sprite_rectangle(150, 147, 100, 25, background_color, true, 0);
    bright_rec->v->draw = false;
    theme_rec->v->draw = false;
    setup_cursor(&cursorbg, &cursor, 184);
    draw_sprites(spi, &cursor, 1);
    draw_all_sprites(spi);
    int num_brspr;
    SPRITE_NODE* br_sprite[4]; 
    int num_themespr;
    SPRITE_NODE* theme_sprite[40];
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            switch(mode) {
            case 0:
                cursorbg->v->posY = selection ? 240-152-14 : 240-184-14;
                selection = !selection;
                cursor->v->posY = selection ? 240-152-14 : 240-184-14;
                draw_all_sprites(spi);
                break;
            case 1:
                settings.disp_brightness += rotencev.state.multiplier * ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : -1);
                if(settings.disp_brightness > 255) {
                    settings.disp_brightness = 255;
                } else if(settings.disp_brightness < 0) {
                    settings.disp_brightness = 0;
                }
                (void) itoa(settings.disp_brightness, bright, 10);
                draw_text(150, 184, bright, br_sprite, &num_brspr, hicolor, *background_color, 0, false, false);
                draw_all_sprites(spi);
                ledc_set_duty(LEDC_LOW_SPEED_MODE, 7, settings.disp_brightness << 6);
                ledc_update_duty(LEDC_LOW_SPEED_MODE, 7);
                break;
            case 2:
                settings.disp_theme = (settings.disp_theme + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 2)) % 3;
                draw_text(150, 152, theme_names[settings.disp_theme][settings.language], theme_sprite, &num_themespr, hicolor, *background_color, 0, false, false);
                draw_all_sprites(spi);
                break;
            default:
            }
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                if(mode == 2) {
                    // start the colour menu!
                    if(settings.disp_theme == 2) {
                        colorbuf = calloc(1, 3);
                        colorbuf->pixelR = settings.custom_theme_color.pixelR;
                        colorbuf->pixelG = settings.custom_theme_color.pixelG;
                        colorbuf->pixelB = settings.custom_theme_color.pixelB;
                        delete_persistent_sprites();
                        return 9;
                    }
                    assign_theme_from_settings();
                    delete_persistent_sprites();

                    load_bgimg(bgbuf, "/mainfs/pb_bg.cbi", true, 0);
                    blit_bg();
                    return MENU_REDRAW_FLAG;
                }
                mode = mode == 0 ? selection + 1 : 0;
                switch(mode) {
                case 0:
                    draw_text(150, 184, bright, br_sprite, &num_brspr, *foreground_color, *background_color, 0, false, false);
                    draw_text(150, 152, theme_names[settings.disp_theme][settings.language], theme_sprite, &num_themespr, *foreground_color, *background_color, 0, false, false);
                    draw_all_sprites(spi);
                    bright_rec->v->draw = false;
                    theme_rec->v->draw = false;
                    break;
                case 1:
                    draw_text(150, 184, bright, br_sprite, &num_brspr, RED, *background_color, 0, false, false);
                    draw_all_sprites(spi);
                    bright_rec->v->draw = true;
                    break;
                case 2:
                    draw_text(150, 152, theme_names[settings.disp_theme][settings.language], theme_sprite, &num_themespr, RED, *background_color, 0, false, false);
                    draw_all_sprites(spi);
                    theme_rec->v->draw = true;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_SETUP_ONLY_TRANSITION_FLAG | 10;
            }
        }
    }
    return MENU_RETURN_FLAG;
}

static int menufunc_color_picker(void) {
    const int ys[] = {184, 152, 120};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    if(colorbuf == NULL)
        return MENU_POP_FLAG;
    set_font_size(14);
    int numsprs;
    SPRITE_NODE* sprs[20];
    int selection = 0;
    int mode = 0;
    unsigned char buffer[3] = {colorbuf->pixelR, colorbuf->pixelG, colorbuf->pixelB};
    char numbuf[3];
    draw_text(0, 216, text_color_picker[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, text_red[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 152, text_green[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 120, text_blue[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    (void) itoa(buffer[0], numbuf, 10);
    draw_text(150, 184, numbuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    (void) itoa(buffer[1], numbuf, 10);
    draw_text(150, 152, numbuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    (void) itoa(buffer[2], numbuf, 10);
    draw_text(150, 120, numbuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 184, ">", NULL, NULL, *foreground_color, *background_color, 0, false, false);
    sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    draw_all_sprites(spi);
    SPRITE_NODE* cursorbg; // = sprite_rectangle(10, 184, 20, 16, background_color, 0);
    SPRITE_NODE* cursor;
    setup_cursor(&cursorbg, &cursor, 184);
    SPRITE_NODE* red_rec = sprite_rectangle(150, 184, 100, 16, background_color, true, 0);
    SPRITE_NODE* green_rec = sprite_rectangle(150, 152, 100, 16, background_color, true, 0);
    SPRITE_NODE* blue_rec = sprite_rectangle(150, 120, 100, 16, background_color, true, 0);
    red_rec->v->draw = false;
    green_rec->v->draw = false;
    blue_rec->v->draw = false;
    SPRITE_NODE* colorrec;
    colorrec = sprite_rectangle(128, 48, 64, 64, (uint24_RGB*) buffer, true, 255);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            // ets_printf("c:%d r:%d b:%d g:%d\n", colorrec->v->bitmap->w, red_rec->v->bitmap->w, blue_rec->v->bitmap->w, green_rec->v->bitmap->w);
            switch(mode) {
            case 0:
                cursorbg->v->posY = 240-ys[selection]-14;
                selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 2)) % 3;
                cursor->v->posY = 240-ys[selection]-14;
                break;
            default:
                buffer[selection] = (buffer[selection] + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? rotencev.state.multiplier : 256 - rotencev.state.multiplier)) & 0xff;
                (void) itoa(buffer[selection], numbuf, 10);
                draw_text(150, ys[selection], numbuf, sprs, &numsprs, RED, *background_color, 0, false, false);
            }
            colorrec->v->fg = *(uint24_RGB*) buffer;
            draw_all_sprites(spi);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                mode = (mode == 0) ? selection + 1 : 0;
                (void) itoa(buffer[selection], numbuf, 10);
                draw_text(150, ys[selection], numbuf, sprs, &numsprs, (mode == 0) ? *foreground_color : RED, *background_color, 0, false, false);
                draw_all_sprites(spi);
                switch(mode) {
                case 0:
                    red_rec->v->draw = false;
                    green_rec->v->draw = false;
                    blue_rec->v->draw = false;
                    break;
                case 1:
                    red_rec->v->draw = true;
                    break;
                case 2:
                    green_rec->v->draw = true;
                    break;
                case 3:
                    blue_rec->v->draw = true;
                }
            }
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                colorbuf->pixelR = buffer[0];
                colorbuf->pixelG = buffer[1];
                colorbuf->pixelB = buffer[2];
                return MENU_POP_FLAG;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

char kpa_name[] = "kPa";
char bar_name[] = "bar";
char mmhg_name[] = "mmHg";
char psi_name[] = "PSI";
char* pressure_names[] = {kpa_name, bar_name, psi_name, mmhg_name};
static int menufunc_add_on_settings(void) {
    button_event_t event;
    rotary_encoder_event_t rotencev;
    int numsprs;
    SPRITE_NODE* sprs[32];
    set_font_size(14);
    draw_text(0, 216, text_addon_settings[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 184, text_mprls[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(190, 184, pressure_names[settings.pressure_units], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    draw_all_sprites(spi);
    (void) sprite_rectangle(190, 178, 100, 28, background_color, true, 0);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            settings.pressure_units = (settings.pressure_units + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : -1)) % 4;
            draw_text(190, 184, pressure_names[settings.pressure_units], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
            draw_all_sprites(spi);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_SETUP_ONLY_TRANSITION_FLAG | 21;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

const int selection_to_menu[] = {8, 19, 12, 14, 10, 15, 1, 23};
static int menufunc_all_settings(void) {
    int ys[] = {184, 152, 120, 88, 56};
    const char* settings_options_list[] = {
        text_settings_display[settings.language],
        text_settings_network[settings.language],
        text_settings_output[settings.language],
        text_settings_rgb[settings.language],
        text_settings_add_ons[settings.language],
        text_settings_apps[settings.language],
        text_language[settings.language],
        text_settings_developer[settings.language],
        NULL
    };
    button_event_t event;
    rotary_encoder_event_t rotencev;
    SPRITE_NODE* sprs[32];
    int numsprs;
    SPRITE_NODE* cursorbg;
    SPRITE_NODE* cursor;
    int selection = 0;
    int page_start = 0;
    set_font_size(14);
    setup_cursor(&cursorbg, &cursor, 184);
    draw_text(0, 216, text_settings[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    SPRITE_NODE* textbg = sprite_rectangle(30, 184, 260, 21, background_color, true, 0);
    SPRITE_NODE* titlebg = sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    draw_options((const char**)settings_options_list, textbg);
    draw_sprites(spi, &cursor, 1);
    textbg->v->draw = false;
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = 240-ys[selection - page_start]-14;
            selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 7)) % 8;
            if(selection > page_start + 4) {
                page_start = selection - 4;
            } else if(selection < page_start) {
                page_start = selection;
            }
            cursor->v->posY = 240-ys[selection - page_start]-14;
            if((selection-page_start == 0) || (selection-page_start == 4)) {
                draw_options((const char**) (&settings_options_list[page_start]), textbg);
            } else {
                draw_all_sprites(spi);
            }
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return selection_to_menu[selection];
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                write_to_file(&settings);
                assign_theme_from_settings();
                return MENU_RETURN_FLAG;
            }
        }
    }
}

int selected_pwm;
static int menufunc_pwm_output_settings(void) {
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    SPRITE_NODE* sprs[32];
    int numsprs;
    SPRITE_NODE* cursorbg;
    SPRITE_NODE* cursor;
    int selection = 0;
    char outputWordBuf[15];
    set_font_size(14);
    draw_text(0, 216, text_output_settings[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    strncpy(outputWordBuf, text_settings_output[settings.language], 15);
    sprintf(outputWordBuf, "%s %d", text_settings_output[settings.language], 0);
    draw_text(32, 184, outputWordBuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    sprintf(outputWordBuf, "%s %d", text_settings_output[settings.language], 1);
    draw_text(32, 152, outputWordBuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    sprintf(outputWordBuf, "%s %d", text_settings_output[settings.language], 2);
    draw_text(32, 120, outputWordBuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    sprintf(outputWordBuf, "%s %d", text_settings_output[settings.language], 3);
    draw_text(32, 88, outputWordBuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 184, ">", &cursor, NULL, *foreground_color, *background_color, 0, false, false);
    sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    draw_all_sprites(spi);
    setup_cursor(&cursorbg, &cursor, 184);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = 240-ys[selection]-14;
            selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 3)) % 4;
            cursor->v->posY = 240-ys[selection]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                selected_pwm = selection;
                return 13;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                write_to_file(&settings);
                return MENU_POP_FLAG;
            }
        }
    }
}

static int menufunc_pwm_output_set(void) {
    uint24_RGB* hicolor = &RED;
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    SPRITE_NODE* sprs[11];
    SPRITE_NODE* minsprs[6];
    int num_minsprs;
    int numsprs;
    SPRITE_NODE* cursorbg;
    SPRITE_NODE* cursor;
    int selection = 0;
    char percentage[5];
    int mode = 0;
    set_font_size(14);
    sprintf(percentage, "%d%%", settings.pwm_min_limit[selected_pwm] / 163);
    draw_text(200, 184, percentage, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    sprintf(percentage, "%d%%", settings.pwm_max_limit[selected_pwm] / 163);
    draw_text(200, 152, percentage, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(0, 216, text_output_settings[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, text_lowest_value[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 152, text_highest_value[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 120, text_pwm_wizard[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 88, text_output_mode[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(200, 88, settings.output_set_on_off_only[selected_pwm] ? text_digital[settings.language] : text_analog[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 184, ">", &cursor, NULL, *foreground_color, *background_color, 0, false, false);
    sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    draw_all_sprites(spi);
    SPRITE_NODE* textbg = sprite_rectangle(200, 184, 120, 21, background_color, true, 0);
    setup_cursor(&cursorbg, &cursor, 184);
    textbg->v->draw = false;
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            switch(mode) {
            case 0:
                cursorbg->v->posY = 240-ys[selection]-14;
                selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 3)) % 4;
                cursor->v->posY = 240-ys[selection]-14;
                draw_sprites(spi, &cursorbg, 1);
                draw_sprites(spi, &cursor, 1);
                break;
            case 1:
                if(rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) {
                    if(settings.pwm_min_limit[selected_pwm] + (163 * rotencev.state.multiplier) < 0x3fff) {
                        settings.pwm_min_limit[selected_pwm] += 163 * rotencev.state.multiplier;
                    } else {
                        settings.pwm_min_limit[selected_pwm] = 0x3fff;
                    }
                } else {
                    if(settings.pwm_min_limit[selected_pwm] > (163 * rotencev.state.multiplier))
                        settings.pwm_min_limit[selected_pwm] -= (163 * rotencev.state.multiplier);
                    else
                        settings.pwm_min_limit[selected_pwm] = 0;
                }
                sprintf(percentage, "%d%%", settings.pwm_min_limit[selected_pwm] / 163);
                draw_text(200, 184, percentage, minsprs, &num_minsprs, *hicolor, *background_color, 0, false, false);
                draw_all_sprites(spi);
                break;
            case 2:
                if(rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) {
                    if(settings.pwm_max_limit[selected_pwm] + (163 * rotencev.state.multiplier) < 0x3fff) {
                        settings.pwm_max_limit[selected_pwm] += 163 * rotencev.state.multiplier;
                    } else {
                        settings.pwm_max_limit[selected_pwm] = 0x3fff;
                    }
                } else {
                    if(settings.pwm_max_limit[selected_pwm] > (163 * rotencev.state.multiplier))
                        settings.pwm_max_limit[selected_pwm] -= (163 * rotencev.state.multiplier);
                    else
                        settings.pwm_max_limit[selected_pwm] = 0;
                }
                sprintf(percentage, "%d%%", settings.pwm_max_limit[selected_pwm] / 163);
                draw_text(200, 152, percentage, sprs, &numsprs, *hicolor, *background_color, 0, false, false);
                draw_all_sprites(spi);
            }
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                if(selection < 2) {
                    mode = (mode == 0) ? selection + 1 : 0;
                    switch(mode) {
                    case 0:
                        textbg->v->draw = false;
                        sprintf(percentage, "%d%%", settings.pwm_max_limit[selected_pwm] / 163);
                        draw_text(200, 152, percentage, sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
                        sprintf(percentage, "%d%%", settings.pwm_min_limit[selected_pwm] / 163);
                        draw_text(200, 184, percentage, minsprs, &num_minsprs, *foreground_color, *background_color, 0, false, false);
                        draw_all_sprites(spi);
                        break;
                    case 1:
                        textbg->v->draw = true;
                        textbg->v->posY = 240-21-184;
                        sprintf(percentage, "%d%%", settings.pwm_min_limit[selected_pwm] / 163);
                        draw_text(200, 184, percentage, minsprs, &num_minsprs, *hicolor, *background_color, 0, false, false);
                        draw_all_sprites(spi);
                        break;
                    case 2:
                        textbg->v->draw = true;
                        textbg->v->posY = 240-21-152;
                        sprintf(percentage, "%d%%", settings.pwm_max_limit[selected_pwm] / 163);
                        draw_text(200, 152, percentage, sprs, &numsprs, *hicolor, *background_color, 0, false, false);
                        draw_all_sprites(spi);
                    default:
                        break;
                    }
                } else if (selection == 3) {
                    textbg->v->draw = true;
                    textbg->v->posY = 240-16-88;
                    settings.output_set_on_off_only[selected_pwm] = !settings.output_set_on_off_only[selected_pwm];
                    draw_text(200, 88, settings.output_set_on_off_only[selected_pwm] ? text_digital[settings.language] : text_analog[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
                    draw_all_sprites(spi);
                    textbg->v->draw = false;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

const char* const* RGB_Mode_Names[] = {text_rgb_off, text_solid, text_fade, text_rainbow};
static int menufunc_rgb_lighting(void) {
    rgb_update();
    uint24_RGB* hicolor = &RED;
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    SPRITE_NODE* sprs[11];
    SPRITE_NODE* codesprs[11];
    SPRITE_NODE* speedsprs[11];
    int numsprs;
    int numcodesprs;
    int numspeedsprs;
    SPRITE_NODE* cursorbg;
    SPRITE_NODE* cursor;
    int selection = 0;
    char percentage[5];
    char speed_val[5];
    int mode = 0;
    set_font_size(14);
    sprintf(percentage, "%d%%", settings.RGB_brightness / 163);
    sprintf(speed_val, "%d", settings.RGB_speed);
    draw_text(200, 184, percentage, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(200, 120, speed_val, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(200, 152, RGB_Mode_Names[settings.RGB_mode][settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(0, 216, text_rgb_settings[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, text_brightness[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 152, text_rgb_mode[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 120, text_rgb_speed[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 88, text_color_1[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 56, text_color_2[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 184, ">", &cursor, NULL, *foreground_color, *background_color, 0, false, false);
    sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    sprite_rectangle(200, 80, 30, 30, &settings.RGB_colour, false, 0);
    sprite_rectangle(200, 48, 30, 30, &settings.RGB_colour_2, false, 0);
    draw_all_sprites(spi);
    SPRITE_NODE* textbg = sprite_rectangle(200, 184, 120, 21, background_color, true, 0);
    setup_cursor(&cursorbg, &cursor, 184);
    textbg->v->draw = false;
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            switch(mode) {
            case 0:
                cursorbg->v->posY = 240-ys[selection]-14;
                selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 4)) % 5;
                cursor->v->posY = 240-ys[selection]-14;
                draw_sprites(spi, &cursorbg, 1);
                draw_sprites(spi, &cursor, 1);
                break;
            case 1:
                if(rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) {
                    if(settings.RGB_brightness + (163 * rotencev.state.multiplier) < 0x3fff)
                        settings.RGB_brightness += (163 * rotencev.state.multiplier);
                    else
                        settings.RGB_brightness = 0x3fff;
                } else {
                    if(settings.RGB_brightness > (163 * rotencev.state.multiplier))
                        settings.RGB_brightness -= (163 * rotencev.state.multiplier);
                    else
                        settings.RGB_brightness = 0;
                }
                sprintf(percentage, "%d%%", settings.RGB_brightness / 163);
                draw_text(200, 184, percentage, sprs, &numsprs, *hicolor, *background_color, 0, false, false);
                textbg->v->draw = true;
                draw_all_sprites(spi);
                rgb_update();
                break;
            case 2:
                settings.RGB_mode = (settings.RGB_mode + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 3)) % 4;
                draw_text(200, 152, RGB_Mode_Names[settings.RGB_mode][settings.language], codesprs, &numcodesprs, *hicolor, *background_color, 0, false, false);
                textbg->v->draw = true;
                draw_all_sprites(spi);
                rgb_update();
                break;
            case 3:
                if(rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) {
                    if(settings.RGB_speed + (rotencev.state.multiplier) < 0xff)
                        settings.RGB_speed += rotencev.state.multiplier;
                    else
                        settings.RGB_speed = 0xff;
                } else {
                    if(settings.RGB_speed > rotencev.state.multiplier)
                        settings.RGB_speed -= rotencev.state.multiplier;
                    else
                        settings.RGB_speed = 1;
                }
                sprintf(speed_val, "%d", settings.RGB_speed);
                draw_text(200, 120, speed_val, speedsprs, &numspeedsprs, *hicolor, *background_color, 0, false, false);
                textbg->v->draw = true;
                draw_all_sprites(spi);
                rgb_update();
            }
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                if(selection < 3) {
                    mode = (mode == 0) ? selection + 1 : 0;
                    switch(mode) {
                    case 0:
                        draw_text(200, 184, percentage, sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
                        draw_text(200, 152, RGB_Mode_Names[settings.RGB_mode][settings.language], codesprs, &numcodesprs, *foreground_color, *background_color, 0, false, false);
                        draw_text(200, 120, speed_val, speedsprs, &numspeedsprs, *foreground_color, *background_color, 0, false, false);
                        textbg->v->draw = false;
                        draw_all_sprites(spi);
                        break;
                    case 1:
                        draw_text(200, 184, percentage, sprs, &numsprs, *hicolor, *background_color, 0, false, false);
                        textbg->v->draw = true;
                        textbg->v->posY = 240-21-184;
                        draw_all_sprites(spi);
                        break;
                    case 2:
                        draw_text(200, 152, RGB_Mode_Names[settings.RGB_mode][settings.language], codesprs, &numcodesprs, *hicolor, *background_color, 0, false, false);
                        textbg->v->draw = true;
                        textbg->v->posY = 240-21-152;
                        draw_all_sprites(spi);
                        break;
                    case 3:
                        draw_text(200, 120, speed_val, speedsprs, &numspeedsprs, *hicolor, *background_color, 0, false, false);
                        textbg->v->draw = true;
                        textbg->v->posY = 240-21-120;
                        draw_all_sprites(spi);
                        break;
                    }
                } else {
                    colorbuf = (selection == 3) ? &settings.RGB_colour : &settings.RGB_colour_2;
                    delete_persistent_sprites();
                    return 9;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

static int menufunc_applications(void) {
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    SPRITE_NODE* sprs[40];
    SPRITE_NODE* downloadsprs[40];
    int numdownloadsprs;
    int numsprs;
    SPRITE_NODE* cursor;
    int selection = 0;
    int page_start = 0;
    char* names[64];
    int num_names = 0;
    DIR* d = opendir("/mainfs/applications");
    struct dirent* dir;
    int i=0;
    if(d) {
        while((dir = readdir(d)) != NULL && i < 64) {
            names[i] = calloc(strlen(dir->d_name) + 1, sizeof(char));
            strncpy(names[i], dir->d_name, strlen(dir->d_name));
            num_names++;
            i++;
        }
        closedir(d);
    }
    if(i < 63)
        names[i+1] = NULL;
    SPRITE_NODE* textbg = sprite_rectangle(50, 184, 220, 21, background_color, true, 0);
    SPRITE_NODE* cursorbg; // = sprite_rectangle(10, 184, 20, 16, background_color, 0);
    set_font_size(14);
    setup_cursor(&cursorbg, &cursor, 184);
    draw_text(0, 216, text_applications[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_text(2, 2, text_app_download[settings.language], downloadsprs, &numdownloadsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(downloadsprs, numdownloadsprs, 2);
    SPRITE_NODE* titlebg = sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    draw_options((const char**)names, textbg);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = 240-ys[selection - page_start]-14;
            selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : num_names - 1)) % num_names;
            if(selection > page_start + 4) {
                page_start = selection - 4;
            } else if(selection < page_start) {
                page_start = selection;
            }
            if((selection-page_start == 0) || (selection-page_start == 4)) {
                draw_options((const char**) (&names[page_start]), textbg);
            }
            cursor->v->posY = 240-ys[selection - page_start]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                for(int i=0;i<num_names;++i)
                    if(i != selection)
                        free(names[i]);
                ibuf = names[selection];
                return 16;
            }
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return 18;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                for(int i=0;i<num_names;++i)
                    free(names[i]);
                return MENU_POP_FLAG;
            }
        }
    }
}

int menufunc_file_run_delete() {
    if(!ibuf)
        return MENU_POP_FLAG;
    int ys[] = {120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    SPRITE_NODE* cursorbg;
    SPRITE_NODE* cursor;
    int k;
    char* c;
    int selection = 0;
    set_font_size(18);
    draw_text(10, 156, ibuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_all_sprites(spi);
    set_font_size(14);
    setup_cursor(&cursorbg, &cursor, 120);
    draw_all_sprites(spi);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = 240-ys[selection]-14;
            selection = ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? selection + 1 : selection + 2) % 3;
            cursor->v->posY = 240-ys[selection]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                switch(selection) {
                case 0:
                    delete_persistent_sprites();
                    k = strlen(ibuf);
                    c = calloc(k+22, sizeof(char));
                    strcpy(c, "/mainfs/applications/");
                    strncpy(c+21, ibuf, k+1);
                    free(ibuf);
                    ibuf = c;
                    return 17;
                case 1:
                    delete_persistent_sprites();
                    k = strlen(ibuf);
                    c = calloc(k+22, sizeof(char));
                    strcpy(c, "/mainfs/applications/");
                    strncpy(c+21, ibuf, k+1);
                    if(remove(c))
                        ets_printf("error removing file %s!\n");
                    free(c);
                    free(ibuf);
                    ibuf = NULL;
                    return MENU_POP_FLAG;
                case 2:
                    k = strlen(ibuf);
                    c = calloc(k+22, sizeof(char));
                    strcpy(c, "/mainfs/applications/");
                    strncpy(c+21, ibuf, k+1);
                    set_default_app(c);
                    free(c);
                default:
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                free(ibuf);
                ibuf = NULL;
                return MENU_POP_FLAG;
            }
        }
    }
}

int menufunc_execute_ibuf_file(void) {
    (void) luaL_dofile(L, ibuf);
    free(ibuf);
    ibuf = NULL;
    return MENU_POP_FLAG;
}

void parse_to_url(char* x) {
    if(strncmp(x, "https://", 8)) {
        char* newurl = calloc(290, sizeof(char));
        sprintf(newurl, "https://raw.githubusercontent.com/%s", ibuf);
        free(ibuf);
        ibuf = newurl;
    }
}

int menufunc_download_file(void) {
    if(!ibuf) {
        ibuf = calloc(256, sizeof(char));
        strcpy(ibuf, "https://");
    }
    button_event_t event;
    char displayName[16];
    int k = strlen(ibuf);
    SPRITE_NODE* sprs[14];
    int numsprs;
    int i;
    char* filename;
    if(k > 12) {
        strcpy(displayName, "...");
        strncpy(displayName+3, ibuf+k-12, 12);
    } else {
        strncpy(displayName, ibuf, 15);
    }
    displayName[15]=0;
    set_font_size(14);
    draw_text(100, 152, displayName, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_all_sprites(spi);
    while(true) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                draw_text(0, 120, text_app_downloading[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
                draw_all_sprites(spi);
                parse_to_url(ibuf);
                k = strlen(ibuf);
                for(i=k-1;ibuf[i]!='/';--i);
                filename = malloc(k-i+22);
                strcpy(filename, "/mainfs/applications");
                strcpy(filename+20, &ibuf[i]);
                ets_printf("downloading %s to %s\n", ibuf, filename);
                http_wget(ibuf, filename);
                free(filename);
                free(ibuf);
                ibuf = NULL;
                return MENU_POP_FLAG;
            }
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return 3;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                free(ibuf);
                ibuf = NULL;
                return MENU_POP_FLAG;
            }
        }
    }
}

static int menufunc_network_settings(void) {
    int ys[] = {120, 88, 56};
    SPRITE_NODE* cursorbg;
    SPRITE_NODE* cursor;
    button_event_t event;
    rotary_encoder_event_t rotencev;
    char namebuf[15];
    int selection = 0;
    set_font_size(14);
    if(system_flags & FLAG_WIFI_CONNECTED) {
        if(strlen(settings.wifi_name) > 10) {
            for(int i=0;i<10;++i) {
                namebuf[i] = settings.wifi_name[i];
            }
            namebuf[11] = '.';
            namebuf[12] = '.';
            namebuf[13] = '.';
            namebuf[14] = 0;
            draw_text(190, 120, namebuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
        } else {
            draw_text(190, 120, &settings.wifi_name[0], NULL, NULL, *foreground_color, *background_color, 0, false, false);
        }
    } else {
        draw_text(190, 120, text_disconnected[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    }
    draw_all_sprites(spi);
    setup_cursor(&cursorbg, &cursor, 120);
    draw_all_sprites(spi);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = 240-ys[selection]-14;
            selection = !selection;
            cursor->v->posY = 240-ys[selection]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return selection ? 20 : ((system_flags & FLAG_WIFI_CONNECTED) ? 6 : 2);
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

static int menufunc_server_settings(void) {
    set_font_size(14);
    if(ibuf != NULL) {
        switch(ibuf_mode) {
        case 1:
            uint8_t temp[4];
            int numvars = sscanf(ibuf, "%hhu.%hhu.%hhu.%hhu", &temp[0], &temp[1], &temp[2], &temp[3]);
            if(numvars == 4)
                for(int i=0;i<4;++i)
                    settings.server_ip[i] = temp[i];
            break;
        case 2:
            strncpy(settings.server_password, ibuf, 64);
            settings.server_password[63] = 0;
            break;
        case 3:
            unsigned int m;
            sscanf(ibuf, "%u", &m);
            if(m <= 0xffff)
                settings.server_port = m;
            break;
        default:
        }
        ibuf_mode = 0;
        free(ibuf);
        ibuf = NULL;
    }
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    SPRITE_NODE* cursorbg;
    SPRITE_NODE* cursor;
    int selection = 0;
    char ip_buf[16];
    char pass_buf[16];
    char port_buf[6];
    SPRITE_NODE* sprs[32];
    int numsprs;
    SPRITE_NODE* rect1;
    sprintf(ip_buf, "%d.%d.%d.%d", settings.server_ip[0], settings.server_ip[1], settings.server_ip[2], settings.server_ip[3]);
    draw_text(150, 184, ip_buf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    int k = strlen(settings.server_password);
    if(k > 12) {
        strncpy(pass_buf, settings.server_password+k-12, 12);
        strcpy(pass_buf+12, "...");
    } else {
        strncpy(pass_buf, settings.server_password, 15);
    }
    draw_text(150, 152, pass_buf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    itoa(settings.server_port, port_buf, 10);
    port_buf[5] = 0;
    draw_text(150, 120, port_buf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(150, 88, 
            ((system_flags & FLAG_WIFI_CONNECTED) ? 
                ((system_flags & FLAG_SERVER_CONNECTED) ? 
                    text_disconnect 
                    : text_connect) 
                : text_cant_connect)[settings.language],
            sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_all_sprites(spi);
    setup_cursor(&cursorbg, &cursor, 184);
    draw_all_sprites(spi);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            cursorbg->v->posY = 240-ys[selection]-14;
            selection = ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? selection+1 : selection+3) % 4;
            cursor->v->posY = 240-ys[selection]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                if(selection == 3) {
                    if(system_flags & FLAG_WIFI_CONNECTED) {
                        if(system_flags & FLAG_SERVER_CONNECTED) {
                            disconnect_from_server();
                            system_flags &= ~FLAG_SERVER_CONNECTED;
                        } else {
                            int r = connect_to_server(*(uint32_t*) &settings.server_ip, settings.server_port);
                            if(r) {
                                draw_text(32, 56, text_connection_fail[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
                                draw_all_sprites(spi);
                                goto failed;
                            }
                            system_flags |= FLAG_SERVER_CONNECTED;
                        }
                        rect1 = sprite_rectangle(32, 88, 176, 20, background_color, false, 0);
                        draw_text(150, 88, (system_flags & FLAG_SERVER_CONNECTED) ? text_disconnect[settings.language] : text_connect[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
                        center_sprite_group_x(sprs, numsprs);
                        draw_all_sprites(spi);
                    }
failed:
                } else {
                    delete_persistent_sprites();
                    ibuf_mode = selection+1;
                    ibuf = calloc(256, sizeof(char));
                    return 3;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

static int menufunc_setup_done(void) {
    button_event_t event;
    while(true) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.event == BUTTON_DOWN && event.pin == 18) {
            return MENU_RETURN_FLAG;
        }
    }
}

static int menufunc_skip_wifi(void) {
    button_event_t event;
    while(true) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.event == BUTTON_DOWN) {
            if(event.pin == 0)
                return MENU_POP_FLAG;
            if(event.pin == 3)
                return MENU_SETUP_ONLY_TRANSITION_FLAG | MENU_SELF_POP_FLAG | 8;
        }
    }
}

static int menufunc_developer(void) {
    SPRITE_NODE* cursor;
    SPRITE_NODE* cursorbg;
    int numsprs;
    SPRITE_NODE* sprs[50];
    button_event_t event;
    set_font_size(14);
    draw_text(0, 216, text_settings_developer[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    draw_text(0, 184, text_reset[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_text(0, 4, text_settings_credits[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(sprs, numsprs, 2);
    setup_cursor(&cursorbg, &cursor, 184);
    draw_all_sprites(spi);
    while(true) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                delete_settings_file();
                FILE* colfile = fopen("/mainfs/colors", "w");
                fprintf(colfile, "#%02x%02x%02x\n#%02x%02x%02x\n%x,%x,%x", settings.RGB_colour.pixelR, settings.RGB_colour.pixelB, settings.RGB_colour.pixelG, settings.RGB_colour_2.pixelR, settings.RGB_colour_2.pixelG, settings.RGB_colour_2.pixelB, settings.RGB_mode, settings.RGB_brightness, settings.RGB_speed);
                fclose(colfile);
                esp_restart();
            }
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return 24;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

const char text_name_blinky[] = "BlinkyBloon";
const char text_name_blue[] = "BalloonBoiBlue";
const char text_name_bb[] = "BalloonBoi";
const char text_name_ath[] = "AaronTheHoss";
const char text_name_erls[] = "Erliora";
const char text_name_yabba[] = "Yabba";
const char text_name_rayraku[] = "Rayraku";
static int menufunc_credits(void) {
    int numsprs;
    SPRITE_NODE* sprs[50];
    button_event_t event;
    set_font_size(14);
    draw_text(0, 216, text_settings_credits[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    sprite_rectangle(85, 211, 150, 21, background_color, false, 0);
    draw_text(10, 192, text_settings_software[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 176, text_settings_hardware[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 160, text_settings_concept[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(0, 192, text_name_blinky, sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(sprs, numsprs, 10);
    draw_text(0, 176, text_name_blue, sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(sprs, numsprs, 10);
    draw_text(0, 160, text_name_bb, sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(sprs, numsprs, 10);
    draw_text(0, 140, text_settings_translations[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_text(10, 120, text_name_ath, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 100, text_name_erls, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(0, 120, text_name_yabba, sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(sprs, numsprs, 10);
    draw_text(0, 100, text_name_rayraku, sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(sprs, numsprs, 10);
    draw_all_sprites(spi);
    while(true) {
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_persistent_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

static int runMenu(const RUNMENU_DATA* const r) {
    void* context = NULL;
    int k;
    rotary_encoder_event_t rotencev;
    button_event_t event;
    if(r->SETUP != NULL)
        r->SETUP(&context);
    while(true) {
        if(r->ROTENC_ACTION != NULL && xQueueReceive(infop->queue, &rotencev, 10/portTICK_PERIOD_MS) == pdTRUE) {
            k = r->ROTENC_ACTION(context, rotencev, r->rotenc_args);
            if(k) goto done;
        }
        if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE) {
            for(int i=0;i<r->NUM_BUTTON_ACTIONS;++i) {
                if(event.pin == r->BUTTONS[i].button_id && event.event == r->BUTTONS[i].button_event_type) {
                    k = r->BUTTONS[i].ACTION(context, r->BUTTONS[i].args);
                    if(k) goto done;
                }
            }
        }
        if(r->POST_LOOP) {
            k = r->POST_LOOP(context, r->POST_LOOP_args);
            if(k) goto done;
        }
    }
done:
    if(r->CLEANUP != NULL)
        r->CLEANUP(&context);
    if(context != NULL)
        free(context);
    return k;
}

const RUNMENU_DATA _RMD_WELCOME_MENU = {
    &_SETUP_welcome_menu,
    &_CLEANUP_COMMON_single_layer_context,
    NULL, NULL,
    &_POSTLOOP_welcome_menu, NULL,
    3, {
        {
            ENCSW,
            BUTTON_DOWN,
            // &_BA_ED_welcome_menu,
            &_BA_COMMON_go_to_menu,
            (void*) (MENU_SETUP_ONLY_TRANSITION_FLAG | 1),
        },
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            // &_BA_ED_welcome_menu,
            &_BA_COMMON_go_to_menu,
            (void*) (MENU_SETUP_ONLY_TRANSITION_FLAG | 1),
        },
        {
            RIGHTBUTTON,
            BUTTON_DOWN,
            // &_BA_ED_welcome_menu,
            &_BA_COMMON_go_to_menu,
            (void*) (MENU_SETUP_ONLY_TRANSITION_FLAG | 1),
        },
    }
};

const RUNMENU_DATA _RMD_SETUP_MENU = {
    &_SETUP_setup_menu,
    &_CLEANUP_COMMON_single_layer_context,
    &_BA_ENC_setup_menu, NULL, 
    NULL, NULL,
    2, {
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) MENU_POP_FLAG,
        },
        {
            RIGHTBUTTON,
            BUTTON_DOWN,
            &_BA_RD_setup_menu_confirm,
            NULL
        }
    }
};

// const int OFFSETOPT = offsetof(struct _WIFI_MENU_CONTEXT, opt);
const RUNMENU_DATA _RMD_WIFI_MENU = {
    &_SETUP_wifi_menu,
    &_CLEANUP_wifi_menu,
    &_ENC_COMMON_scroll_options, (void*) offsetof(struct _WIFI_MENU_CONTEXT, opt), 
    NULL, NULL, 
    3, {
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) MENU_POP_FLAG,
        },
        {
            RIGHTBUTTON,
            BUTTON_DOWN,
            &_BA_RD_wifi_menu_reload,
            NULL
        },
        {
            ENCSW,
            BUTTON_DOWN,
            &_BA_ED_wifi_menu_set_wifi_name,
            NULL
        }
    }
};

const RUNMENU_DATA _RMD_PB_SETUP_METHOD = {
    &_SETUP_pb_setup_method,
    &_CLEANUP_COMMON_single_layer_context, 
    &_ENC_pb_setup_method, NULL,
    NULL, NULL,
    3, {
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) MENU_POP_FLAG,
        },
        {
            RIGHTBUTTON,
            BUTTON_DOWN,
            &_BA_pb_setup_method_confirm,
            NULL
        },
        {
            ENCSW,
            BUTTON_DOWN,
            &_BA_pb_setup_method_confirm,
            NULL
        }
    }
};

const RUNMENU_DATA _RMD_PB_WIFI_CONNECT = {
    &_SETUP_wifi_connect,
    &_CLEANUP_COMMON_single_layer_context,
    NULL, NULL,
    &_POSTLOOP_wifi_connect, NULL,
    1, {
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            &_BA_LD_wifi_connect,
            NULL
        }
    }
};

const RUNMENU_DATA _RMD_PB_WIFI_PREVIEW = {
    &_SETUP_wifi_preview,
    &_CLEANUP_COMMON_single_layer_context,
    &_ENC_wifi_preview, NULL,
    NULL, NULL,
    3, {
        {
            RIGHTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) (MENU_SETUP_ONLY_TRANSITION_FLAG | 22)
        },
        {
            ENCSW,
            BUTTON_DOWN,
            &_BA_ED_wifi_preview,
            NULL
        },
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            &_BA_LD_wifi_preview,
            NULL
        }
    }
};

const RUNMENU_DATA _RMD_TEXT_INPUT = {
    &_SETUP_text_input,
    &_CLEANUP_text_input,
    &_ENC_text_input, NULL,
    NULL, NULL,
    3, {
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) MENU_POP_FLAG
        },
        {
            RIGHTBUTTON,
            BUTTON_DOWN,
            &_BA_RD_switch_register,
            NULL
        },
        {
            ENCSW,
            BUTTON_DOWN,
            &_BA_ED_accept_char,
            NULL
        }
    }
};

const RUNMENU_DATA _RMD_SKIP_WIFI_CONNECTION = {
    &_SETUP_COMMON_no_context,
    &_CLEANUP_COMMON_single_layer_context,
    NULL, NULL,
    NULL, NULL,
    2, {
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) MENU_POP_FLAG
        },
        {
            RIGHTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) (MENU_SETUP_ONLY_TRANSITION_FLAG | MENU_SELF_POP_FLAG | 8)
        },
    }
};

const struct _MODAL_MENU_ARGS_ROTENC DISPLAY_SETTINGS_MM_ARGS_ROTENC = {
    3, NULL,
    {
        &_ENC_display_menu_main_mode,
        &_ENC_display_menu_bright_mode,
        &_ENC_display_menu_theme_mode,
    }
};

const struct _MODAL_MENU_ARGS_BUTTON DISPLAY_SETTINGS_MM_ARGS_BUTTON = {
    3, NULL,
    {
        &_BA_ED_select_value_main_mode,
        &_BA_ED_select_value_brightness_mode,
        &_BA_ED_select_value_theme_mode,
    }
};

const RUNMENU_DATA _RMD_DISPLAY_SETTINGS = {
    &_SETUP_display_menu,
    &_CLEANUP_display_menu,
    &_ENC_COMMON_modal_menu, (void*) &DISPLAY_SETTINGS_MM_ARGS_ROTENC,
    NULL, NULL,
    3, {
        {
            LEFTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) MENU_POP_FLAG
        },
        {
            RIGHTBUTTON,
            BUTTON_DOWN,
            &_BA_COMMON_go_to_menu,
            (void*) (MENU_SETUP_ONLY_TRANSITION_FLAG | 10)
        },
        {
            ENCSW,
            BUTTON_DOWN,
            &_BA_COMMON_modal_menu,
            (void*) &DISPLAY_SETTINGS_MM_ARGS_BUTTON
        }
    }
};

MENU_INFO_t allmenus[] = {
    {&welcome_menu[0], 3, menufunc_welcome, MENU_BG_SOLID_COL, &_RMD_WELCOME_MENU},
    {&menusetup0[0], 4, menufunc_setup, 4, &_RMD_SETUP_MENU},
    {&menusetup3[0], 2, menufunc_wifi_scan, 6, &_RMD_WIFI_MENU},
    {&menutextenter[0], 2, menufunc_text_write, 2, &_RMD_TEXT_INPUT},
    {&menuwifistarting[0], 2, menufunc_connect_wifi, 0, &_RMD_PB_WIFI_CONNECT},
    {&menusetup2a[0], 7, menufunc_http_setup, 3, NULL},
    {&menusetup3[0], 3, menufunc_network_preview, 2, &_RMD_PB_WIFI_PREVIEW},
    {&menusetup1[0], 6, menufunc_pb_setup_method, 5, &_RMD_PB_SETUP_METHOD},
    {&menusetup3[0], 3, menufunc_display_settings, 2, &_RMD_DISPLAY_SETTINGS},
    {&menusetup3[0], 3, menufunc_color_picker, 2, NULL},
    {&menusetup3[0], 3, menufunc_add_on_settings, 2, NULL},
    {&menusetup3[0], 2, menufunc_all_settings, 6, NULL},
    {&menusetup3[0], 2, menufunc_pwm_output_settings, 6, NULL},
    {&menusetup3[0], 2, menufunc_pwm_output_set, 6, NULL},
    {&menusetup3[0], 2, menufunc_rgb_lighting, 2, NULL},
    {&menusetup3[0], 2, menufunc_applications, 6, NULL},
    {&menuapprundelete[0], 10, menufunc_file_run_delete, 0, NULL},
    {NULL, 0, menufunc_execute_ibuf_file, 0, NULL},
    {&menudownloadapp[0], 6, menufunc_download_file, 0, NULL},
    {&menunetworksettings[0], 7, menufunc_network_settings, 0, NULL},
    {&menuserversettings[0], 9, menufunc_server_settings, 6, NULL},
    {&menusetupdone[0], 2, menufunc_setup_done, MENU_BG_SOLID_COL, NULL},
    {&menuskipwifi[0], 4, menufunc_skip_wifi, 0, &_RMD_SKIP_WIFI_CONNECTION},
    {&menusetup3[0], 2, menufunc_developer, 2, NULL},
    {&menusetup3[0], 2, menufunc_credits, 2, NULL},
};

// extern SPRITE_NODE* persistent_sprites;
int start_menu_tree(int startmenu, char settings_mode) {
    int menu_stack[32];
    int menu_stackp = 0;
    int nextmenu;
    char currmenu_background = 0xff;
    MENU_INFO_t* currmenu;
    menu_stack[menu_stackp] = startmenu;
    do {
        currmenu = &allmenus[menu_stack[menu_stackp]];
        if(currmenu->bg == MENU_BG_SOLID_COL)
            send_color(spi, background_color);
        else {
            if(currmenu_background != currmenu->bg) {
                load_bgimg(bgbuf, "/mainfs/pb_bg.cbi", true, currmenu->bg);
                blit_bg();
                currmenu_background = currmenu->bg;
            }
            gen_bg(spi);
        }
        if(currmenu->background != NULL) {
            draw_menu_elements(currmenu->background, currmenu->num_elements);
            draw_all_sprites(spi);
            // if(persistent_sprites != NULL)
            //     delete_persistent_sprites();
        }
        if(currmenu->rmd)
            nextmenu = runMenu(currmenu->rmd);
        else
            nextmenu = currmenu->menu_functionality();
        if(nextmenu & MENU_SELF_POP_FLAG) {
            if((nextmenu & MENU_SETUP_ONLY_TRANSITION_FLAG) && settings_mode) {
                menu_stackp--;
                if(menu_stackp < 0)
                    menu_stackp = 0;
            } else
                menu_stack[menu_stackp] = nextmenu & 0xff;
        } else if(nextmenu & MENU_POP_FLAG) {
            menu_stackp--;
            if(menu_stackp < 0) {
                menu_stackp = 0;
            }
        } else if ((nextmenu & MENU_SETUP_ONLY_TRANSITION_FLAG) && settings_mode) {
            menu_stackp = 0;
            nextmenu = 0;
        } else if (!(nextmenu & MENU_REDRAW_FLAG)) {
            menu_stackp++;
            menu_stack[menu_stackp] = nextmenu & 0xff;
        }
    } while((nextmenu & MENU_RETURN_FLAG) == 0);
    return menu_stackp;
}

// #pragma GCC push_options
// #pragma GCC optimize ("O0")

int draw_menu_elements(const MENU_ELEMENT* elems, int numElements) {
    int err;
    int sizeControl = 0;

    for (int i = 0; i < numElements; i++) {
        if (elems[i].flags & MENU_FLAG_IS_HLINE) {
            draw_hline(elems[i].y, elems[i].textsize, *(elems[i].col), false);
            continue;
        }
        if (elems[i].flags & MENU_FLAG_IS_VLINE) {
            draw_vline(elems[i].x, elems[i].textsize, *(elems[i].col), false);
            continue;
        }
        if (sizeControl != elems[i].textsize) {
            err = set_font_size(elems[i].textsize);
            if (err) {
                ets_printf("!!error in draw_menu_elements: could not set size to %d.\n", elems[i].textsize);
                return 1;
            }
            sizeControl = elems[i].textsize;
        }

        int numsprs;
        SPRITE_NODE* spriteArray[64];

        draw_text(elems[i].x, elems[i].y, (elems[i].flags & MENU_FLAG_LANGUAGE_AGNOSTIC) ? elems[i].text[0] : elems[i].text[settings.language], &spriteArray[0], &numsprs, **(elems[i].col), *background_color, 0, false, false);
        if (elems[i].flags & MENU_FLAG_CENTER) {
            center_sprite_group_x(spriteArray, numsprs);
            continue;
        }
        if (elems[i].flags & MENU_FLAG_RIGHT_JUSTIFY)
            right_justify_sprite_group_x(spriteArray, numsprs, elems[i].x);
    }
    return 0;
}

// #pragma GCC pop_options


extern const uint24_RGB WHITE;
SPRITE_NODE* draw_hline(int y, int thickness, uint24_RGB* colour, bool persistent) {
    uint24_RGB* spriteBuf = (uint24_RGB*) malloc(320*thickness*sizeof(uint24_RGB));
    SPRITE_BITMAP* bmp = (SPRITE_BITMAP*) malloc(sizeof(SPRITE_BITMAP));
    bmp->refcount = 0;
    bmp->c = spriteBuf;
    bmp->w = 320;
    bmp->h = thickness;
    for(int p=0;p<320*thickness;p++) {
        spriteBuf[p].pixelB = colour->pixelB;
        spriteBuf[p].pixelG = colour->pixelG;
        spriteBuf[p].pixelR = colour->pixelR;
    }
    return init_sprite(bmp, 0, 240-y, WHITE, *background_color, true, false, false, true, persistent);
}

SPRITE_NODE* draw_vline(int x, int thickness, uint24_RGB* colour, bool persistent) {
    uint24_RGB* spriteBuf = (uint24_RGB*) malloc(240*thickness*sizeof(uint24_RGB));
    SPRITE_BITMAP* bmp = (SPRITE_BITMAP*) malloc(sizeof(SPRITE_BITMAP));
    bmp->refcount = 0;
    bmp->c = spriteBuf;
    bmp->w = thickness;
    bmp->h = 240;
    for(int p=0;p<240*thickness;p++) {
        spriteBuf[p].pixelB = colour->pixelB;
        spriteBuf[p].pixelG = colour->pixelG;
        spriteBuf[p].pixelR = colour->pixelR;
    }
    return init_sprite(bmp, x, 0, WHITE, *background_color, true, false, false, true, persistent);
}

