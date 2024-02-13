#ifndef MENUDATA_H
#define MENUDATA_H

#pragma once

const MENU_ELEMENT menuabcde[] = {
    {
        .text = "nope.",
        .x = 0,
        .y = 0,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "hello Blue!",
        .x = 30,
        .y = 30,
        .textlen = 11,
        .numspaces = 1,
        .center = true,
        .textsize = 32,
        .hline = false,
        .col = { .pixelR = 0, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 100,
        .textsize = 2,
        .col = { .pixelR = 255, .pixelB = 0, .pixelG = 0 },
        .hline = true
    },
    {
        .x = 100,
        .textsize = 3,
        .col = { .pixelR = 127, .pixelB = 0, .pixelG = 200},
        .vline = true
    }
};

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
        .y = 176,
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
        .y = 132,
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
        .y = 44,
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
        .y = 20,
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
        .y = 20,
        .textlen = 3,
        .numspaces = 0,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "0(OXX)",
        .x = 40,
        .y = 32,
        .textlen = 6,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "XXX%",
        .x = 40,
        .y = 16,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "1(OXX)",
        .x = 100,
        .y = 32,
        .textlen = 6,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "XXX%",
        .x = 100,
        .y = 16,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "2(OXX)",
        .x = 160,
        .y = 32,
        .textlen = 6,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "XXX%",
        .x = 160,
        .y = 16,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "3(OXX)",
        .x = 220,
        .y = 32,
        .textlen = 6,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "XXX%",
        .x = 220,
        .y = 16,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Horizontal Line
        .y = 26,
        .textsize = 2,
        .col = { .pixelR = 127, .pixelB = 127, .pixelG = 127},
        .hline = true
    }
};
const MENU_ELEMENT menuwelcome[] = { //all text needs to be dynamic for language changes! 
    { // (also unsure if \n works. Make it work!)
        .text = "Welcome To",
        .x = 0,
        .y = 224,
        .textlen = 10,
        .numspaces = 1,
        .center = true,
        .textsize = 24,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Pumpbot!",
        .x = 0,
        .y = 200,
        .textlen = 8,
        .numspaces = 0,
        .center = true,
        .textsize = 24,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Press ⤓ to Continue",
        .y = 32,
        .textlen = 19,
        .numspaces = 3,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
};

const MENU_ELEMENT menusetup0[] = {
    {
        .text = "Choose your Language",
        .x = 0,
        .y = 224,
        .textlen = 20,
        .numspaces = 2,
        .center = true,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .y = 176,
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "Language", 
        .x = 32,
        .y = 168,
        .textlen = 8,
        .numspaces = 0,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "English", //Really would help with right justification!!! Please!!!
        .x = 220,
        .y = 168,
        .textlen = 7,
        .numspaces = 0,
        .center = true,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {      
        .y = 144,
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Bottom menu
        .y = 24,
        .textsize = 2,
        .hline = true,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Back",
        .x = 0,
        .y = 18,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 14,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { 
        .text = "Next", // ALSO WISH I HAD RIGFHGT FCHNNGJH JUSTIFICATION
        .x = 0,
        .y = 260,
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
