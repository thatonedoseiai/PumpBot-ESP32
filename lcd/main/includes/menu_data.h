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
        .y = 218,
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
        .y = 152,
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
        .text = "36px",
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
        .text = "30px",
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
        .text = "22px",
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
        .text = "18px",
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
        .text = "A Quick",
        .x = 0,
        .y = 173,
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
        .y = 113,
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
        .y = 61,
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
        .y = 13,
        .textlen = 13,
        .numspaces = 2,
        .center = false,
        .textsize = 12,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {//sample text in chinese
        .text = "鑑於人",
	    //.text = "36PX",
        .x = 162,
        .y = 173,
        .textlen = 3,
        .numspaces = 0,
        .center = false,
        .textsize = 36,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "類社會",
        //.text = "30P",
        .x = 162,
        .y = 113,
        .textlen = 3,
        .numspaces = 0,
        .center = false,
        .textsize = 30,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "個成員儕有",
        // .text = "22PIX",
        .x = 162,
        .y = 61,
        .textlen = 5,
        .numspaces = 0,
        .center = false,
        .textsize = 22,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "個固有尊嚴脫",
	//.text = "18PIXELSBIG",
        .x = 162,
        .y = 13,
        .textlen = 6,
        .numspaces = 0,
        .center = false,
        .textsize = 18,
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
        .textsize = 48,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "⟳", //static symbol
        .x = 0,
        .y = 112,
        .textlen = 1,
        .numspaces = 0,
        .center = true,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "CHX⤓", //X = current channel
        .x = 0,
        .y = 52,
        .textlen = 4,
        .numspaces = 0,
        .center = true,
        .textsize = 20,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "OXX", //OFF or ON for current channel
        .x = 2,
        .y = 22,
        .textlen = 3,
        .numspaces = 0,
        .center = true,
        .textsize = 20,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    {
        .text = "SET", //Settings menu, will be custom icon later
        .x = 268,
        .y = 22,
        .textlen = 3,
        .numspaces = 0,
        .center = false,
        .textsize = 20,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //I know this shit won't line up, I'm lazy
        .text = "0(OXX)  1(OXX)  2(OXX)  3(OXX)",//and this is also temp(?)
        .x = 0,
        .y = 32,
        .textlen = 30,
        .numspaces = 6,
        .center = true,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { // yeah this definitely won't work as only 2 objects
        .text = "XXX%  XXX%  XXX%  XXX%",
        .x = 0,
        .y = 16,
        .textlen = 22,
        .numspaces = 6,
        .center = false,
        .textsize = 16,
        .hline = false,
        .col = { .pixelR = 255, .pixelB = 255, .pixelG = 255 }
    },
    { //Horizontal Line
        .y = 34,
        .textsize = 2,
        .col = { .pixelR = 127, .pixelB = 127, .pixelG = 127},
        .hline = true
    }
};

#endif
