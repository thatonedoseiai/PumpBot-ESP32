#ifndef MENU_FUNCTIONS_H
#define MENU_FUNCTIONS_H

#include "rotenc.h"
#include "oam.h"
#include "esp_wifi.h"

#define MENU_RETURN_FLAG 0x8000
#define MENU_POP_FLAG 0x4000
#define MENU_REDRAW_FLAG 0x2000
#define MENU_SETUP_ONLY_TRANSITION_FLAG 0x1000
#define MENU_SELF_POP_FLAG 0x800

#define NUM_WIFIS 10
struct _OPTIONS_DATA_ {
    int currentOption;
    int numOptions;
    SPRITE_NODE* cursor;
    SPRITE_NODE* cursorbg;
    char** options;
};
struct _WIFI_MENU_CONTEXT {
    wifi_ap_record_t ap_info[NUM_WIFIS];
    struct _OPTIONS_DATA_ opt;
};

// int _RA_welcome_menu(void** context, rotary_encoder_event_t, void* args);
int _BA_COMMON_go_to_menu(void* context, void* args);
void _SETUP_COMMON_no_context(void** context);
void _CLEANUP_COMMON_single_layer_context(void** context);
int _ENC_COMMON_scroll_options(void* context, rotary_encoder_event_t ev, void* args);

void _SETUP_welcome_menu(void** context);
// "button action, encoder down"
int _BA_ED_welcome_menu(void* context, void* args);
int _POSTLOOP_welcome_menu(void* context, void* args);

void _SETUP_setup_menu(void** context);
int _BA_RD_setup_menu_confirm(void* context, void* args);
int _BA_ENC_setup_menu(void* context, rotary_encoder_event_t ev, void* args);

void _SETUP_wifi_menu(void** context);
void _CLEANUP_wifi_menu(void** context);
int _BA_ED_wifi_menu_set_wifi_name(void* context, void* args);
int _BA_RD_wifi_menu_reload(void* context, void* args);

void _SETUP_pb_setup_method(void** context);
int _ENC_pb_setup_method(void* context, rotary_encoder_event_t ev, void* args);
int _BA_pb_setup_method_confirm(void* context, void* args);

void _SETUP_wifi_connect(void** context);
int _BA_LD_wifi_connect(void* context, void* args);
int _POSTLOOP_wifi_connect(void* context, void* args);

void _SETUP_wifi_preview(void** context);
int _BA_LD_wifi_preview(void* context, void* args);
int _BA_ED_wifi_preview(void* context, void* args);
int _ENC_wifi_preview(void* context, rotary_encoder_event_t ev, void* args);

void _SETUP_text_input(void** context);
void _CLEANUP_text_input(void** context);
int _ENC_text_input(void* context, rotary_encoder_event_t ev, void* args);
int _BA_RD_switch_register(void* context, void* args);
int _BA_ED_accept_char(void* context, void* args);

#endif
