#include "menu_functions.h"
#include "fontfile.h"
#include "menus.h"
#include "oam.h"
#include "lua_exports.h"
#include "lang.h"
#include "settings.h"
#include <string.h>
#include "board_config.h"
#include "rom/ets_sys.h"
#include <driver/gpio.h>
#include "system_status.h"
#include "file_server.h"
#include "pwm_output.h"
#include "socket.h"

extern spi_device_handle_t spi;
extern uint24_RGB* background_color;
extern uint24_RGB* foreground_color;
extern SETTINGS_t settings;
extern unsigned char wifi_restart_counter;
extern uint24_RGB* bgbuf;
extern uint16_t button_disable_counter;
char* TEXT_ENTRY_BUFFER;
unsigned char TEXT_ENTRY_BUFFER_LENGTH;
uint24_RGB COLOR_SELECTION_BUFFER;
unsigned char COLOR_SELECTION_BUFFER_VALID;
const uint24_RGB HIGHLIGHT_COLOR = {0xff, 0x00, 0x00};

// HELPERS {{{
void ellipsized_name(char* dest, char* src, int max) {
    strncpy(dest, src, max);
    if(strlen(dest) > max) {
        dest[max+1] = '.';
        dest[max+2] = '.';
        dest[max+3] = '.';
        dest[max+4] = 0;
    } else {
        dest[max+1] = 0;
    }
}

void setup_cursor(SPRITE_NODE** cursorbg, SPRITE_NODE** cursor, int y) {
    draw_text(10, y, ">", cursor, NULL, *foreground_color, *background_color, 0, false, true);
    *cursorbg = sprite_rectangle(10, y, 20, 16, background_color, true, 0);
}
// }}}
// COMMON {{{
int _BA_COMMON_go_to_menu(void* context, void* args) {
    (void) context;
    return *(int*) &args;
}

void _SETUP_COMMON_no_context(void** context) {
    (void) context;
}

void _CLEANUP_COMMON_single_layer_context(void** context) {
    // delete_persistent_sprites();
    delete_all_sprites_immediate();
}

int _ENC_WRAP_BLOCK_HID(void* context, rotary_encoder_event_t ev, void* args) {
    struct _BLOCK_HID_ARGS_ROTENC* a = (struct _BLOCK_HID_ARGS_ROTENC*) args;
    if(button_disable_counter <= 0)
        return a->callback(context, ev, a->args);
    return 0;
}

int _BA_WRAP_BLOCK_HID(void* context, void* args) {
    struct _BLOCK_HID_ARGS_BUTTON* a = (struct _BLOCK_HID_ARGS_BUTTON*) args;
    if(button_disable_counter <= 0)
        return a->callback(context, a->args);
    return 0;
}

struct _MODAL_MENU_CONTEXT_WRAPPER {
    int mode;
    void* context;
};
void* _HELP_COMMON_create_and_wrap_modal_context(void* context) {
    struct _MODAL_MENU_CONTEXT_WRAPPER* x = malloc(sizeof(struct _MODAL_MENU_CONTEXT_WRAPPER));
    x->mode = 0;
    x->context = context;
    return x;
}

// no need to free, as usually this context will be the outer context, and the menu runner function will take care of it
void* _HELP_COMMON_unwrap_modal_context(void* context) {
    struct _MODAL_MENU_CONTEXT_WRAPPER* x = (struct _MODAL_MENU_CONTEXT_WRAPPER*) context;
    return x->context;
}

int _ENC_COMMON_modal_menu(void* context, rotary_encoder_event_t ev, void* args) {
    struct _MODAL_MENU_CONTEXT_WRAPPER* x = (struct _MODAL_MENU_CONTEXT_WRAPPER*) context;
    struct _MODAL_MENU_ARGS_ROTENC* a = (struct _MODAL_MENU_ARGS_ROTENC*) args;
    if(x->mode > a->num_modes || x->mode < 0) {
        ets_printf("INVALID MODE REACHED! %d\n", x->mode);
        return MENU_POP_FLAG;
    }
    return a->modes[x->mode](x->context, ev, a->args, &x->mode);
}

int _BA_COMMON_modal_menu(void* context, void* args) {
    struct _MODAL_MENU_CONTEXT_WRAPPER* x = (struct _MODAL_MENU_CONTEXT_WRAPPER*) context;
    struct _MODAL_MENU_ARGS_BUTTON* a = (struct _MODAL_MENU_ARGS_BUTTON*) args;
    if(x->mode > a->num_modes || x->mode < 0) {
        ets_printf("INVALID MODE REACHED! %d\n", x->mode);
        return MENU_POP_FLAG;
    }
    return a->modes[x->mode](x->context, a->args, &x->mode);
}

const int OPTION_Ys[] = {184, 152, 120, 88, 56};
void _HELP_COMMON_draw_options(struct _OPTIONS_DATA_* opt, int pageStart) {
    SPRITE_NODE* sprs[64];
    int numsprs;
    // SPRITE_NODE* textbg = sprite_rectangle(50, 184, 220, 21, background_color, false, 0);
    int numToDraw = opt->numOptions > 5 ? 5 : opt->numOptions;
    for(int i=0;i<numToDraw;++i) {
        // textbg->v->posY = 224 - OPTION_Ys[i];
        // draw_sprites(spi, &textbg, 1);
        draw_text(0, OPTION_Ys[i], opt->options[pageStart + i], &sprs[0], &numsprs, *foreground_color, *background_color, 0, false, false);
        set_sprites_lifetime(1, sprs, numsprs);
        center_sprite_group_x(sprs, numsprs);
    }
    // delete_node(textbg);
    draw_all_sprites(spi);
}

void _HELP_COMMON_clear_options(int numOpts) {
    // SPRITE_NODE* textbg = sprite_rectangle(50, 184, 220, 21, background_color, false, 0);
    // for(int i=0;i<numOpts;++i) {
    //     textbg->v->posY = 224 - OPTION_Ys[i];
    //     draw_sprites(spi, &textbg, 1);
    // }
    // draw_all_sprites(spi);
    // delete_node(textbg);
}

int _ENC_COMMON_scroll_options(void* context, rotary_encoder_event_t ev, void* args) {
    // context must point to options first;
    unsigned int offset = *(unsigned int*) &args;
    struct _OPTIONS_DATA_* opt = (struct _OPTIONS_DATA_*) (context + offset);
    opt->currentOption = (ev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? (opt->currentOption + 1) % opt->numOptions : (opt->currentOption + opt->numOptions - 1) % opt->numOptions;
    int relativeCursorLocation, pageStart;
    if(opt->currentOption < 2 || opt->numOptions < 5) {
        relativeCursorLocation = opt->currentOption;
        pageStart = 0;
    } else if(opt->currentOption > opt->numOptions-3) {
        relativeCursorLocation = 5 + opt->currentOption - opt->numOptions;
        pageStart = opt->numOptions - 5;
    } else {
        relativeCursorLocation = 2;
        pageStart = opt->currentOption - 2;
    }
    // use relative cursor location and page start to draw options.
    // draw_options(&opt->options[pageStart], relativeCursorLocation, opt->numOptions, opt->cursor, opt->cursorbg);
    opt->cursorbg->v->posY = opt->cursor->v->posY;
    opt->cursor->v->posY = 240 - OPTION_Ys[relativeCursorLocation] - 14;
    _HELP_COMMON_draw_options(opt, pageStart);
    return 0;
}
// }}}

// WELCOME MENU {{{
struct _CONTEXT_wm {
    int counter;
    int currlang;
};
void _SETUP_welcome_menu(void** context) {
    struct _CONTEXT_wm* cont = malloc(sizeof(struct _CONTEXT_wm));
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
        draw_text(60, 195, text_welcome[ct->currlang], sprs, &numsprs, *foreground_color, *background_color, 0, true, false);
        center_sprite_group_x(sprs, numsprs);

        draw_text(60, 154, text_welcome_a[ct->currlang], sprs, &numsprs, *foreground_color, *background_color, 0, true, false);
        center_sprite_group_x(sprs, numsprs);

        set_font_size(14);
        draw_text(60, 10, text_pressenc[ct->currlang], sprs, &numsprs, *foreground_color, *background_color, 0, true, false);
        center_sprite_group_x(sprs, numsprs);

        set_font_size(24);
        draw_all_sprites(spi);
        wait_for_end_of_frame();
        delete_persistent_sprites();
        ct->counter = 200;
    }
    return 0;
}
// }}}
// LANGUAGE MENU {{{

void _SETUP_setup_menu(void** context) {
    struct _CONTEXT_wm* cont = malloc(sizeof(struct _CONTEXT_wm));
    cont->currlang = 0;
    *context = (void*) cont;
    SPRITE_NODE* sprs[15];
    int numsprs;
    draw_text(220, 161, text_language_name[0], &sprs[0], &numsprs, *foreground_color, *background_color, 0, false, false);
    set_sprites_lifetime(1, sprs, numsprs);

    draw_all_sprites(spi);
    sprite_rectangle(220, 240-73-13, 100, 22, background_color, true, 0);
    set_font_size(14);
}

int _BA_RD_setup_menu_confirm(void* context, void* args) {
    struct _CONTEXT_wm* cont = (struct _CONTEXT_wm*) context;
    settings.language = cont->currlang;
    return MENU_SETUP_ONLY_TRANSITION_FLAG | 7;
}

int _BA_ENC_setup_menu(void* context, rotary_encoder_event_t ev, void* args) {
    struct _CONTEXT_wm* cont = (struct _CONTEXT_wm*) context;
    cont->currlang = ((unsigned) ev.state.position) % 9;
    SPRITE_NODE* sprs[15];
    int numsprs;
    draw_text(220, 161, text_language_name[cont->currlang], &sprs[0], &numsprs, *foreground_color, *background_color, 0, false, false);
    set_sprites_lifetime(1, sprs, numsprs);

    draw_all_sprites(spi);
    return 0;
}
// }}}
// WIFI LIST MENU {{{
int load_wifi(struct _WIFI_MENU_CONTEXT* ctx) {
    // wifi_ap_record_t ap_info[numWifi];
    uint16_t ap_count = 0;
    uint16_t max_wifis = NUM_WIFIS;
    SPRITE_NODE* sprs[64];
    int numsprs;
    draw_text(0, 120, text_searching[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, true);
    set_sprites_lifetime(1, sprs, numsprs);
    center_sprite_group_x(sprs, numsprs);
    draw_all_sprites(spi);
    ESP_ERROR_CHECK(esp_wifi_scan_start(NULL, true));
    ESP_ERROR_CHECK(esp_wifi_scan_get_ap_records(&max_wifis, &ctx->ap_info[0]));
    ESP_ERROR_CHECK(esp_wifi_scan_get_ap_num(&ap_count));
    for(int i=0;i<ap_count;++i) {
        ctx->opt.options[i] = (char*) ctx->ap_info[i].ssid;
    }
    ctx->opt.numOptions = ap_count;
    return 0;
}

void _SETUP_wifi_menu(void** context) {
    struct _WIFI_MENU_CONTEXT* ctx = malloc(sizeof(struct _WIFI_MENU_CONTEXT));
    *context = (void*) ctx;
    ctx->opt.currentOption = 0;
    ctx->opt.numOptions = 0;
    memset(ctx->ap_info, 0, NUM_WIFIS * sizeof(wifi_ap_record_t));
    ctx->opt.options = malloc(33 * NUM_WIFIS * sizeof(char));

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));

    (void) _BA_RD_wifi_menu_reload(ctx, NULL);
}

void _CLEANUP_wifi_menu(void** context) {
    free(((struct _WIFI_MENU_CONTEXT*) context)->opt.options);
    _CLEANUP_COMMON_single_layer_context(context);
    return;
}

int _BA_ED_wifi_menu_set_wifi_name(void* context, void* args) {
    struct _WIFI_MENU_CONTEXT* ctx = (struct _WIFI_MENU_CONTEXT*) context;
    strncpy(&settings.wifi_name[0], (char*)ctx->ap_info[ctx->opt.currentOption].ssid, 32);
    return 6;
}

int _BA_RD_wifi_menu_reload(void* context, void* args) {
    struct _WIFI_MENU_CONTEXT* ctx = (struct _WIFI_MENU_CONTEXT*) context;
    SPRITE_NODE* sprs[10];
    int numsprs;
    ctx->opt.currentOption = 0;
    _HELP_COMMON_clear_options(ctx->opt.numOptions>5 ? 5 : ctx->opt.numOptions);
    draw_text(270, 2, text_search[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    right_justify_sprite_group_x(sprs, numsprs, 2);

    draw_all_sprites(spi);

    load_wifi(ctx);

    //set up the cursor
    setup_cursor(&ctx->opt.cursorbg, &ctx->opt.cursor, OPTION_Ys[0]);

    _HELP_COMMON_draw_options(&ctx->opt, 0);
    return 0;
}
// }}}
// SETUP METHOD MENU {{{
struct PB_SETUP_METHOD_CONTEXT {
    unsigned char selection;
    SPRITE_NODE* tooltip_1[60];
    SPRITE_NODE* tooltip_2[60];
    int lentt1;
    int lentt2;
};
void _SETUP_pb_setup_method(void** context) {
    const char* options_1 = text_tooltip_wifi_setup[settings.language];
    const char* options_2 = text_tooltip_wifi_setup_a[settings.language];
    const char* options_3 = text_tooltip_standalone_setup[settings.language];
    const char* options_4 = text_tooltip_standalone_setup_a[settings.language];
    struct PB_SETUP_METHOD_CONTEXT* cont = malloc(sizeof(struct PB_SETUP_METHOD_CONTEXT));
    cont->selection = 0;
    *context = cont;

    int lentt;
    int lenttline2;
    set_font_size(12);
    draw_text(0, 52, options_1, cont->tooltip_1, &lentt, *foreground_color, *background_color, 0, false, true);
    center_sprite_group_x(cont->tooltip_1, lentt);
    draw_text(0, 34, options_2, cont->tooltip_1 + lentt, &lenttline2, *foreground_color, *background_color, 0, false, true);
    center_sprite_group_x(cont->tooltip_1+lentt, lenttline2);
    cont->lentt1 = lentt+lenttline2;
    draw_text(0, 52, options_3, cont->tooltip_2, &lentt, *foreground_color, *background_color, 0, false, true);
    center_sprite_group_x(cont->tooltip_2, lentt);
    draw_text(0, 34, options_4, cont->tooltip_2+lentt, &lenttline2, *foreground_color, *background_color, 0, false, true);
    center_sprite_group_x(cont->tooltip_2+lentt, lenttline2);
    for(int i=0;i<lentt+lenttline2;++i) {
        cont->tooltip_2[i]->v->draw = false;
    }
    cont->lentt2 = lentt+lenttline2;
    SPRITE_NODE* CURSOR;
    draw_text(10, 137, ">", &CURSOR, NULL, *foreground_color, *background_color, 0, false, false);
    set_sprites_lifetime(1, &CURSOR, 1);
    sprite_rectangle(0, 25, 320, 16, background_color, true, 0);
    sprite_rectangle(0, 41, 320, 16, background_color, true, 0);
    sprite_rectangle(0, 57, 320, 16, background_color, true, 0);
    draw_all_sprites(spi);
}

int _ENC_pb_setup_method(void* context, rotary_encoder_event_t ev, void* args) {
    struct PB_SETUP_METHOD_CONTEXT* cont = (struct PB_SETUP_METHOD_CONTEXT*) context;
    // int cursorBgPos = cont->selection ? 240-105-14 : 240-137-14;
    cont->selection = !cont->selection;
    int cursorPos = cont->selection ? 105: 137;
    SPRITE_NODE* CURSOR;
    draw_text(10, cursorPos, ">", &CURSOR, NULL, *foreground_color, *background_color, 0, false, false);
    set_sprites_lifetime(1, &CURSOR, 1);
    for(int i=0;i<cont->lentt1;++i) {
        if(cont->selection)
            undraw_node(cont->tooltip_1[i]);
        else
            cont->tooltip_1[i]->v->draw = 1;
    }
    for(int i=0;i<cont->lentt2;++i) {
        if(!cont->selection)
            undraw_node(cont->tooltip_2[i]);
        else
            cont->tooltip_2[i]->v->draw = 1;
    }
    draw_all_sprites(spi);
    return 0;
}

int _BA_pb_setup_method_confirm(void* context, void* args) {
    delete_persistent_sprites();
    struct PB_SETUP_METHOD_CONTEXT* cont = (struct PB_SETUP_METHOD_CONTEXT*) context;
    return MENU_SETUP_ONLY_TRANSITION_FLAG | (cont->selection ? 2 : 5);
}
// }}}
// WIFI CONNECTING MENU {{{
void _SETUP_wifi_connect(void** context) {
    strncpy((char*)sta_wifi_config.sta.ssid, (char*)&settings.wifi_name[0], 32);
    strncpy((char*)sta_wifi_config.sta.password, (char*)&settings.wifi_pass[0], 64);
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));
    ESP_ERROR_CHECK(esp_wifi_connect());
    set_font_size(14);
    TEXT_ENTRY_BUFFER = NULL;
    COLOR_SELECTION_BUFFER_VALID = 0;
}

int _BA_LD_wifi_connect(void* context, void* args) {
    if((system_flags & (FLAG_WIFI_CONNECTED | FLAG_WIFI_TIMED_OUT)))
        return 0;
    wifi_restart_counter = 10;
    return MENU_POP_FLAG;
}

int _POSTLOOP_wifi_connect(void* context, void* args) {
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
// }}}
// WIFI PREVIEW MENU {{{
struct _WIFI_PREVIEW_CONTEXT {
    unsigned char selection;
};

void _SETUP_wifi_preview(void** context) {
    SPRITE_NODE* sprs[15];
    int numsprs;
    char namebuf[14];
    struct _WIFI_PREVIEW_CONTEXT* cont = malloc(sizeof(struct _WIFI_PREVIEW_CONTEXT));
    SPRITE_NODE* cursor;
    *context = (void*) cont;

    if(TEXT_ENTRY_BUFFER != NULL) {
        strncpy(settings.wifi_pass, TEXT_ENTRY_BUFFER, 64);
        free(TEXT_ENTRY_BUFFER);
        TEXT_ENTRY_BUFFER = NULL;
        TEXT_ENTRY_BUFFER_LENGTH = 0;
    }
    set_font_size(14);
    draw_text(32, 184, text_settings_network[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 152, text_password[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 120, (system_flags & FLAG_WIFI_CONNECTED) ? text_disconnect[settings.language] : text_connect[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    ellipsized_name(namebuf, settings.wifi_name, 10);
    draw_text(150, 184, namebuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    ellipsized_name(namebuf, settings.wifi_pass, 10);
    draw_text(150, 152, namebuf, NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 184, ">", &cursor, NULL, *foreground_color, *background_color, 0, false, true);
    set_sprites_lifetime(1, &cursor, 1);
    draw_all_sprites(spi);
}

int _BA_LD_wifi_preview(void* context, void* args) {
    memset(&settings.wifi_pass[0], 0, 64);
    return MENU_POP_FLAG;
}

int _BA_ED_wifi_preview(void* context, void* args) {
    struct _WIFI_PREVIEW_CONTEXT* cont = (struct _WIFI_PREVIEW_CONTEXT*) context;
    switch(cont->selection) {
    case 0:
        return _BA_LD_wifi_preview(context, args);
    case 1:
        TEXT_ENTRY_BUFFER = calloc(65, sizeof(char));
        strncpy(TEXT_ENTRY_BUFFER, &settings.wifi_pass[0], 64);
        TEXT_ENTRY_BUFFER_LENGTH = 65;
        return 3;
    case 2:
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
    return 0;
}

int _ENC_wifi_preview(void* context, rotary_encoder_event_t ev, void* args) {
    int ys[] = {184, 152, 120};
    SPRITE_NODE* cursor;
    struct _WIFI_PREVIEW_CONTEXT* cont = (struct _WIFI_PREVIEW_CONTEXT*) context;
    cont->selection = (ev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? (cont->selection + 1) % 3 : (cont->selection + 2) % 3;
    draw_text(10, ys[cont->selection], ">", &cursor, NULL, *foreground_color, *background_color, 0, false, true);
    set_sprites_lifetime(1, &cursor, 1);
    draw_all_sprites(spi);
    return 0;
}
// }}}
// TEXT INPUT MENU {{{
const char TEXT_table1[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ␣⌫✓";
const char TEXT_table2[] = "abcdefghijklmnopqrstuvwxyz␣⌫✓";
const char TEXT_table3[] = "\"#$%&'()*+,-./0123456789:;<=>!?@␣⌫✓";
const char* TEXT_metatable[] = {TEXT_table1, TEXT_table2, TEXT_table3};
const unsigned char TEXT_string_lengths[] = {29, 29, 36};
const char TEXT_tablename[][2] = {"a", "@", "A"};
const int TEXT_xs[] = {40, 60, 80, 100, 120, 140, 160, 180, 200, 220, 240, 260};
const int TEXT_ys[] = {90, 58, 26};
const char* TEXT_CURSOR_1 = "▵";
const char* TEXT_CURSOR_2 = "v";
#define TEXT_MENU_PREVIEW_X 2
#define TEXT_MENU_PREVIEW_Y 184
#define TEXT_MENU_PREVIEW_LINE_HEIGHT 24
#define NUM_TEXT_ROWS 12
const unsigned char TEXT_ROW_TABLE[] = {
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2};
enum TEXT_CHARREGISTER {
    UPPERCASE = 0,
    LOWERCASE,
    SYMBOLS
};
struct _TEXT_INPUT_CONTEXT {
    enum TEXT_CHARREGISTER charReg; 
    SPRITE_NODE* grid_chars[36];
    int num_grid_chars;
    unsigned int selection;
    SPRITE_NODE* cursor[2];
    SPRITE_NODE* reg_tag;
    SPRITE_NODE** text_buffer_chars;
    int num_text_buffer_chars;
    unsigned int next_x;
    unsigned int next_y;
};

void _HELP_draw_grid(SPRITE_NODE** sprs, int* numsprs, int reg) {
    for(int i=0;i<*numsprs;++i) // IF numsprs == 0 then we don't delete
        delete_node(sprs[i]);
    draw_text(0, 10, TEXT_metatable[reg], sprs, numsprs, *foreground_color, *background_color, 0, false, false);
    for(int i=0;i<*numsprs;++i) {
        sprs[i]->v->posX = TEXT_xs[i%NUM_TEXT_ROWS];
        sprs[i]->v->posY = sprs[i]->v->posY - TEXT_ys[TEXT_ROW_TABLE[i]];
    }
}

#define CURSOR_NUM_SPRS 2
#define CURSOR_SPACING 14
void _HELP_draw_cursor(struct _TEXT_INPUT_CONTEXT* cont) {
    if(cont->cursor[0] != NULL) {
        delete_node(cont->cursor[0]);
        delete_node(cont->cursor[1]);
    }
    int x = cont->grid_chars[cont->selection]->v->posX;
    int y = TEXT_ys[TEXT_ROW_TABLE[cont->selection]];
    int num;
    draw_text(x, y-CURSOR_SPACING+10, TEXT_CURSOR_1, &cont->cursor[0], &num, *foreground_color, *background_color, 0, false, false);
    draw_text(x, y+CURSOR_SPACING+10, TEXT_CURSOR_2, &cont->cursor[1], &num, *foreground_color, *background_color, 0, false, false);
}

void _HELP_draw_register_tag(enum TEXT_CHARREGISTER cr, SPRITE_NODE** tag) {
    if(*tag != NULL)
        delete_node(*tag);

    int num = 0;
    draw_text(300, 5, TEXT_tablename[cr], tag, &num, *foreground_color, *background_color, 0, false, false);
}

void _HELP_draw_text_preview(struct _TEXT_INPUT_CONTEXT* cont) {
    for(int i=0;i<cont->num_text_buffer_chars;++i)
        delete_node(cont->text_buffer_chars[i]);
    int adv_x = draw_text(TEXT_MENU_PREVIEW_X, TEXT_MENU_PREVIEW_Y, TEXT_ENTRY_BUFFER, cont->text_buffer_chars, &cont->num_text_buffer_chars, *foreground_color, *background_color, TEXT_MENU_PREVIEW_LINE_HEIGHT, false, false);
    if(cont->num_text_buffer_chars == 0) {
        cont->next_x = TEXT_MENU_PREVIEW_X;
        cont->next_y = TEXT_MENU_PREVIEW_Y;
    } else {
        SPRITE_NODE* finalChar = cont->text_buffer_chars[cont->num_text_buffer_chars-1];
        cont->next_x = adv_x + finalChar->v->posX;
        cont->next_y = 240-finalChar->v->posY-14;
        if(cont->next_x > 320) {
            cont->next_x = TEXT_MENU_PREVIEW_X;
            cont->next_y += TEXT_MENU_PREVIEW_LINE_HEIGHT;
        }
    }
}

void _SETUP_text_input(void** context) {
    if(TEXT_ENTRY_BUFFER == NULL) return;

    struct _TEXT_INPUT_CONTEXT* cont = malloc(sizeof(struct _TEXT_INPUT_CONTEXT));
    *context = cont;
    cont->cursor[0] = NULL;
    cont->grid_chars[0] = NULL;
    cont->num_grid_chars = 0;
    cont->selection = 0;
    cont->charReg = UPPERCASE;
    cont->reg_tag = NULL;
    cont->text_buffer_chars = malloc(TEXT_ENTRY_BUFFER_LENGTH * sizeof(intptr_t));
    cont->num_text_buffer_chars = 0;

    set_font_size(14);
    _HELP_draw_grid(&cont->grid_chars[0], &cont->num_grid_chars, 0);
    _HELP_draw_cursor(cont);
    _HELP_draw_register_tag(cont->charReg, &cont->reg_tag);
    _HELP_draw_text_preview(cont);

    draw_all_sprites(spi);
}

void _CLEANUP_text_input(void** context) {
    if(TEXT_ENTRY_BUFFER == NULL) return;
    struct _TEXT_INPUT_CONTEXT* cont = *context;
    free(cont->text_buffer_chars);
    _CLEANUP_COMMON_single_layer_context(context);
}

int _ENC_text_input(void* context, rotary_encoder_event_t ev, void* args) {
    if(TEXT_ENTRY_BUFFER == NULL)
        return MENU_POP_FLAG;

    struct _TEXT_INPUT_CONTEXT* cont = context;
    int direction = 1;
    if(ev.state.direction != ROTARY_ENCODER_DIRECTION_CLOCKWISE)
        direction = TEXT_string_lengths[cont->charReg]-1;
    cont->selection = (cont->selection+direction) % (TEXT_string_lengths[cont->charReg]);
    _HELP_draw_cursor(cont);
    draw_all_sprites(spi);
    return 0;
}

int _BA_RD_switch_register(void* context, void* args) {
    if(TEXT_ENTRY_BUFFER == NULL)
        return MENU_POP_FLAG;
    struct _TEXT_INPUT_CONTEXT* cont = context;
    cont->charReg = (cont->charReg+1)%3;
    _HELP_draw_grid(&cont->grid_chars[0], &cont->num_grid_chars, cont->charReg);
    _HELP_draw_register_tag(cont->charReg, &cont->reg_tag);
    draw_all_sprites(spi);
    return 0;
}

int _BA_ED_accept_char(void* context, void* args) {
    if(TEXT_ENTRY_BUFFER == NULL)
        return MENU_POP_FLAG;

    struct _TEXT_INPUT_CONTEXT* cont = context;
    int numFromEnd = TEXT_string_lengths[cont->charReg] - cont->selection;
    int lenBuffer = strlen(TEXT_ENTRY_BUFFER);
    switch(numFromEnd) {
        case 1:
            return MENU_POP_FLAG;
            break;
        case 2:
            if(lenBuffer == 0)
                return 0;
            TEXT_ENTRY_BUFFER[lenBuffer-1] = 0;
            break;
        case 3:
            if(lenBuffer == TEXT_ENTRY_BUFFER_LENGTH-1)
                return 0;
            TEXT_ENTRY_BUFFER[lenBuffer] = ' ';
            TEXT_ENTRY_BUFFER[lenBuffer+1] = 0;
            break;
        default:
            if(lenBuffer == TEXT_ENTRY_BUFFER_LENGTH-1)
                return 0;
            char selected = TEXT_metatable[cont->charReg][cont->selection];
            TEXT_ENTRY_BUFFER[lenBuffer] = selected;
            TEXT_ENTRY_BUFFER[lenBuffer+1] = 0;
    }
    _HELP_draw_text_preview(cont);
    draw_all_sprites(spi);
    return 0;
}
// }}}
// SKIP WIFI CONNECTION CONFIRMATION {{{
// nothing here. Just transitioning to a few menus.
// }}}
// DISPLAY THEMING {{{
enum _DISPLAY_MENU_MODES {
    MAIN = 0,
    BRIGHTNESS,
    THEME
};
struct _CONTEXT_DISPLAY_MENU {
    unsigned char selection;
    SPRITE_NODE* brightness_sprites[4];
    int num_brightness_sprites;
    SPRITE_NODE* theming_sprites[40];
    int num_theming_sprites;
    SPRITE_NODE* aux_cursors[2];
};
const char* const* theme_names[] = {text_dark_mode, text_light_mode, text_custom};

void _SETUP_display_menu(void** context) {
    struct _CONTEXT_DISPLAY_MENU* ctx = malloc(sizeof(struct _CONTEXT_DISPLAY_MENU));
    *context = _HELP_COMMON_create_and_wrap_modal_context(ctx);
    ctx->selection = 0;

    if(COLOR_SELECTION_BUFFER_VALID) {
        settings.custom_theme_color.pixelR = COLOR_SELECTION_BUFFER.pixelR;
        settings.custom_theme_color.pixelG = COLOR_SELECTION_BUFFER.pixelG;
        settings.custom_theme_color.pixelB = COLOR_SELECTION_BUFFER.pixelB;
        COLOR_SELECTION_BUFFER_VALID = 0;
        assign_theme_from_settings();
        ctx->selection = 10; // trigger redraw
        //reload manually because the background did not change and thus did not trigger the manual reload
        load_bgimg(bgbuf, "/mainfs/pb_bg.cbi", true, 0);
        blit_bg();
        return;
    }

    int numsprs;
    SPRITE_NODE* sprs[32];
    char brightness[4];
    SPRITE_NODE* cursor;
    (void) itoa(settings.disp_brightness, brightness, 10);
    set_font_size(14);
    draw_text(0, 216, text_display_setting[settings.language], sprs, &numsprs, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);
    draw_text(32, 184, text_brightness[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(150, 184, brightness, &ctx->brightness_sprites[0], &ctx->num_brightness_sprites, *foreground_color, *background_color, 0, false, false);
    draw_text(150, 152, theme_names[settings.disp_theme][settings.language], &ctx->theming_sprites[0], &ctx->num_theming_sprites, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 152, text_theme[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(10, 184, ">", &cursor, &numsprs, *foreground_color, *background_color, 0, false, false);
    set_sprites_lifetime(1, &cursor, 1);
    draw_all_sprites(spi);
}

void _CLEANUP_display_menu(void** context) {
    struct _CONTEXT_DISPLAY_MENU* ctx = (struct _CONTEXT_DISPLAY_MENU*) _HELP_COMMON_unwrap_modal_context(*context);
    free(ctx);
    _CLEANUP_COMMON_single_layer_context(context);
}

int _POSTLOOP_display_menu(void* context, void* args) {
    struct _CONTEXT_DISPLAY_MENU* ctx = (struct _CONTEXT_DISPLAY_MENU*) _HELP_COMMON_unwrap_modal_context(context);
    if(ctx->selection == 10)
        return MENU_REDRAW_FLAG;
    return 0;
}

int _ENC_display_menu_main_mode(void* context, rotary_encoder_event_t ev, void* args, int* mode) {
    struct _CONTEXT_DISPLAY_MENU* ctx = (struct _CONTEXT_DISPLAY_MENU*) context;
    if(ctx->selection == 10)
        return MENU_REDRAW_FLAG;
    SPRITE_NODE* cursor;
    int numsprs = 0;
    ctx->selection ^= 1;
    draw_text(10, ctx->selection ? 152 : 184, ">", &cursor, &numsprs, *foreground_color, *background_color, 0, false, false);
    set_sprites_lifetime(1, &cursor, 1);
    draw_all_sprites(spi);
    return 0;
}

int _ENC_display_menu_bright_mode(void* context, rotary_encoder_event_t ev, void* args, int* mode) {
    struct _CONTEXT_DISPLAY_MENU* ctx = (struct _CONTEXT_DISPLAY_MENU*) context;
    if(ctx->selection == 10)
        return MENU_REDRAW_FLAG;
    char bright[4];
    for(int i=0;i<ctx->num_brightness_sprites;++i)
        delete_node(ctx->brightness_sprites[i]);
    settings.disp_brightness += ev.state.multiplier * ((ev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : -1);
    if(settings.disp_brightness > 255)
        settings.disp_brightness = 255;
    else if(settings.disp_brightness < 0)
        settings.disp_brightness = 0;
    (void) itoa(settings.disp_brightness, bright, 10);
    draw_text(150, 184, bright, &ctx->brightness_sprites[0], &ctx->num_brightness_sprites, HIGHLIGHT_COLOR, *background_color, 0, false, false);
    draw_all_sprites(spi);
    ledc_set_duty(LEDC_LOW_SPEED_MODE, 7, settings.disp_brightness << 6);
    ledc_update_duty(LEDC_LOW_SPEED_MODE, 7);
    return 0;
}

int _ENC_display_menu_theme_mode(void* context, rotary_encoder_event_t ev, void* args, int* mode) {
    struct _CONTEXT_DISPLAY_MENU* ctx = (struct _CONTEXT_DISPLAY_MENU*) context;
    if(ctx->selection == 10)
        return MENU_REDRAW_FLAG;
    for(int i=0;i<ctx->num_theming_sprites;++i)
        delete_node(ctx->theming_sprites[i]);
    settings.disp_theme = (settings.disp_theme + ((ev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 2)) % 3;
    draw_text(150, 152, theme_names[settings.disp_theme][settings.language], &ctx->theming_sprites[0], &ctx->num_theming_sprites, HIGHLIGHT_COLOR, *background_color, 0, false, false);
    draw_all_sprites(spi);
    return 0;
}

int _BA_ED_select_value_main_mode(void* context, void* args, int* mode) {
    struct _CONTEXT_DISPLAY_MENU* ctx = (struct _CONTEXT_DISPLAY_MENU*) context;
    if(ctx->selection == 10)
        return MENU_REDRAW_FLAG;
    *mode = ctx->selection + 1;
    switch(*mode) {
        case 1:
            for(int i=0;i<ctx->num_brightness_sprites;++i)
                ctx->brightness_sprites[i]->v->fg = HIGHLIGHT_COLOR;
            break;
        case 2:
            for(int i=0;i<ctx->num_theming_sprites;++i)
                ctx->theming_sprites[i]->v->fg = HIGHLIGHT_COLOR;
            break;
    }
    int height = ctx->selection ? 152 : 184;
    int num;
    draw_text(140, height, "<", &ctx->aux_cursors[0], &num, *foreground_color, *background_color, 0, false, false);
    draw_text(280, height, ">", &ctx->aux_cursors[1], &num, *foreground_color, *background_color, 0, false, false);
    draw_all_sprites(spi);
    return 0;
}

int _BA_ED_select_value_brightness_mode(void* context, void* args, int* mode) {
    struct _CONTEXT_DISPLAY_MENU* ctx = (struct _CONTEXT_DISPLAY_MENU*) context;
    if(ctx->selection == 10)
        return MENU_REDRAW_FLAG;
    for(int i=0;i<ctx->num_brightness_sprites;++i)
        ctx->brightness_sprites[i]->v->fg = *foreground_color;
    delete_node(ctx->aux_cursors[0]);
    delete_node(ctx->aux_cursors[1]);
    SPRITE_NODE* cursor;
    int numsprs;
    draw_text(10, 184, ">", &cursor, &numsprs, *foreground_color, *background_color, 0, false, false);
    set_sprites_lifetime(1, &cursor, 1);
    draw_all_sprites(spi);
    ctx->selection = 0;
    *mode = MAIN;
    return 0;
}

int _BA_ED_select_value_theme_mode(void* context, void* args, int* mode) {
    struct _CONTEXT_DISPLAY_MENU* ctx = (struct _CONTEXT_DISPLAY_MENU*) context;
    if(ctx->selection == 10)
        return MENU_REDRAW_FLAG;
    for(int i=0;i<ctx->num_theming_sprites;++i)
        ctx->theming_sprites[i]->v->fg = *foreground_color;
    delete_node(ctx->aux_cursors[0]);
    delete_node(ctx->aux_cursors[1]);
    if(settings.disp_theme == 2) {
        COLOR_SELECTION_BUFFER_VALID = 0;
        COLOR_SELECTION_BUFFER.pixelR = settings.custom_theme_color.pixelR;
        COLOR_SELECTION_BUFFER.pixelG = settings.custom_theme_color.pixelG;
        COLOR_SELECTION_BUFFER.pixelB = settings.custom_theme_color.pixelB;
        return 9;
    }
    ctx->selection = 1;
    *mode = MAIN;
    assign_theme_from_settings();
    //reload manually because the background did not change and thus did not trigger the manual reload
    load_bgimg(bgbuf, "/mainfs/pb_bg.cbi", true, 0);
    blit_bg();
    return MENU_REDRAW_FLAG;
}
// }}}
// COLOR PICKER MENU {{{
struct _COLOR_PICKER_CONTEXT {
    SPRITE_NODE* cursor;
    unsigned int selection;
    SPRITE_NODE* preview_rect;
    SPRITE_NODE* channel_chars[4][3];
    int num_channel_chars[3];
};

void _SETUP_color_picker_menu(void** context) {
    struct _COLOR_PICKER_CONTEXT* cont = malloc(sizeof(struct _COLOR_PICKER_CONTEXT));
    *context = _HELP_COMMON_create_and_wrap_modal_context(cont);
    cont->selection = 0;

    SPRITE_NODE* sprs[20];
    int numsprs;
    char numbuf[4];

    set_font_size(14);
    draw_text(0, 216, text_color_picker[settings.language], sprs, &numsprs,*foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(sprs, numsprs);

    draw_text(32, 184, text_red[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 152, text_green[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);
    draw_text(32, 120, text_blue[settings.language], NULL, NULL, *foreground_color, *background_color, 0, false, false);

    (void) itoa(COLOR_SELECTION_BUFFER.pixelR, numbuf, 10);
    draw_text(150, 184, numbuf, &cont->channel_chars[0][0], &cont->num_channel_chars[0], *foreground_color, *background_color, 0, false, false);
    (void) itoa(COLOR_SELECTION_BUFFER.pixelG, numbuf, 10);
    draw_text(150, 152, numbuf, &cont->channel_chars[1][0], &cont->num_channel_chars[1], *foreground_color, *background_color, 0, false, false);
    (void) itoa(COLOR_SELECTION_BUFFER.pixelB, numbuf, 10);
    draw_text(150, 120, numbuf, &cont->channel_chars[2][0], &cont->num_channel_chars[2], *foreground_color, *background_color, 0, false, false);

    draw_text(10, 184, ">", &cont->cursor, NULL, *foreground_color, *background_color, 0, false, false);
    cont->preview_rect = sprite_rectangle(128, 48, 64, 64, &COLOR_SELECTION_BUFFER, true, 255);
    draw_all_sprites(spi);
}

const int COLOR_ys[] = {184, 152, 120};
int _ENC_color_picker_main_mode(void* context, rotary_encoder_event_t ev, void* args, int* mode) {
    struct _COLOR_PICKER_CONTEXT* c = (struct _COLOR_PICKER_CONTEXT*) context;
    if(c->cursor != NULL)
        delete_node(c->cursor);
    c->selection = (c->selection + ((ev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 2)) % 3;
    draw_text(10, COLOR_ys[c->selection], ">", &c->cursor, NULL, *foreground_color, *background_color, 0, false, false);
    draw_all_sprites(spi);
    return 0;
}

int _ENC_color_picker_channel_mode(void* context, rotary_encoder_event_t ev, void* args, int* channel) {
    char numbuf[4];
    struct _COLOR_PICKER_CONTEXT* c = (struct _COLOR_PICKER_CONTEXT*) context;
    for(int i=0;i<c->num_channel_chars[*channel-1];++i)
        delete_node(c->channel_chars[*channel-1][i]);

    unsigned char channel_val = ((unsigned char*) &COLOR_SELECTION_BUFFER)[*channel-1];
    channel_val = (channel_val + ((ev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? ev.state.multiplier : 256 - ev.state.multiplier)) & 0xff;
    ((unsigned char*) &COLOR_SELECTION_BUFFER)[*channel-1] = channel_val;
    c->preview_rect->v->fg = COLOR_SELECTION_BUFFER;
    (void) itoa(channel_val, numbuf, 10);
    draw_text(150, COLOR_ys[*channel-1], numbuf, &c->channel_chars[*channel-1][0], &c->num_channel_chars[*channel-1], HIGHLIGHT_COLOR, *background_color, 0, false, false);
    draw_all_sprites(spi);
    return 0;
}

int _BA_ED_color_picker_select_focused_channel(void* context, void* args, int* mode) {
    struct _COLOR_PICKER_CONTEXT* c = (struct _COLOR_PICKER_CONTEXT*) context;
    for(int i=0;i<c->num_channel_chars[c->selection];++i)
        c->channel_chars[c->selection][i]->v->fg = HIGHLIGHT_COLOR;
    *mode = c->selection + 1;
    draw_all_sprites(spi);
    return 0;
}

int _BA_ED_color_picker_return_to_main_mode(void* context, void* args, int* channel) {
    struct _COLOR_PICKER_CONTEXT* c = (struct _COLOR_PICKER_CONTEXT*) context;
    for(int i=0;i<c->num_channel_chars[*channel-1];++i)
        c->channel_chars[*channel-1][i]->v->fg = *foreground_color;
    *channel = 0;
    draw_all_sprites(spi);
    return 0;
}

int _BA_RD_color_picker_confirm(void* context, void* args) {
    COLOR_SELECTION_BUFFER_VALID = 1;
    return MENU_POP_FLAG;
}
// }}}
// HTTP SERVER CONFIGURATION MENU {{{
void _SETUP_http_server_config(void** context) {
    system_flags &= ~FLAG_HTTP_SERVER_DONE;
    ESP_ERROR_CHECK(example_start_file_server("/mainfs"));
}

void _CLEANUP_http_server_config(void** context) {
    stop_file_server();
    _CLEANUP_COMMON_single_layer_context(context);
}

int _POSTLOOP_http_server_config(void* context, void* args) {
    if((system_flags & FLAG_HTTP_SERVER_DONE) != 0) {
        system_flags &= ~FLAG_HTTP_SERVER_DONE;
        ledc_set_duty(LEDC_LOW_SPEED_MODE, 7, settings.disp_brightness << 6);
        ledc_update_duty(LEDC_LOW_SPEED_MODE, 7);
        return MENU_SETUP_ONLY_TRANSITION_FLAG | 21;
    }
    return 0;
}
// }}}
// HOME MENU {{{
struct _CONTEXT_HOME_MENU {
    int selected_channel;
    int num_on_off_button_sprites;
    SPRITE_NODE* on_off_button_sprites[3];
    SPRITE_NODE* channel_label[4];
    int num_channel_on_off_label[4];
    SPRITE_NODE* channel_on_off_label[6][4];
    int num_output_percentage_main_sprites;
    SPRITE_NODE* output_percentage_main[4];
    int num_output_percentages_per_channel_sprites[4];
    SPRITE_NODE* output_percentages_per_channel[4][4];
    char channel_chars[7];
};
const char WIFI_CONNECTED_SYMBOL[] = "";
const char WIFI_DISCONNECTED_SYMBOL[] = "";
const char SERVER_CONNECTED_SYMBOL[] = "+";
const char SERVER_DISCONNECTED_SYMBOL[] = "-";
const int HOME_MENU_xs[] = {58, 111, 164, 217};
const int HOME_MENU_on_off_y = 16;
const int HOME_MENU_percentage_y = 4;
const int NUM_CHANNELS = 4;
void _SETUP_home_menu(void** context) {
    struct _CONTEXT_HOME_MENU* cont = malloc(sizeof(struct _CONTEXT_HOME_MENU));
    *context = _HELP_COMMON_create_and_wrap_modal_context(cont);
    set_font_size(12);
    cont->selected_channel = 0;

    const char* wifi_flag = (system_flags & FLAG_WIFI_CONNECTED) ? WIFI_CONNECTED_SYMBOL : WIFI_DISCONNECTED_SYMBOL;
    const char* server_flag = (system_flags & FLAG_SERVER_CONNECTED) ? SERVER_CONNECTED_SYMBOL : SERVER_DISCONNECTED_SYMBOL;
    int num;
    SPRITE_NODE* wifi_symbol;
    SPRITE_NODE* server_symbol;
    draw_text(306, 226, wifi_flag, &wifi_symbol, &num, *foreground_color, *background_color, 0, false, false);
    draw_text(294, 220, server_flag, &server_symbol, &num, *foreground_color, *background_color, 0, false, false);

    set_font_size(42);
    draw_text(0, 134, "0%", &cont->output_percentage_main[0], &cont->num_output_percentage_main_sprites, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(cont->output_percentage_main, cont->num_output_percentage_main_sprites);
    set_font_size(18);
    draw_text(2, 6, "OFF", &cont->on_off_button_sprites[0], &cont->num_on_off_button_sprites, *foreground_color, *background_color, 0, false, false);
    strncpy(cont->channel_chars, "CH1⤓", 7);
    draw_text(0, 36, &cont->channel_chars[0], &cont->channel_label[0], &num, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(cont->channel_label, 4);
    set_font_size(12);
    char on_off_text[6];
    for(int i=0;i<NUM_CHANNELS;++i) {
        draw_text(HOME_MENU_xs[i], HOME_MENU_percentage_y, "0%", &cont->output_percentages_per_channel[i][0], &cont->num_output_percentages_per_channel_sprites[i], *foreground_color, *background_color, 0, false, false);
        sprintf(on_off_text, "%d-On", i+1);
        draw_text(HOME_MENU_xs[i], HOME_MENU_on_off_y, on_off_text, &cont->channel_on_off_label[i][0], &cont->num_channel_on_off_label[i], *foreground_color, *background_color, 0, false, false);
    }
    wait_for_end_of_frame();
    draw_all_sprites(spi);
}

void _CLEANUP_home_menu(void** context) {
    free(_HELP_COMMON_unwrap_modal_context(*context));
    _CLEANUP_COMMON_single_layer_context(context);
}

void _HELP_HOME_MENU_update_percentages(struct _CONTEXT_HOME_MENU* cont) {
    for(int i=0;i<cont->num_output_percentage_main_sprites;++i)
        delete_node(cont->output_percentage_main[i]);
    for(int i=0;i<cont->num_output_percentages_per_channel_sprites[cont->selected_channel];++i)
        delete_node(cont->output_percentages_per_channel[cont->selected_channel][i]);
    set_font_size(42);
    int val = output_get_value(cont->selected_channel);
    char percentage[7];
    int k = sprintf(percentage, "%d%%", (uint16_t) val / 163);
    if(k > 4)
        assert(false);
    draw_text(0, 134, percentage, &cont->output_percentage_main[0], &cont->num_output_percentage_main_sprites, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(cont->output_percentage_main, cont->num_output_percentage_main_sprites);
    set_font_size(12);
    draw_text(HOME_MENU_xs[cont->selected_channel], HOME_MENU_percentage_y, percentage, &cont->output_percentages_per_channel[cont->selected_channel][0], &cont->num_output_percentages_per_channel_sprites[cont->selected_channel], *foreground_color, *background_color, 0, false, false);
}

void _HELP_HOME_MENU_update_on_off_button(struct _CONTEXT_HOME_MENU* cont) {
    for(int i=0;i<cont->num_on_off_button_sprites;++i)
        delete_node(cont->on_off_button_sprites[i]);
    for(int i=0;i<cont->num_channel_on_off_label[cont->selected_channel];++i)
        delete_node(cont->channel_on_off_label[cont->selected_channel][i]);
    set_font_size(18);
    draw_text(2, 6, is_off(cont->selected_channel) ? "ON" : "OFF", &cont->on_off_button_sprites[0], &cont->num_on_off_button_sprites, *foreground_color, *background_color, 0, false, false);
    set_font_size(12);
    char label[6];
    sprintf(label, "%d-%s", (cont->selected_channel+1) & 8, is_off(cont->selected_channel) ? "Off" : "On");
    draw_text(HOME_MENU_xs[cont->selected_channel], HOME_MENU_on_off_y, label, &cont->channel_on_off_label[cont->selected_channel][0], &cont->num_channel_on_off_label[0], *foreground_color, *background_color, 0, false, false);
}

int _ENC_home_menu_change_channel_value(void* context, rotary_encoder_event_t ev, void* args, int* mode) {
    struct _CONTEXT_HOME_MENU* cont = (struct _CONTEXT_HOME_MENU*) context;
    output_add_value(cont->selected_channel, (ev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 163 : -163);
    _HELP_HOME_MENU_update_percentages(cont);
    draw_all_sprites(spi);
    return 0;
}

int _ENC_home_menu_change_channel(void* context, rotary_encoder_event_t ev, void* args, int* mode) {
    struct _CONTEXT_HOME_MENU* cont = (struct _CONTEXT_HOME_MENU*) context;
    for(int i=0;i<4;++i)
        delete_node(cont->channel_label[i]);
    cont->selected_channel = (cont->selected_channel + ((ev.state.direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE) ? 1 : 3)) % 4;
    set_font_size(18);
    cont->channel_chars[2] = '1' + cont->selected_channel;
    int num;
    draw_text(0, 36, cont->channel_chars, &cont->channel_label[0], &num, *foreground_color, *background_color, 0, false, false);
    center_sprite_group_x(cont->channel_label, 4);
    draw_all_sprites(spi);
    return 0;
}

int _BA_LD_home_menu_toggle_channel(void* context, void* args) {
    struct _CONTEXT_HOME_MENU* cont = (struct _CONTEXT_HOME_MENU*) _HELP_COMMON_unwrap_modal_context(context);
    output_toggle(cont->selected_channel);
    _HELP_HOME_MENU_update_on_off_button(cont);
    draw_all_sprites(spi);
    return 0;
}

int _BA_ED_home_menu_switch_rotation_mode(void* context, void* args, int* mode) {
    *mode ^= 1;
    return 0;
}

void _HELP_HOME_MENU_parse_message_and_act(char* msg, int numbytes, struct _CONTEXT_HOME_MENU* cont) {
    char opcode = msg[0];
    int channel;
    uint16_t value;
    uint16_t time;
    switch(opcode) {
        case 1:
            if(numbytes != 6)
                return;
            value = ((msg[1] & 0x3f) << 8) | msg[2];
            channel = msg[3] & 0x3;
            time = ((msg[4] & 0x3f) << 8) | msg[5];
            if(time == 0)
                output_set_value(channel, value);
            else
                output_set_value_timeout(channel, value, time);
            _HELP_HOME_MENU_update_percentages(cont);
            break;
        case 3:
            int16_t ivalue = (msg[1] << 8) | msg[2];
            channel = msg[3] & 0x3;
            output_add_value(channel, ivalue);
            _HELP_HOME_MENU_update_percentages(cont);
            break;
        case 4:
            char flags = msg[1];
            channel = msg[2] & 0x3;
            if(flags & 0x80)
                output_toggle(channel);
            if(flags & 0x40)
                output_set_power(channel, 1);
            if(flags & 0x20)
                output_set_power(channel, 0);
            if(flags & 0x10) {
                char retmessg[3];
                int outval = output_get_value(channel);
                retmessg[0] = (outval << 8) & 0xff;
                retmessg[1] = outval & 0xff;
                retmessg[2] = is_off(channel) & 0x1;
                send_message(retmessg, 3);
            }
            _HELP_HOME_MENU_update_on_off_button(cont);
            break;
        case 8:
            time = (msg[1] << 8) | msg[2];
            button_disable_counter = 5*time;
            break;
        case 10:
            time = (msg[1] << 8) | msg[2];
            if(0xffff - button_disable_counter < 5*time) {
                button_disable_counter = 0xffff;
            } else if(button_disable_counter < -5*time) {
                button_disable_counter = 0;
            } else {
                button_disable_counter += time;
            }
            break;
        default:
            ets_printf("unknown opcode %x\n", opcode);
            break;
    }
}

int _POSTLOOP_home_menu(void* context, void* args) {
    struct _CONTEXT_HOME_MENU* cont = (struct _CONTEXT_HOME_MENU*) _HELP_COMMON_unwrap_modal_context(context);
    char buf[256];
    memset(buf, 0, 256);
    int len = get_message(buf, 255);
    if(len > 0)
        _HELP_HOME_MENU_parse_message_and_act(buf, len, cont);
    return 0;
}
// }}}
// SERVER SETTINGS {{{
void _SETUP_server_settings(void** context) {
    set_font_size(14);
}

void _CLEANUP_server_settings(void** context) {
    
}
// }}}
// SETTINGS OPTIONS {{{
void _SETUP_settings_options(void** context) {
    struct _CONTEXT_settings_menu* ctx = malloc(sizeof(struct _CONTEXT_settings_menu));
    ctx->options_strings[0] = text_settings_display[settings.language],
    ctx->options_strings[1] = text_settings_network[settings.language],
    ctx->options_strings[2] = text_settings_output[settings.language],
    ctx->options_strings[3] = text_settings_rgb[settings.language],
    ctx->options_strings[4] = text_settings_add_ons[settings.language],
    ctx->options_strings[5] = text_settings_apps[settings.language],
    ctx->options_strings[6] = text_language[settings.language],
    ctx->options_strings[7] = text_settings_developer[settings.language],
    ctx->opt.currentOption = 0;
    ctx->opt.numOptions = 8;
    ctx->opt.options = ctx->options_strings;
    setup_cursor(&ctx->opt.cursorbg, &ctx->opt.cursor, OPTION_Ys[0]);
    _HELP_COMMON_draw_options(&ctx->opt, 0);
    *context = ctx;
}

const int menu_table[] = {
    8, 19, 12, 14, 10, 15, 1, 23
};
int _BA_ED_settings_options_select_menu(void* context, void* args) {
    struct _CONTEXT_settings_menu* ctx = (struct _CONTEXT_settings_menu*) context;
    return menu_table[ctx->opt.currentOption];
}

// }}}

// vim:fdm=marker
