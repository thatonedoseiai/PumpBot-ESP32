#ifndef MENUDATA_H
#define MENUDATA_H

#pragma once

const MENU_ELEMENT menuabcde[] = {
    {
        .text = "nope.",
        .x = 30,
        .y = 80,
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
        .textsize = 32,
        .col = { .pixelR = 255, .pixelG = 255, .pixelB = 255 }
    },
    {
        .text = "PumpBot!",
        .x = 60,
        .y = 154,
        .textlen = 8,
        .numspaces = 0,
        .center = true,
        .textsize = 32,
        .col = { .pixelR = 255, .pixelG = 255, .pixelB = 255 }
    },
    {
        .text = "Press v to Continue",
        .x = 60,
        .y = 10,
        .textlen = 19,
        .numspaces = 3,
        .center = true,
        .textsize = 16,
        .col = { .pixelR = 255, .pixelG = 255, .pixelB = 255 }
    },
};

const MENU_ELEMENT text_test[] = {
    {//Font size numbers
        .text = "Large - 32px",
        .x = 0,
        .y = 240,
        .textlen = 12,
        .numspaces = 2,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "Medium - 24px",
        .x = 0,
        .y = 176,
        .textlen = 13,
        .numspaces = 2,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "Standard - 20px",
        .x = 0,
        .y = 108,
        .textlen = 15,
        .numspaces = 2,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "Small - 16px",
        .x = 0,
        .y = 44,
        .textlen = 12,
        .numspaces = 2,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
     {
        .text = "40px",
        .x = 160,
        .y = 240,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "32px",
        .x = 160,
        .y = 176,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "24px",
        .x = 160,
        .y = 108,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "20px",
        .x = 160,
        .y = 44,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {//sample text
        .text = "A Quick Br",
        .x = 0,
        .y = 216,
        .textlen = 10,
        .numspaces = 2,
        .center = false,
        .textsize = 32,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }

    },
    {
        .text = "own Fox Jumps",
        .x = 0,
        .y = 150,
        .textlen = 13,
        .numspaces = 3,
        .center = false,
        .textsize = 24,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    
    },
    {
        .text = "over the lazy dog.",
        .x = 0,
        .y = 80,
        .textlen = 18,
        .numspaces = 3,
        .center = false,
        .textsize = 20,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    
    },
    {
        .text = "The dog was very lazy.",
        .x = 0,
        .y = 20,
        .textlen = 22,
        .numspaces = 4,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {//sample text in chinese
        .text = "鑑於人類",
        .x = 160,
        .y = 216,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 40,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "社會個成員",
        .x = 160,
        .y = 150,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 32,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "儕有個固有尊",
        .x = 160,
        .y = 80,
        .textlen = 6,
        .numspaces = 0,
        .center = false,
        .textsize = 24,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "嚴脫仔平等個脫仔",
        .x = 160,
        .y = 20,
        .textlen = 8,
        .numspaces = 0,
        .center = false,
        .textsize = 20,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },

    { //Vertical Line
        .x = 160,
        .textsize = 2,
        .col = { .pixelR = 127, .pixelB = 127, .pixelG = 127},
        .vline = true
    }
};

#endif
