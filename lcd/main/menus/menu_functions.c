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

extern spi_device_handle_t spi;
extern uint24_RGB* background_color;
extern uint24_RGB* foreground_color;
extern SETTINGS_t settings;

// HELPERS {{{
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

void _CLEANUP_COMMON_single_layer_context(void** context) {
    // delete_persistent_sprites();
    delete_all_sprites_immediate();
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
    } else if(opt->currentOption > opt->numOptions-2) {
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
// WIFI MENU {{{
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

// vim:fdm=marker
