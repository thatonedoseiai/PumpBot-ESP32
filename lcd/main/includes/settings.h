#ifndef SETTINGS_H
#define SETTINGS_H

#define SETTINGS_FILE "/mainfs/settings.conf"

#include "oam.h"

enum RGB_MODE_t {
    RGB_MODE_OFF = 0,
    RGB_MODE_SOLID,
    RGB_MODE_FADE,
    RGB_MODE_RAINBOW
};

enum PRESSURE_UNIT_t {
    KPA = 0,
    BAR,
    PSI,
    MMHG
};

enum LANGUAGE_t {
    EN = 0,
    JP = 1, 
    FR = 2,
    ES = 3,
    PT = 4,
    ZH = 5,
    CN = 6,
    RU = 7,
    DE = 8
};

typedef struct {
    char wifi_name[32];
    char wifi_pass[64];
    int disp_theme;
    uint24_RGB custom_theme_color;
    int disp_brightness;
    uint16_t RGB_brightness;
    int RGB_speed;
    enum RGB_MODE_t RGB_mode;
    uint24_RGB RGB_colour;
    uint24_RGB RGB_colour_2;
    enum PRESSURE_UNIT_t pressure_units;
    enum LANGUAGE_t language;
    uint16_t pwm_min_limit[4];
    uint16_t pwm_max_limit[4];
    char output_set_on_off_only[4];
} SETTINGS_t;

void write_to_file(SETTINGS_t* settings);
int read_from_file(SETTINGS_t* settings);

#endif
