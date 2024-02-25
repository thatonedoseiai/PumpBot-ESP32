#ifndef SETTINGS_H
#define SETTINGS_H

#define SETTINGS_FILE "/mainfs/settings.conf"

#include "oam.h"

enum RGB_MODE_t {
    RGB_MODE_SOLID = 0,
    RGB_MODE_FADE,
    RGB_MODE_RAINBOW
};

enum PRESSURE_UNIT_t {
    KPA = 0,
    BAR,
    PSI,
    MMHG
};

typedef struct {
    char wifi_name[32];
    char wifi_pass[64];
    uint32_t serv_ip;
    char serv_pass[64];
    int disp_theme;
    int disp_brightness;
    int RGB_brightness;
    int RGB_speed;
    enum RGB_MODE_t RGB_mode;
    uint24_RGB RGB_colour;
    enum PRESSURE_UNIT_t pressure_units;
    int pwm_min_limit[4];
    int pwm_max_limit[4];
    char output_set_on_off_only[4];
    int pwm_frequency;
} SETTINGS_t;

void write_to_file(SETTINGS_t* settings);
void read_from_file(SETTINGS_t* settings);

#endif
