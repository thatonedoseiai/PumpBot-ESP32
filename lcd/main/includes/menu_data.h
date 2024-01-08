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

#endif
