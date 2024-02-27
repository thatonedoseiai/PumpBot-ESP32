#ifndef MENUDATA_H
#define MENUDATA_H

#pragma once

const MENU_ELEMENT welcome_menu[] = {
    {
        .text = "Welcome to",
        .x = 60,
        .y = 195,
        .textlen = 10,
        .numspaces = 1,
        .center = true,
        .textsize = 24,
        .col = { .pixelR = 255, .pixelG = 255, .pixelB = 255 }
    },
    {
        .text = "PumpBot!",
        .x = 60,
        .y = 154,
        .textlen = 8,
        .numspaces = 0,
        .center = true,
        .textsize = 24,
        .col = { .pixelR = 255, .pixelG = 255, .pixelB = 255 }
    },
    {
        .text = "Press v to Continue",
        .x = 60,
        .y = 10,
        .textlen = 19,
        .numspaces = 3,
        .center = true,
        .textsize = 14,
        .col = { .pixelR = 255, .pixelG = 255, .pixelB = 255 }
    },
};

const MENU_ELEMENT text_test[] = {
    {
        .x = 160,
        .textsize = 2,
        .col = { .pixelR = 127, .pixelB = 127, .pixelG = 127},
        .vline = true
    },
    {
        .text = "Large - 24px",
        .x = 0,
        .y = 218,
        .textlen = 12,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Medium - 18px",
        .x = 0,
        .y = 152,
        .textlen = 13,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Standard - 14px",
        .x = 0,
        .y = 92,
        .textlen = 15,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Small - 12px",
        .x = 0,
        .y = 40,
        .textlen = 12,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "28px",
        .x = 162,
        .y = 218,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "24px",
        .x = 162,
        .y = 152,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "18px",
        .x = 162,
        .y = 92,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "14px",
        .x = 162,
        .y = 40,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "A Quick Br",
        .x = 0,
        .y = 173,
        .textlen = 10,
        .numspaces = 2,
        .center = false,
        .textsize = 24,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }

    },
    {
        .text = "own Fox Jumps",
        .x = 0,
        .y = 113,
        .textlen = 13,
        .numspaces = 2,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    
    },
    {
        .text = "over the lazy dog.",
        .x = 0,
        .y = 61,
        .textlen = 18,
        .numspaces = 3,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    
    },
    {
        .text = "that dog was very lazy",
        .x = 0,
        .y = 13,
        .textlen = 22,
        .numspaces = 4,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {//sample text in chinese
        .text = "鑑於人類",
        .x = 162,
        .y = 173,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 28,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "社會個成員",
        .x = 162,
        .y = 113,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 24,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "儕有個固有尊",
        .x = 162,
        .y = 61,
        .textlen = 6,
        .numspaces = 0,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "嚴脫仔平等個脫仔",
        .x = 162,
        .y = 13,
        .textlen = 8,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    }
};

const MENU_ELEMENT menuhome[] = {
    {
        .text = "XXX%", //Percentage of current channel
        .x = 0,
        .y = 134,
        .textlen = 4,
        .numspaces = 0,
        .center = true,
        .textsize = 42, // size H, "Heccin Chonker"
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "⟳", //static symbol
        .x = 0,
        .y = 114,
        .textlen = 1,
        .numspaces = 0,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "CHX⤓", //X = current channel
        .x = 0,
        .y = 32,
        .textlen = 4,
        .numspaces = 0,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "OXX", //OFF or ON for current channel
        .x = 2,
        .y = 2,
        .textlen = 3,
        .numspaces = 0,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "SET", //Settings menu, will be custom icon later
        .x = 271,
        .y = 2,
        .textlen = 3,
        .numspaces = 0,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "0-Oxx",
        .x = 58,
        .y = 16,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "XXX%",
        .x = 58,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "1-Oxx",
        .x = 111,
        .y = 16,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "XXX%",
        .x = 111,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "2-Oxx",
        .x = 164,
        .y = 16,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "XXX%",
        .x = 164,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "3-Oxx",
        .x = 216,
        .y = 16,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "XXX%",
        .x = 216,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Horizontal Line
        .y = 210,
        .textsize = 2,
        .col = { .pixelR = 127, .pixelB = 127, .pixelG = 127},
        .hline = true
    }
};


const MENU_ELEMENT menusetup0[] = {
    {
        .text = "Choose your Language",
        .x = 0,
        .y = 204,
        .textlen = 20,
        .numspaces = 2,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 64,
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Language", 
        .x = 32,
        .y = 152,
        .textlen = 8,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "English", //Really would help with right justification!!! Please!!!
        .x = 220,
        .y = 152,
        .textlen = 7,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {      
        .y = 96,
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Bottom menu
        .y = 216,
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Back",
        .x = 0,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Next", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 275,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    }
};
const MENU_ELEMENT menusetup1[] = {
    {
        .text = "How would you like to",
        .x = 0,
        .y = 204,
        .textlen = 21,
        .numspaces = 4,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "setup PumpBot?",
        .x = 0,
        .y = 176,
        .textlen = 14,
        .numspaces = 1,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Wi-Fi Setup", 
        .x = 0,
        .y = 120,
        .textlen = 11,
        .numspaces = 1,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Standalone Setup",
        .x = 0,
        .y = 88,
        .textlen = 16,
        .numspaces = 1,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {      
        .y = 96, //144
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 128, //112
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 160, //80
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Bottom menu
        .y = 216, //24
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Back",
        .x = 0,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "OK", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 293,
        .y = 2,
        .textlen = 2,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    }
};
const MENU_ELEMENT menuwifistarting[] = { //Setup Menu 2
    {
        .text = "Starting Wi-Fi Network...", 
        .x = 0,
        .y = 120,
        .textlen = 25,
        .numspaces = 2,
        .center = false, //true
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Bottom menu
        .y = 216, //24
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Cancel",
        .x = 0,
        .y = 2,
        .textlen = 6,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
};
const MENU_ELEMENT menusetup2a[] = {
    {
        .text = "Connect to this Wi-Fi Network",
        .x = 0,
        .y = 216, // yes this is 216, moved it up 12px to align with 32px grid for continuity with screen 3
        .textlen = 29,
        .numspaces = 4,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "to setup PumpBot",
        .x = 0,
        .y = 194,
        .textlen = 16,
        .numspaces = 2,
        .center = true, 
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "[Network Name]",
        .x = 0,
        .y = 152,
        .textlen = 14,
        .numspaces = 1,
        .center = true, 
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Open this URL in your browser:", 
        .x = 0,
        .y = 120,
        .textlen = 30,
        .numspaces = 5,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "[URL]",
        .x = 0,
        .y = 88,
        .textlen = 5,
        .numspaces = 0,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {      
        .y = 64, //176
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {      
        .y = 96, //144
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 128, //112
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 160, //80
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Bottom menu
        .y = 216, //24
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Back",
        .x = 0,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Next", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 275,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    }
};
const MENU_ELEMENT menusetup3[] = { //Setup Menu 3, basically a template for Setting menu and the likes
    {
        .text = "Wi-Fi Settings",
        .x = 0,
        .y = 216,
        .textlen = 14,
        .numspaces = 1,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "[Text goes here]",
        .x = 0,
        .y = 184,
        .textlen = 16,
        .numspaces = 2,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "[Text goes here]", 
        .x = 0,
        .y = 152,
        .textlen = 16,
        .numspaces = 2,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "[Text goes here]", 
        .x = 0,
        .y = 120,
        .textlen = 16,
        .numspaces = 2,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "[Text goes here]", 
        .x = 0,
        .y = 88,
        .textlen = 16,
        .numspaces = 2,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "[Text goes here]", //last option on the screen, would get overwritten with a tooltip
        .x = 0,
        .y = 56,
        .textlen = 16,
        .numspaces = 2,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {      
        .y = 32, //208
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 64, //176
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {      
        .y = 96, //144
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 128, //112
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 160, //80
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 192, //48, last line on the screen, would get overwritten if there was a tooltip
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Bottom menu
        .y = 216, //24
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Back",
        .x = 0,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Next", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 275,
        .y = 2,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    }
};

const MENU_ELEMENT menusetupdone[] = {
       {
        .text = "Setup Complete!",
        .x = 0,
        .y = 224,
        .textlen = 4,
        .numspaces = 0,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Press ⤓ to Continue",
        .x = 0,
        .y = 32,
        .textlen = 4,
        .numspaces = 0,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
};


#endif
