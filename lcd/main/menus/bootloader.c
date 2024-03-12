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
#include "rgb_fade.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);
#define MENU_RETURN_FLAG 0x8000
#define MENU_POP_FLAG 0x4000
#define MENU_REDRAW_FLAG 0x2000
#define MENU_SETUP_ONLY_TRANSITION_FLAG 0x1000
#define IBUF_SIZE 256

extern FT_Face typeFace;
extern QueueHandle_t* button_events;
extern rotary_encoder_info_t* infop;
extern spi_device_handle_t spi;
extern uint24_RGB* background_color;
extern uint24_RGB* foreground_color;
// extern uint24_RGB WHITE;
extern char connect_flag;
extern SPRITE_24_H** OAM_SPRITE_TABLE;
extern SETTINGS_t settings;
static char* ibuf;
static uint24_RGB* colorbuf;
extern unsigned char wifi_restart_counter;

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
                return MENU_SETUP_ONLY_TRANSITION_FLAG | 7;
            }
            if(event.pin == 0)
                return MENU_POP_FLAG;
        }
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            currlang = ((unsigned) rotencev.state.position) % 3;
            int sprs[15];
            sprite_rectangle(220, 240-73-22, 100, 22, background_color);
            draw_text(220, 152, &languages[currlang][0], typeFace, &sprs[0], NULL, foreground_color, background_color);
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
            draw_text(0, ys[i], options[i], typeFace, &sprs[0], &name_length, foreground_color, background_color);
            // name_length = strlen(options[i]);
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
    error = draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
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
    draw_text(0, 120, (char*) visibleBuffer, typeFace, sprs, NULL, foreground_color, background_color);
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
    draw_text(0, 184, ibuf, typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(300, 3, "a", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(147, 152, "v", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(147, 88, "^", typeFace, NULL, NULL, foreground_color, background_color);
    draw_textreel(curtable, selection, &loc);

    while(true) {
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                curtable = (curtable + 1) % 3;
                sprite_rectangle(300, 0, 20, 22, background_color);
                draw_text(300, 3, tablename[curtable], typeFace, NULL, NULL, foreground_color, background_color);
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
                draw_text(0, 184, ibuf, typeFace, NULL, NULL, foreground_color, background_color);
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
            return MENU_SETUP_ONLY_TRANSITION_FLAG | 1;
        }
    }
}

static int menufunc_connect_wifi(void) {
    strncpy((char*)sta_wifi_config.sta.ssid, (char*)&settings.wifi_name[0], 32);
    strncpy((char*)sta_wifi_config.sta.password, (char*)&settings.wifi_pass[0], 64);
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));
    ESP_ERROR_CHECK(esp_wifi_connect());
    ets_printf("connecting...\n");
    while(!connect_flag);
    if(wifi_restart_counter >= 10) {
        ets_printf("FAILED!\n");
        return MENU_POP_FLAG;
    }
    ets_printf("success!\n");
    return MENU_SETUP_ONLY_TRANSITION_FLAG | 8;
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
    return MENU_SETUP_ONLY_TRANSITION_FLAG | 8;
}

static int menufunc_network_preview(void) {
    int ys[] = {184, 152, 120, 88, 56};
    char namebuf[14];
    int k[8];
    int lenk;
    button_event_t event;
    rotary_encoder_event_t rotencev;

    if(ibuf != NULL) {
        strncpy(settings.wifi_pass, ibuf, 64);
        free(ibuf);
        ibuf = NULL;
    }
    FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0);

    draw_text(32, 184, "Network", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 152, "Password", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 120, "Advanced", typeFace, k, &lenk, foreground_color, background_color);
    center_sprite_group_x(k, lenk);
    draw_text(32, 88, "Connect", typeFace, k, &lenk, foreground_color, background_color);
    center_sprite_group_x(k, lenk);
    if(strlen(settings.wifi_name) > 10) {
        for(int i=0;i<10;++i) {
            namebuf[i] = settings.wifi_name[i];
        }
        namebuf[11] = '.';
        namebuf[12] = '.';
        namebuf[13] = '.';
        namebuf[14] = 0;
        draw_text(150, 184, namebuf, typeFace, NULL, NULL, foreground_color, background_color);
    } else {
        draw_text(150, 184, &settings.wifi_name[0], typeFace, NULL, NULL, foreground_color, background_color);
    }
    if(strlen(settings.wifi_pass) > 10) {
        for(int i=0;i<10;++i) {
            namebuf[i] = settings.wifi_pass[i];
        }
        namebuf[11] = '.';
        namebuf[12] = '.';
        namebuf[13] = '.';
        namebuf[14] = 0;
        draw_text(150, 152, namebuf, typeFace, NULL, NULL, foreground_color, background_color);
    } else {
        draw_text(150, 152, &settings.wifi_pass[0], typeFace, NULL, NULL, foreground_color, background_color);
    }
    draw_all_sprites(spi);
    delete_all_sprites();
    int cursorbg = sprite_rectangle(2, 184, 20, 16, background_color);
    int cursor;
    int selection = 0;
    draw_text(2, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    draw_all_sprites(spi);
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
                return MENU_SETUP_ONLY_TRANSITION_FLAG | 8;
            }
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
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
}

static int menufunc_pb_setup_method (void) {
    button_event_t event;
    rotary_encoder_event_t rotencev;
    char options_1[] = "Set up PumpBot by connecting";
    char options_2[] = "another device";
    char options_3[] = "Set up PumpBot without connecting";
    char options_4[] = "to another device";
    unsigned char selection = 0;
    int lentt;
    int lenttline2;
    int tooltip_1[37];
    int tooltip_2[44];
    int err = FT_Set_Char_Size (typeFace, 12 << 6, 0, 100, 0);
    int tooltip_bg = sprite_rectangle(0, 25, 320, 16, background_color);
    int tooltip_bg2 = sprite_rectangle(0, 41, 320, 16, background_color);
    int tooltip_bg3 = sprite_rectangle(0, 57, 320, 16, background_color);
    draw_text(0, 52, options_1, typeFace, tooltip_1, &lentt, foreground_color, background_color);
    center_sprite_group_x(tooltip_1, lentt);
    draw_text(0, 34, options_2, typeFace, tooltip_1 + lentt, &lenttline2, foreground_color, background_color);
    center_sprite_group_x(tooltip_1+lentt, lenttline2);
    draw_text(0, 52, options_3, typeFace, tooltip_2, &lentt, foreground_color, background_color);
    center_sprite_group_x(tooltip_2, lentt);
    draw_text(0, 34, options_4, typeFace, tooltip_2+lentt, &lenttline2, foreground_color, background_color);
    center_sprite_group_x(tooltip_2+lentt, lenttline2);
    for(int i=0;i<44;++i) {
        OAM_SPRITE_TABLE[tooltip_2[i]]->draw = false;
    }
    int cursorbg = sprite_rectangle(10, 120, 20, 16, background_color);
    int cursor;
    draw_text(10, 120, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    draw_all_sprites(spi);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            OAM_SPRITE_TABLE[cursorbg]->posY = selection ? 240-88-14 : 240-120-14;
            selection = !selection;
            OAM_SPRITE_TABLE[cursor]->posY = selection ? 240-88-14 : 240-120-14;
            for(int i=0;i<44;++i) {
                OAM_SPRITE_TABLE[tooltip_2[i]]->draw = selection;
            }
            for(int i=0;i<37;++i) {
                OAM_SPRITE_TABLE[tooltip_1[i]]->draw = !selection;
            }
            draw_all_sprites(spi);
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                return MENU_SETUP_ONLY_TRANSITION_FLAG | (selection ? 2 : 5);
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

const char theme_names[][7] = {"dark", "light", "custom"};
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
        delete_all_sprites();
        return MENU_REDRAW_FLAG;
    }
    int numsprs;
    int sprs[15];
    char bright[4];
    (void) itoa(settings.disp_brightness, bright, 10);
    int mode = 0;
    unsigned char selection = 0;
    FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0);
    sprite_rectangle(0, 216, 320, 16, background_color);
    draw_text(0, 216, "Display Settings", typeFace, sprs, &numsprs, foreground_color, background_color);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, "Brightness", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(150, 184, bright, typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(150, 152, theme_names[settings.disp_theme], typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 152, "Theme", typeFace, NULL, NULL, foreground_color, background_color);
    draw_all_sprites(spi);
    delete_all_sprites();
    int cursorbg = sprite_rectangle(10, 184, 20, 16, background_color);
    int cursor;
    int bright_rec = sprite_rectangle(150, 184, 150, 16, background_color);
    int theme_rec = sprite_rectangle(150, 147, 100, 25, background_color);
    OAM_SPRITE_TABLE[bright_rec]->draw = false;
    OAM_SPRITE_TABLE[theme_rec]->draw = false;
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    draw_all_sprites(spi);
    int num_brspr;
    int br_sprite[4]; 
    int num_themespr;
    int theme_sprite[5];
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            switch(mode) {
            case 0:
                OAM_SPRITE_TABLE[cursorbg]->posY = selection ? 240-152-14 : 240-184-14;
                selection = !selection;
                OAM_SPRITE_TABLE[cursor]->posY = selection ? 240-152-14 : 240-184-14;
                draw_all_sprites(spi);
                break;
            case 1:
                settings.disp_brightness += (rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : -1;
                if(settings.disp_brightness > 255) {
                    settings.disp_brightness = 255;
                } else if(settings.disp_brightness < 0) {
                    settings.disp_brightness = 0;
                }
                (void) itoa(settings.disp_brightness, bright, 10);
                draw_text(150, 184, bright, typeFace, br_sprite, &num_brspr, &hicolor, background_color);
                draw_all_sprites(spi);
                for(int i=0;i<num_brspr;++i)
                    delete_sprite(br_sprite[i]);
                ledc_set_duty(LEDC_LOW_SPEED_MODE, 7, settings.disp_brightness << 6);
                ledc_update_duty(LEDC_LOW_SPEED_MODE, 7);
                break;
            case 2:
                settings.disp_theme = (settings.disp_theme + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 2)) % 3;
                draw_text(150, 152, theme_names[settings.disp_theme], typeFace, theme_sprite, &num_themespr, &hicolor, background_color);
                draw_all_sprites(spi);
                for(int i=0;i<num_themespr;++i)
                    delete_sprite(theme_sprite[i]);
                break;
            default:
            }
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                if(mode == 2) {
                    // start the colour menu!
                    if(settings.disp_theme == 2) {
                        colorbuf = calloc(1, 3);
                        colorbuf->pixelR = settings.custom_theme_color.pixelR;
                        colorbuf->pixelG = settings.custom_theme_color.pixelG;
                        colorbuf->pixelB = settings.custom_theme_color.pixelB;
                        delete_all_sprites();
                        return 9;
                    }
                    assign_theme_from_settings();
                    delete_all_sprites();
                    return MENU_REDRAW_FLAG;
                }
                mode = mode == 0 ? selection + 1 : 0;
                switch(mode) {
                case 0:
                    draw_text(150, 184, bright, typeFace, br_sprite, &num_brspr, foreground_color, background_color);
                    draw_text(150, 152, theme_names[settings.disp_theme], typeFace, theme_sprite, &num_themespr, foreground_color, background_color);
                    draw_all_sprites(spi);
                    for(int i=0;i<num_brspr;++i)
                        delete_sprite(br_sprite[i]);
                    for(int i=0;i<num_themespr;++i)
                        delete_sprite(theme_sprite[i]);
                    OAM_SPRITE_TABLE[bright_rec]->draw = false;
                    OAM_SPRITE_TABLE[theme_rec]->draw = false;
                    break;
                case 1:
                    draw_text(150, 184, bright, typeFace, br_sprite, &num_brspr, &RED, background_color);
                    draw_all_sprites(spi);
                    for(int i=0;i<num_brspr;++i)
                        delete_sprite(br_sprite[i]);
                    OAM_SPRITE_TABLE[bright_rec]->draw = true;
                    break;
                case 2:
                    draw_text(150, 152, theme_names[settings.disp_theme], typeFace, theme_sprite, &num_themespr, &RED, background_color);
                    draw_all_sprites(spi);
                    for(int i=0;i<num_themespr;++i)
                        delete_sprite(theme_sprite[i]);
                    OAM_SPRITE_TABLE[theme_rec]->draw = true;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                return MENU_POP_FLAG;
            }
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
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
    FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0);
    sprite_rectangle(85, 211, 150, 21, background_color);
    int numsprs;
    int sprs[10];
    int selection = 0;
    int mode = 0;
    unsigned char buffer[3] = {colorbuf->pixelR, colorbuf->pixelG, colorbuf->pixelB};
    char numbuf[3];
    draw_text(0, 216, "Pick a Color", typeFace, sprs, &numsprs, foreground_color, background_color);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, "Red", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 152, "Green", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 120, "Blue", typeFace, NULL, NULL, foreground_color, background_color);
    (void) itoa(buffer[0], numbuf, 10);
    draw_text(150, 184, numbuf, typeFace, NULL, NULL, foreground_color, background_color);
    (void) itoa(buffer[1], numbuf, 10);
    draw_text(150, 152, numbuf, typeFace, NULL, NULL, foreground_color, background_color);
    (void) itoa(buffer[2], numbuf, 10);
    draw_text(150, 120, numbuf, typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(10, 184, ">", typeFace, NULL, NULL, foreground_color, background_color);
    draw_all_sprites(spi);
    delete_all_sprites();
    int cursorbg = sprite_rectangle(10, 184, 20, 16, background_color);
    int cursor;
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    int red_rec = sprite_rectangle(150, 184, 100, 16, background_color);
    int green_rec = sprite_rectangle(150, 152, 100, 16, background_color);
    int blue_rec = sprite_rectangle(150, 120, 100, 16, background_color);
    OAM_SPRITE_TABLE[red_rec]->draw = false;
    OAM_SPRITE_TABLE[green_rec]->draw = false;
    OAM_SPRITE_TABLE[blue_rec]->draw = false;
    int colorrec;
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            switch(mode) {
            case 0:
                OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection]-14;
                selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 2)) % 3;
                OAM_SPRITE_TABLE[cursor]->posY = 240-ys[selection]-14;
                draw_all_sprites(spi);
                break;
            default:
                buffer[selection] = buffer[selection] + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 255) % 256;
                (void) itoa(buffer[selection], numbuf, 10);
                draw_text(150, ys[selection], numbuf, typeFace, sprs, &numsprs, &RED, background_color);
                draw_all_sprites(spi);
                for(int i=0;i<numsprs;++i)
                    delete_sprite(sprs[i]);
            }
            colorrec = sprite_rectangle(128, 48, 64, 64, (uint24_RGB*) buffer);
            draw_sprites(spi, &colorrec, 1);
            delete_sprite(colorrec);
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                mode = (mode == 0) ? selection + 1 : 0;
                (void) itoa(buffer[selection], numbuf, 10);
                draw_text(150, ys[selection], numbuf, typeFace, sprs, &numsprs, (mode == 0) ? foreground_color : &RED, background_color);
                draw_all_sprites(spi);
                for(int i=0;i<numsprs;++i)
                    delete_sprite(sprs[i]);
                switch(mode) {
                case 0:
                    OAM_SPRITE_TABLE[red_rec]->draw = false;
                    OAM_SPRITE_TABLE[green_rec]->draw = false;
                    OAM_SPRITE_TABLE[blue_rec]->draw = false;
                    break;
                case 1:
                    OAM_SPRITE_TABLE[red_rec]->draw = true;
                    break;
                case 2:
                    OAM_SPRITE_TABLE[green_rec]->draw = true;
                    break;
                case 3:
                    OAM_SPRITE_TABLE[blue_rec]->draw = true;
                }
            }
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                colorbuf->pixelR = buffer[0];
                colorbuf->pixelG = buffer[1];
                colorbuf->pixelB = buffer[2];
                return MENU_POP_FLAG;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
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
    int sprs[14];
    FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0);
    sprite_rectangle(85, 211, 150, 21, background_color);
    draw_text(0, 216, "Add-on Settings", typeFace, sprs, &numsprs, foreground_color, background_color);
    draw_text(32, 184, "Pressure Units", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(190, 184, pressure_names[settings.pressure_units], typeFace, NULL, NULL, foreground_color, background_color);
    center_sprite_group_x(sprs, numsprs);
    draw_all_sprites(spi);
    delete_all_sprites();
    (void) sprite_rectangle(190, 178, 100, 28, background_color);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            settings.pressure_units = (settings.pressure_units + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : -1)) % 4;
            draw_text(190, 184, pressure_names[settings.pressure_units], typeFace, sprs, &numsprs, foreground_color, background_color);
            draw_all_sprites(spi);
            for(int i=0;i<numsprs;++i)
                delete_sprite(sprs[i]);
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 3 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                return MENU_SETUP_ONLY_TRANSITION_FLAG | MENU_RETURN_FLAG;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

const char set_disp[] = "Display";
const char set_net[] = "Network";
const char set_outp[] = "Output";
const char set_RGB[] = "RGB";
const char set_addon[] = "Add-on";
const char set_apps[] = "Apps";
const char set_devs[] = "Developer";
const char* settings_en[] = {set_disp, set_net, set_outp, set_RGB, set_addon, set_apps, set_devs, NULL};
const int selection_to_menu[] = {8, 2, 12, 14, 10, MENU_RETURN_FLAG, MENU_RETURN_FLAG};
static int menufunc_all_settings(void) {
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    int sprs[8];
    int numsprs;
    int cursor;
    int selection = 0;
    int page_start = 0;
    int textbg = sprite_rectangle(50, 184, 220, 21, background_color);
    int cursorbg = sprite_rectangle(10, 184, 20, 16, background_color);
    FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0);
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    int titlebg = sprite_rectangle(85, 211, 150, 21, background_color);
    draw_text(0, 216, "Settings", typeFace, sprs, &numsprs, foreground_color, background_color);
    center_sprite_group_x(sprs, numsprs);
    draw_options((char**)settings_en, textbg);
    for(int i=0;i<numsprs;++i)
        delete_sprite(sprs[i]);
    delete_sprite(titlebg);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection - page_start]-14;
            selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 6)) % 7;
            page_start = (selection > 4) ? selection - 4 : 0;
            if((selection-page_start == 0) || (selection-page_start == 4)) {
                draw_options((char**) (&settings_en[page_start]), textbg);
            }
            OAM_SPRITE_TABLE[cursor]->posY = 240-ys[selection - page_start]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                return selection_to_menu[selection];
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                write_to_file(&settings);
                assign_theme_from_settings();
                return MENU_POP_FLAG;
            }
        }
    }
}

int selected_pwm;
static int menufunc_pwm_output_settings(void) {
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    int sprs[11];
    int numsprs;
    int cursor;
    int selection = 0;
    FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0);
    sprite_rectangle(85, 211, 150, 21, background_color);
    draw_text(0, 216, "PWM settings", typeFace, sprs, &numsprs, foreground_color, background_color);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, "Output 0", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 152, "Output 1", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 120, "Output 2", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 88, "Output 3", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    draw_all_sprites(spi);
    delete_all_sprites();
    int cursorbg = sprite_rectangle(10, 184, 20, 16, background_color);
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection]-14;
            selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 3)) % 4;
            OAM_SPRITE_TABLE[cursor]->posY = 240-ys[selection]-14;
            draw_sprites(spi, &cursorbg, 1);
            draw_sprites(spi, &cursor, 1);
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                selected_pwm = selection;
                return 13;
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                write_to_file(&settings);
                return MENU_POP_FLAG;
            }
        }
    }
}
char analog_mode[] = "Analog";
char digital_mode[] = "Digital";
static int menufunc_pwm_output_set(void) {
    uint24_RGB* hicolor = &RED;
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    int sprs[11];
    int minsprs[6];
    int num_minsprs;
    int numsprs;
    int cursor;
    int selection = 0;
    char percentage[5];
    int mode = 0;
    FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0);
    sprintf(percentage, "%d%%", settings.pwm_min_limit[selected_pwm] / 163);
    draw_text(200, 184, percentage, typeFace, NULL, NULL, foreground_color, background_color);
    sprintf(percentage, "%d%%", settings.pwm_max_limit[selected_pwm] / 163);
    draw_text(200, 152, percentage, typeFace, NULL, NULL, foreground_color, background_color);
    sprite_rectangle(85, 211, 150, 21, background_color);
    draw_text(0, 216, "PWM settings", typeFace, sprs, &numsprs, foreground_color, background_color);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, "Minimum PWM", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 152, "Maximum PWM", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 120, "Setup Wizard", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 88, "Output Mode", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(200, 88, settings.output_set_on_off_only[selected_pwm] ? digital_mode : analog_mode, typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    draw_all_sprites(spi);
    delete_all_sprites();
    int textbg = sprite_rectangle(200, 184, 100, 21, background_color);
    int cursorbg = sprite_rectangle(10, 184, 20, 16, background_color);
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    OAM_SPRITE_TABLE[textbg]->draw = false;
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            switch(mode) {
            case 0:
                OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection]-14;
                selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 3)) % 4;
                OAM_SPRITE_TABLE[cursor]->posY = 240-ys[selection]-14;
                draw_sprites(spi, &cursorbg, 1);
                draw_sprites(spi, &cursor, 1);
                break;
            case 1:
                if(rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) {
                    if(settings.pwm_min_limit[selected_pwm] + 163 < 0x3fff) {
                        settings.pwm_min_limit[selected_pwm] += 163;
                    } else {
                        settings.pwm_min_limit[selected_pwm] = 0x3fff;
                    }
                } else {
                    if(settings.pwm_min_limit[selected_pwm] > 163)
                        settings.pwm_min_limit[selected_pwm] -= 163;
                    else
                        settings.pwm_min_limit[selected_pwm] = 0;
                }
                sprintf(percentage, "%d%%", settings.pwm_min_limit[selected_pwm] / 163);
                draw_text(200, 184, percentage, typeFace, minsprs, &num_minsprs, hicolor, background_color);
                draw_all_sprites(spi);
                for(int i=0;i<num_minsprs;++i)
                    delete_sprite(minsprs[i]);
                break;
            case 2:
                if(rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) {
                    if(settings.pwm_max_limit[selected_pwm] + 163 < 0x3fff) {
                        settings.pwm_max_limit[selected_pwm] += 163;
                    } else {
                        settings.pwm_max_limit[selected_pwm] = 0x3fff;
                    }
                } else {
                    if(settings.pwm_max_limit[selected_pwm] > 163)
                        settings.pwm_max_limit[selected_pwm] -= 163;
                    else
                        settings.pwm_max_limit[selected_pwm] = 0;
                }
                sprintf(percentage, "%d%%", settings.pwm_max_limit[selected_pwm] / 163);
                draw_text(200, 152, percentage, typeFace, sprs, &numsprs, hicolor, background_color);
                draw_all_sprites(spi);
                for(int i=0;i<numsprs;++i)
                    delete_sprite(sprs[i]);
            }
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                if(selection < 2) {
                    mode = (mode == 0) ? selection + 1 : 0;
                    switch(mode) {
                    case 0:
                        OAM_SPRITE_TABLE[textbg]->draw = false;
                        sprintf(percentage, "%d%%", settings.pwm_max_limit[selected_pwm] / 163);
                        draw_text(200, 152, percentage, typeFace, sprs, &numsprs, foreground_color, background_color);
                        sprintf(percentage, "%d%%", settings.pwm_min_limit[selected_pwm] / 163);
                        draw_text(200, 184, percentage, typeFace, minsprs, &num_minsprs, foreground_color, background_color);
                        draw_all_sprites(spi);
                        for(int i=0;i<numsprs;++i)
                            delete_sprite(sprs[i]);
                        for(int i=0;i<num_minsprs;++i)
                            delete_sprite(minsprs[i]);
                        break;
                    case 1:
                        OAM_SPRITE_TABLE[textbg]->draw = true;
                        OAM_SPRITE_TABLE[textbg]->posY = 240-21-184;
                        sprintf(percentage, "%d%%", settings.pwm_min_limit[selected_pwm] / 163);
                        draw_text(200, 184, percentage, typeFace, minsprs, &num_minsprs, hicolor, background_color);
                        draw_all_sprites(spi);
                        for(int i=0;i<num_minsprs;++i)
                            delete_sprite(minsprs[i]);
                        break;
                    case 2:
                        OAM_SPRITE_TABLE[textbg]->draw = true;
                        OAM_SPRITE_TABLE[textbg]->posY = 240-21-152;
                        sprintf(percentage, "%d%%", settings.pwm_max_limit[selected_pwm] / 163);
                        draw_text(200, 152, percentage, typeFace, sprs, &numsprs, hicolor, background_color);
                        draw_all_sprites(spi);
                        for(int i=0;i<numsprs;++i)
                            delete_sprite(sprs[i]);
                    default:
                        break;
                    }
                } else if (selection == 3) {
                    OAM_SPRITE_TABLE[textbg]->draw = true;
                    OAM_SPRITE_TABLE[textbg]->posY = 240-16-88;
                    settings.output_set_on_off_only[selected_pwm] = !settings.output_set_on_off_only[selected_pwm];
                    draw_text(200, 88, settings.output_set_on_off_only[selected_pwm] ? digital_mode : analog_mode, typeFace, sprs, &numsprs, foreground_color, background_color);
                    draw_all_sprites(spi);
                    for(int i=0;i<numsprs;++i)
                        delete_sprite(sprs[i]);
                    OAM_SPRITE_TABLE[textbg]->draw = false;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

char RGB_Mode_Off[] = "Off";
char RGB_Mode_Solid[] = "Solid";
char RGB_Mode_Fade[] = "Fade";
char RGB_Mode_Rainbow[] = "Rainbow";
char* RGB_Mode_Names[] = {RGB_Mode_Off, RGB_Mode_Solid, RGB_Mode_Fade, RGB_Mode_Rainbow};
static int menufunc_rgb_lighting(void) {
    rgb_update();
    uint24_RGB* hicolor = &RED;
    int ys[] = {184, 152, 120, 88, 56};
    button_event_t event;
    rotary_encoder_event_t rotencev;
    int sprs[11];
    int codesprs[11];
    int numsprs;
    int numcodesprs;
    int cursor;
    int selection = 0;
    char percentage[5];
    int mode = 0;
    FT_Set_Char_Size(typeFace, 14 << 6, 0, 100, 0);
    sprintf(percentage, "%d%%", settings.RGB_brightness / 163);
    draw_text(200, 184, percentage, typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(200, 152, RGB_Mode_Names[settings.RGB_mode], typeFace, NULL, NULL, foreground_color, background_color);
    sprite_rectangle(85, 211, 150, 21, background_color);
    draw_text(0, 216, "RGB Settings", typeFace, sprs, &numsprs, foreground_color, background_color);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, "Brightness", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 152, "Mode", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 120, "Color 1", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(32, 88, "Color 2", typeFace, NULL, NULL, foreground_color, background_color);
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    sprite_rectangle(200, 112, 30, 30, &settings.RGB_colour);
    sprite_rectangle(200, 80, 30, 30, &settings.RGB_colour_2);
    draw_all_sprites(spi);
    delete_all_sprites();
    int textbg = sprite_rectangle(200, 184, 100, 21, background_color);
    int cursorbg = sprite_rectangle(10, 184, 20, 16, background_color);
    draw_text(10, 184, ">", typeFace, &cursor, NULL, foreground_color, background_color);
    OAM_SPRITE_TABLE[textbg]->draw = false;
    while(true) {
        if(xQueueReceive(infop->queue, &rotencev, 50/portTICK_PERIOD_MS) == pdTRUE) {
            switch(mode) {
            case 0:
                OAM_SPRITE_TABLE[cursorbg]->posY = 240-ys[selection]-14;
                selection = (selection + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 3)) % 4;
                OAM_SPRITE_TABLE[cursor]->posY = 240-ys[selection]-14;
                draw_sprites(spi, &cursorbg, 1);
                draw_sprites(spi, &cursor, 1);
                break;
            case 1:
                if(rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) {
                    if(settings.RGB_brightness + 163 < 0x3fff)
                        settings.RGB_brightness += 163;
                    else
                        settings.RGB_brightness = 0x3fff;
                } else {
                    if(settings.RGB_brightness > 163)
                        settings.RGB_brightness -= 163;
                    else
                        settings.RGB_brightness = 0;
                }
                sprintf(percentage, "%d%%", settings.RGB_brightness / 163);
                draw_text(200, 184, percentage, typeFace, sprs, &numsprs, hicolor, background_color);
                OAM_SPRITE_TABLE[textbg]->draw = true;
                draw_all_sprites(spi);
                for(int i=0;i<numsprs;++i)
                    delete_sprite(sprs[i]);
                rgb_update();
                break;
            case 2:
                settings.RGB_mode = (settings.RGB_mode + ((rotencev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 3)) % 4;
                draw_text(200, 152, RGB_Mode_Names[settings.RGB_mode], typeFace, codesprs, &numcodesprs, hicolor, background_color);
                OAM_SPRITE_TABLE[textbg]->draw = true;
                draw_all_sprites(spi);
                for(int i=0;i<numcodesprs;++i)
                    delete_sprite(codesprs[i]);
                break;
            }
        }
        if(xQueueReceive(*button_events, &event, 50/portTICK_PERIOD_MS) == pdTRUE) {
            if(event.pin == 18 && event.event == BUTTON_DOWN) {
                if(selection < 2) {
                    mode = (mode == 0) ? selection + 1 : 0;
                    switch(mode) {
                    case 0:
                        draw_text(200, 184, percentage, typeFace, sprs, &numsprs, foreground_color, background_color);
                        draw_text(200, 152, RGB_Mode_Names[settings.RGB_mode], typeFace, codesprs, &numcodesprs, foreground_color, background_color);
                        OAM_SPRITE_TABLE[textbg]->draw = false;
                        draw_all_sprites(spi);
                        for(int i=0;i<numsprs;++i)
                            delete_sprite(sprs[i]);
                        for(int i=0;i<numcodesprs;++i)
                            delete_sprite(codesprs[i]);
                        break;
                    case 1:
                        draw_text(200, 184, percentage, typeFace, sprs, &numsprs, hicolor, background_color);
                        OAM_SPRITE_TABLE[textbg]->draw = true;
                        OAM_SPRITE_TABLE[textbg]->posY = 240-21-184;
                        draw_all_sprites(spi);
                        for(int i=0;i<numsprs;++i)
                            delete_sprite(sprs[i]);
                        break;
                    case 2:
                        draw_text(200, 152, RGB_Mode_Names[settings.RGB_mode], typeFace, codesprs, &numcodesprs, hicolor, background_color);
                        OAM_SPRITE_TABLE[textbg]->draw = true;
                        OAM_SPRITE_TABLE[textbg]->posY = 240-21-152;
                        draw_all_sprites(spi);
                        for(int i=0;i<numcodesprs;++i)
                            delete_sprite(codesprs[i]);
                        break;
                    }
                } else {
                    colorbuf = (selection == 2) ? &settings.RGB_colour : &settings.RGB_colour_2;
                    delete_all_sprites();
                    return 9;
                }
            }
            if(event.pin == 0 && event.event == BUTTON_DOWN) {
                delete_all_sprites();
                return MENU_POP_FLAG;
            }
        }
    }
}

MENU_INFO_t allmenus[] = {
    {&welcome_menu[0], 3, menufunc_welcome},
    {&menusetup0[0], 8, menufunc_setup},
    {&menusetup3[0], 9, menufunc_wifi_scan},
    {&menusetup3[0], 9, menufunc_text_write},
    {&menuwifistarting[0], 3, menufunc_connect_wifi},
    {&menusetup2a[0], 3, menufunc_http_setup},
    {&menusetup3[0], 10, menufunc_network_preview},
    {&menusetup1[0], 10, menufunc_pb_setup_method},
    {&menusetup3[0], 10, menufunc_display_settings},
    {&menusetup3[0], 10, menufunc_color_picker},
    {&menusetup3[0], 10, menufunc_add_on_settings},
    {&menusetup3[0], 9, menufunc_all_settings},
    {&menusetup3[0], 9, menufunc_pwm_output_settings},
    {&menusetup3[0], 9, menufunc_pwm_output_set},
    {&menusetup3[0], 9, menufunc_rgb_lighting},
};

int start_menu_tree(int startmenu, char settings_mode) {
    int menu_stack[20];
    int menu_stackp = 0;
    int nextmenu;
    char do_text_menu = 0;
    MENU_INFO_t* currmenu;
    menu_stack[menu_stackp] = startmenu;
    do {
        send_color(spi, background_color);
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
        } else if ((nextmenu & MENU_SETUP_ONLY_TRANSITION_FLAG) && settings_mode) {
            menu_stackp = 0;
            nextmenu = 0;
        } else if (!(nextmenu & MENU_REDRAW_FLAG)) {
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
            draw_hline(elems[i].y, elems[i].textsize, *(elems[i].col));
            continue;
        }
        if (elems[i].vline) {
            draw_vline(elems[i].x, elems[i].textsize, *(elems[i].col));
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
        // int spriteArray[elems[i].textlen - elems[i].numspaces];
        int numsprs;
        int spriteArray[64];

        err = draw_text(elems[i].x, elems[i].y, elems[i].text, typeFace, &spriteArray[0], &numsprs, *(elems[i].col), NULL);
        if (elems[i].center)
            // center_sprite_group_x(spriteArray, elems[i].textlen - elems[i].numspaces);
            center_sprite_group_x(spriteArray, numsprs);
        if (err)
            return err;
    }
    return 0;
}

// #pragma GCC pop_options

int draw_hline(int y, int thickness, uint24_RGB* colour) {
    uint24_RGB* spriteBuf = (uint24_RGB*) malloc(320*thickness*sizeof(uint24_RGB));
    SPRITE_BITMAP* bmp = (SPRITE_BITMAP*) malloc(sizeof(SPRITE_BITMAP));
    bmp->refcount = 1;
    bmp->c = spriteBuf;
    for(int p=0;p<320*thickness;p++) {
        spriteBuf[p].pixelB = colour->pixelB;
        spriteBuf[p].pixelG = colour->pixelG;
        spriteBuf[p].pixelR = colour->pixelR;
    }
    return init_sprite(bmp, 0, 240-y, 320, thickness, false, false, true);
}

int draw_vline(int x, int thickness, uint24_RGB* colour) {
    uint24_RGB* spriteBuf = (uint24_RGB*) malloc(240*thickness*sizeof(uint24_RGB));
    SPRITE_BITMAP* bmp = (SPRITE_BITMAP*) malloc(sizeof(SPRITE_BITMAP));
    bmp->refcount = 1;
    bmp->c = spriteBuf;
    for(int p=0;p<240*thickness;p++) {
        spriteBuf[p].pixelB = colour->pixelB;
        spriteBuf[p].pixelG = colour->pixelG;
        spriteBuf[p].pixelR = colour->pixelR;
    }
    return init_sprite(bmp, x, 0, thickness, 240, false, false, true);
}
    
