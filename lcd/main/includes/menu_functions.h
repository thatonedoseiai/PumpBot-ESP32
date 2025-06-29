#ifndef MENU_FUNCTIONS_H
#define MENU_FUNCTIONS_H

#include "rotenc.h"

#define MENU_RETURN_FLAG 0x8000
#define MENU_POP_FLAG 0x4000
#define MENU_REDRAW_FLAG 0x2000
#define MENU_SETUP_ONLY_TRANSITION_FLAG 0x1000
#define MENU_SELF_POP_FLAG 0x800

// int _RA_welcome_menu(void** context, rotary_encoder_event_t, void* args);
void _SETUP_welcome_menu(void** context);
void _CLEANUP_welcome_menu(void** context);
// "button action, encoder down"
int _BA_ED_welcome_menu(void* context, void* args);
int _POSTLOOP_welcome_menu(void* context, void* args);

#endif