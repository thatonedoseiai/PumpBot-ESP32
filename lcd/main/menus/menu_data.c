#include "menus.h"

extern uint24_RGB* foreground_color;
extern uint24_RGB* background_color;

const MENU_ELEMENT welcome_menu[] = {
    {
        .text = "Welcome to",
        .x = 60,
        .y = 195,
        .center = true,
        .textsize = 24,
        .col = &foreground_color
    },
    {
        .text = "PumpBot!",
        .x = 60,
        .y = 154,
        .center = true,
        .textsize = 24,
        .col = &foreground_color
    },
    {
        .text = "Press ⤓ to Continue",
        .x = 60,
        .y = 10,
        .center = true,
        .textsize = 14,
        .col = &foreground_color
    },
};

const MENU_ELEMENT text_test[] = {
    {
        .x = 160,
        .textsize = 2,
        .col = &foreground_color,
        .vline = true
    },
    {
        .text = "Large - 24px",
        .x = 0,
        .y = 218,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "Medium - 18px",
        .x = 0,
        .y = 152,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "Standard - 14px",
        .x = 0,
        .y = 92,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "Small - 12px",
        .x = 0,
        .y = 40,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "28px",
        .x = 162,
        .y = 218,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "24px",
        .x = 162,
        .y = 152,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "18px",
        .x = 162,
        .y = 92,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "14px",
        .x = 162,
        .y = 40,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "A Quick Br",
        .x = 0,
        .y = 173,
        .center = false,
        .textsize = 24,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "own Fox Jumps",
        .x = 0,
        .y = 113,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "over the lazy dog.",
        .x = 0,
        .y = 61,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "that dog was very lazy",
        .x = 0,
        .y = 13,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {//sample text in chinese
        .text = "鑑於人類",
        .x = 162,
        .y = 173,
        .center = false,
        .textsize = 28,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "社會個成員",
        .x = 162,
        .y = 113,
        .center = false,
        .textsize = 24,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "儕有個固有尊",
        .x = 162,
        .y = 61,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "嚴脫仔平等個脫仔",
        .x = 162,
        .y = 13,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menuhome[] = {
    {
        .text = "XXX%", //Percentage of current channel
        .x = 0,
        .y = 134,
        .center = true,
        .textsize = 42, // size H, "Heccin Chonker"
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "⟳", //static symbol
        .x = 0,
        .y = 114,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "CHX⤓", //X = current channel
        .x = 0,
        .y = 36,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "OXX", //OFF or ON for current channel
        .x = 2,
        .y = 2,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "SET", //Settings menu, will be custom icon later
        .x = 271,
        .y = 2,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "0-Oxx",
        .x = 58,
        .y = 16,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    { 
        .text = "XXX%",
        .x = 58,
        .y = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "1-Oxx",
        .x = 111,
        .y = 16,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    { 
        .text = "XXX%",
        .x = 111,
        .y = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
        {
        .text = "2-Oxx",
        .x = 164,
        .y = 16,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    { 
        .text = "XXX%",
        .x = 164,
        .y = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "3-Oxx",
        .x = 224,
        .y = 16,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    { 
        .text = "XXX%",
        .x = 224,
        .y = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = &foreground_color
    },
    { //Horizontal Line
        .y = 32,
        .textsize = 2,
        .col = &foreground_color,
        .hline = true
    }
};


const MENU_ELEMENT menusetup0[] = {
    {
        .text = "Choose your Language",
        .x = 0,
        .y = 204,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .y = 176,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .text = "Language", 
        .x = 32,
        .y = 152,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "English", //Really would help with right justification!!! Please!!!
        .x = 220,
        .y = 152,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { 
        .text = "Back",
        .x = 2,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    { 
        .text = "Next", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 275,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    }
};
const MENU_ELEMENT menusetup1[] = {
    {
        .text = "How would you like to",
        .x = 0,
        .y = 204,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "setup PumpBot?",
        .x = 0,
        .y = 176,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "Wi-Fi Setup", 
        .x = 0,
        .y = 120,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "Standalone Setup",
        .x = 0,
        .y = 88,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24, //24
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { 
        .text = "Back",
        .x = 2,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    { 
        .text = "OK", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 293,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    }
};
const MENU_ELEMENT menuwifistarting[] = { //Setup Menu 2
    {
        .text = "Starting Wi-Fi Network...", 
        .x = 0,
        .y = 120,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { 
        .text = "Cancel",
        .x = 0,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
};
const MENU_ELEMENT menusetup2a[] = {
    {
        .text = "Connect to this Wi-Fi Network",
        .x = 0,
        .y = 216, // yes this is 216, moved it up 12px to align with 32px grid for continuity with screen 3
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
        {
        .text = "to setup PumpBot",
        .x = 0,
        .y = 194,
        .center = true, 
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "pumpy wifi",
        .x = 0,
        .y = 152,
        .center = true, 
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "Open this URL in your browser:", 
        .x = 0,
        .y = 120,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "http://192.168.1.4/",
        .x = 0,
        .y = 88,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    {      
        .y = 176,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { 
        .text = "Back",
        .x = 2,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    { 
        .text = "Next", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 275,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    }
};
const MENU_ELEMENT menusetup3[] = { //Setup Menu 3, basically a template for Setting menu and the likes
    {
        .text = "Wi-Fi Settings",
        .x = 0,
        .y = 216,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    // {
    //     .text = "[Text goes here]",
    //     .x = 0,
    //     .y = 184,
    //     .textlen = 16,
    //     .numspaces = 2,
    //     .center = true,
    //     .textsize = 14,
    //     .hline = false,
    //     .col = &foreground_color
    // },
    // {
    //     .text = "[Text goes here]", 
    //     .x = 0,
    //     .y = 152,
    //     //.textlen = 16,
    //     //.numspaces = 2,
    //     .center = true,
    //     .textsize = 14,
    //     .hline = false,
    //     .col = &foreground_color
    // },
    // {
    //     .text = "[Text goes here]", 
    //     .x = 0,
    //     .y = 120,
    //     //.textlen = 16,
    //     //.numspaces = 2,
    //     .center = true,
    //     .textsize = 14,
    //     .hline = false,
    //     .col = &foreground_color
    // },
    // {
    //     .text = "[Text goes here]", 
    //     .x = 0,
    //     .y = 88,
    //     //.textlen = 16,
    //     //.numspaces = 2,
    //     .center = true,
    //     .textsize = 14,
    //     .hline = false,
    //     .col = &foreground_color
    // },
    // {
    //     .text = "[Text goes here]", //last option on the screen, would get overwritten with a tooltip
    //     .x = 0,
    //     .y = 56,
    //     //.textlen = 16,
    //     //.numspaces = 2,
    //     .center = true,
    //     .textsize = 14,
    //     .hline = false,
    //     .col = &foreground_color
    // },
    {      
        .y = 208,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .y = 176,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {      
        .y = 144,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .y = 112,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .y = 80,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    {
        .y = 48, //last line on the screen, would get overwritten if there was a tooltip
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .hline = true,
        .col = &foreground_color
    },
    { 
        .text = "Back",
        .x = 2,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
    { 
        .text = "Next", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 275,
        .y = 2,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    }
};

const MENU_ELEMENT menusetupdone[] = {
       {
        .text = "Setup Complete!",
        .x = 0,
        .y = 224,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = &foreground_color
    },
    {
        .text = "Press ⤓ to Continue",
        .x = 0,
        .y = 32,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = &foreground_color
    },
};

