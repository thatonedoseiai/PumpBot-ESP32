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
    	{
        .x = 160,
        .textsize = 2,
        .col = { .pixelR = 127, .pixelB = 127, .pixelG = 127},
        .vline = true
    },
    	{
        .text = "Large - 30px",
        .x = 0,
        .y = 226,
        .textlen = 12,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "Medium - 22px",
        .x = 0,
        .y = 160,
        .textlen = 13,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "Standard - 18px",
        .x = 0,
        .y = 100,
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
        .y = 48,
        .textlen = 12,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
     	{
        .text = "36px",
        .x = 160,
        .y = 226,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
        {
        .text = "30px",
        .x = 160,
        .y = 160,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "22px",
        .x = 160,
        .y = 100,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "18px",
        .x = 160,
        .y = 48,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "A Quick",
        .x = 0,
        .y = 205,
        .textlen = 7,
        .numspaces = 1,
        .center = false,
        .textsize = 30,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }

    },
    {
        .text = "Brown Fox",
        .x = 0,
        .y = 139,
        .textlen = 9,
        .numspaces = 1,
        .center = false,
        .textsize = 22,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    
    },
    {
        .text = "Jumps over",
        .x = 0,
        .y = 79,
        .textlen = 10,
        .numspaces = 1,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    
    },
    {
        .text = "the lazy dog.",
        .x = 0,
        .y = 27,
        .textlen = 13,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {//sample text in chinese
        //.text = "鑑於人類",
	.text = "36PX",
        .x = 162,
        .y = 205,
        .textlen = 4,
        .numspaces = 0,
        .center = false,
        .textsize = 36,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        //.text = "社會個",
        .text = "30P",
        .x = 162,
        .y = 139,
        .textlen = 3,
        .numspaces = 0,
        .center = false,
        .textsize = 30,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        //.text = "成員儕有個",
        .text = "22PIX",
        .x = 162,
        .y = 79,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 22,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        //.text = "固有尊嚴脫仔平等個脫仔",
	.text = "18PIXELSBIG",
        .x = 162,
        .y = 27,
        .textlen = 11,
        .numspaces = 0,
        .center = false,
        .textsize = 18,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    }
};

#endif
