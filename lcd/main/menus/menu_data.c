#include "menus.h"
#include "lang.h"

extern uint24_RGB* foreground_color;
extern uint24_RGB* background_color;

const MENU_ELEMENT welcome_menu[] = {
    {
        .text = text_welcome,
        .x = 60,
        .y = 195,
        // .flags = MENU_FLAG_CENTER,
        .flags = MENU_FLAG_CENTER,
        .textsize = 24,
        .col = &foreground_color
    },
    {
        .text = text_welcome_a,
        .x = 60,
        .y = 154,
        .flags = MENU_FLAG_CENTER,
        .textsize = 24,
        .col = &foreground_color
    },
    {
        .text = text_pressenc,
        .x = 60,
        .y = 10,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
};

const char zeroper[] = "0%";
const char rotateenc[] = "⟳";
const char ch0[] = "CH0⤓";
const char off[] = "OFF";
const char set[] = "│ ⚙";
const char aon[] = "0-On";
const char bon[] = "1-On";
const char con[] = "2-On";
const char don[] = "3-On";

const char* const zeroperp[] = {zeroper};
const char* const rotateencp[] = {rotateenc};
const char* const ch0p[] = {ch0};
const char* const offp[] = {off};
const char* const setp[] = {set};
const char* const aonp[] = {aon};
const char* const bonp[] = {bon};
const char* const conp[] = {con};
const char* const donp[] = {don};
const MENU_ELEMENT menuhome[] = {
    {
        .text = zeroperp, //Percentage of current channel
        .x = 0,
        .y = 134,
        .flags = MENU_FLAG_CENTER | MENU_FLAG_LANGUAGE_AGNOSTIC,
        .textsize = 42, // size H, "Heccin Chonker"
        .col = &foreground_color
    },
    {
        .text = rotateencp, //static symbol
        .x = 0,
        .y = 114,
        .flags = MENU_FLAG_CENTER | MENU_FLAG_LANGUAGE_AGNOSTIC,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .text = ch0p, //X = current channel
        .x = 0,
        .y = 36,
        .flags = MENU_FLAG_CENTER | MENU_FLAG_LANGUAGE_AGNOSTIC,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .text = offp, //OFF or ON for current channel
        .x = 2,
        .y = 2,
        .textsize = 18,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    {
        .text = setp, //Settings menu, will be custom icon later
        .x = 2,
        .y = 2,
        .textsize = 22,
        .flags = MENU_FLAG_RIGHT_JUSTIFY | MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    {
        .text = aonp,
        .x = 58,
        .y = 16,
        .textsize = 12,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    { 
        .text = zeroperp,
        .x = 58,
        .y = 2,
        .textsize = 12,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    {
        .text = bonp,
        .x = 111,
        .y = 16,
        .textsize = 12,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    { 
        .text = zeroperp,
        .x = 111,
        .y = 2,
        .textsize = 12,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
        {
        .text = conp,
        .x = 164,
        .y = 16,
        .textsize = 12,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    { 
        .text = zeroperp,
        .x = 164,
        .y = 2,
        .textsize = 12,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    {
        .text = donp,
        .x = 217,
        .y = 16,
        .textsize = 12,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    { 
        .text = zeroperp,
        .x = 217,
        .y = 2,
        .textsize = 12,
        .flags = MENU_FLAG_LANGUAGE_AGNOSTIC,
        .col = &foreground_color
    },
    { //Horizontal Line
        .y = 32,
        .textsize = 2,
        .col = &foreground_color,
        // .hline = true
        .flags = MENU_FLAG_IS_HLINE
    }
};

const MENU_ELEMENT menusetup0[] = {
    {
        .text = text_choose_lang,
        .x = 0,
        .y = 204,
        .flags = MENU_FLAG_CENTER,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .y = 176,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .text = text_language, 
        .x = 32,
        .y = 152,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_language_name,
        .x = 220,
        .y = 152,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
    { 
        .text = text_next, // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        // .x = 275,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .flags = MENU_FLAG_RIGHT_JUSTIFY,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menusetup1[] = {
    {
        .text = text_setup_pb,
        .x = 0,
        .y = 204,
        .flags = MENU_FLAG_CENTER,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .text = text_setup_pb_a,
        .x = 0,
        .y = 176,
        .flags = MENU_FLAG_CENTER,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .text = text_wifi_setup, 
        .x = 0,
        .y = 120,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_standalone_setup,
        .x = 0,
        .y = 88,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24, //24
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
    { 
        .text = text_ok, // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        // .x = 293,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .flags = MENU_FLAG_RIGHT_JUSTIFY,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menuwifistarting[] = { //Setup Menu 2
    {
        .text = text_connecting, 
        .x = 0,
        .y = 120,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_cancel,
        .x = 0,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
};

const char* const wifi_name = "pumpy wifi";
const char* const url_name = "192.168.4.1/index.html";
const MENU_ELEMENT menusetup2a[] = {
    {
        .text = text_connect_here,
        .x = 0,
        .y = 216, // yes this is 216, moved it up 12px to align with 32px grid for continuity with screen 3
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_connect_here_a,
        .x = 0,
        .y = 194,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = &wifi_name,
        .x = 0,
        .y = 152,
        .flags = MENU_FLAG_CENTER | MENU_FLAG_LANGUAGE_AGNOSTIC,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_go_to_url, 
        .x = 0,
        .y = 120,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_go_to_url_a, 
        .x = 0,
        .y = 88,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = &url_name,
        .x = 0,
        .y = 56,
        .flags = MENU_FLAG_CENTER | MENU_FLAG_LANGUAGE_AGNOSTIC,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 176,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
    // { 
    //     .text = text_next, // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
    //     // .x = 275,
    //     .x = 2,
    //     .y = 2,
    //     .textsize = 14,
    //     .flags = MENU_FLAG_RIGHT_JUSTIFY,
    //     .col = &foreground_color
    // }
};

const MENU_ELEMENT menusetup3[] = { //Setup Menu 3, basically a template for Setting menu and the likes
    {
        .text = text_wifi_settings,
        .x = 0,
        .y = 216,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 208,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 176,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 48, //last line on the screen, would get overwritten if there was a tooltip
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
    { 
        .text = text_next, // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        // .x = 275,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .flags = MENU_FLAG_RIGHT_JUSTIFY,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menutextenter[] = {
    {
        .text = text_enter_text,
        .x = 0,
        .y = 216,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 208,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 24,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menusetupdone[] = {
       {
        .text = text_setup_complete,
        .x = 0,
        .y = 195,
        .flags = MENU_FLAG_CENTER,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .text = text_pressenc,
        .x = 0,
        .y = 10,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
};

const MENU_ELEMENT menuapprundelete[] = {
    {
        .text = text_app_manage,
        .x = 0,
        .y = 204,
        .flags = MENU_FLAG_CENTER,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .text = text_app_run, 
        .x = 0,
        .y = 120,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_app_del,
        .x = 0,
        .y = 88,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_app_default,
        .x = 0,
        .y = 56,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 48,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24, //24
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
};

const MENU_ELEMENT menudownloadapp[] = {
    {
        .text = text_app_dl_from_url,
        .x = 0,
        .y = 204,
        .flags = MENU_FLAG_CENTER,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .y = 176,
        .textsize = 2,
        // .hline = true,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .text = text_app_url, 
        .x = 32,
        .y = 152,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
    { 
        .text = text_app_go, 
        .x = 2,
        .y = 2,
        .textsize = 14,
        .flags = MENU_FLAG_RIGHT_JUSTIFY,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menuserversettings[] = {
    {
        .text = text_server_settings,
        .x = 0,
        .y = 216,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_ip_addr,
        .x = 32,
        .y = 184,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_password, 
        .x = 32,
        .y = 152,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_port, 
        .x = 32,
        .y = 120,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 208,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 176,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
    { 
        .text = text_next, // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        // .x = 275,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .flags = MENU_FLAG_RIGHT_JUSTIFY,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menunetworksettings[] = {
    {
        .text = text_network_settings,
        .x = 0,
        .y = 204,
        .flags = MENU_FLAG_CENTER,
        .textsize = 18,
        .col = &foreground_color
    },
    {
        .text = text_wifi_settings, 
        .x = 32,
        .y = 120,
        .textsize = 14,
        .col = &foreground_color
    },
    {
        .text = text_server_settings,
        .x = 32,
        .y = 88,
        .flags = MENU_FLAG_CENTER,
        .textsize = 14,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    {
        .y = 48,
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24, //24
        .textsize = 2,
        .flags = MENU_FLAG_IS_HLINE,
        .col = &foreground_color
    },
    { 
        .text = text_back,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menuskipwifi[] = {
    {
        .text = text_wifi_continue_without,
        .x = 0,
        .y = 195,
        .textsize = 18,
        .flags = MENU_FLAG_CENTER,
        .col = &foreground_color
    },
    {
        .text = text_wifi_continue_without_a,
        .x = 0,
        .y = 154,
        .textsize = 18,
        .flags = MENU_FLAG_CENTER,
        .col = &foreground_color
    },
    // {
    //     .text = text_wifi_continue_without_b,
    //     .x = 0,
    //     .y = 113,
    //     .textsize = 18,
    //     .flags = MENU_FLAG_CENTER,
    //     .col = &foreground_color
    // },
    { 
        .text = text_no,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .col = &foreground_color
    },
    { 
        .text = text_ok,
        .x = 2,
        .y = 2,
        .textsize = 14,
        .flags = MENU_FLAG_RIGHT_JUSTIFY,
        .col = &foreground_color
    }
};

// const MENU_ELEMENT text_test[] = {
//     {
//         .x = 160,
//         .textsize = 2,
//         .col = &foreground_color,
//         // .vline = true
//         .flags = MENU_FLAG_IS_VLINE
//     },
//     {
//         .text = "Large - 24px",
//         .x = 0,
//         .y = 218,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {
//         .text = "Medium - 18px",
//         .x = 0,
//         .y = 152,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {
//         .text = "Standard - 14px",
//         .x = 0,
//         .y = 92,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {
//         .text = "Small - 12px",
//         .x = 0,
//         .y = 40,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {
//         .text = "28px",
//         .x = 162,
//         .y = 218,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {
//         .text = "24px",
//         .x = 162,
//         .y = 152,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {
//         .text = "18px",
//         .x = 162,
//         .y = 92,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {
//         .text = "14px",
//         .x = 162,
//         .y = 40,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {
//         .text = "A Quick Br",
//         .x = 0,
//         .y = 173,
//         .textsize = 24,
//         .col = &foreground_color
//     },
//     {
//         .text = "own Fox Jumps",
//         .x = 0,
//         .y = 113,
//         .textsize = 18,
//         .col = &foreground_color
//     },
//     {
//         .text = "over the lazy dog.",
//         .x = 0,
//         .y = 61,
//         .textsize = 14,
//         .col = &foreground_color
//     },
//     {
//         .text = "that dog was very lazy",
//         .x = 0,
//         .y = 13,
//         .textsize = 12,
//         .col = &foreground_color
//     },
//     {//sample text in chinese
//         .text = "鑑於人類",
//         .x = 162,
//         .y = 173,
//         .textsize = 28,
//         .col = &foreground_color
//     },
//     {
//         .text = "社會個成員",
//         .x = 162,
//         .y = 113,
//         .textsize = 24,
//         .col = &foreground_color
//     },
//     {
//         .text = "儕有個固有尊",
//         .x = 162,
//         .y = 61,
//         .textsize = 18,
//         .col = &foreground_color
//     },
//     {
//         .text = "嚴脫仔平等個脫仔",
//         .x = 162,
//         .y = 13,
//         .textsize = 14,
//         .col = &foreground_color
//     }
// };
