// #include "esp_timer.h"
#include "settings.h"
#include "driver/ledc.h"
#include "rom/ets_sys.h"
#include "driver/gptimer.h"
#include "rgb_fade.h"

extern SETTINGS_t settings;

// static esp_timer_handle_t rgb_timer;
int curcol[] = {0, 0, 0};
int curcycle = 0;
gptimer_handle_t rainbow_timer;
gptimer_handle_t fade_timer;

static void fade_rgb_callback(void* arg) {
    static char goingup = 0;
    // ets_printf("%d %d %d\n", curcol[0], curcol[1], curcol[2]);
    // int largerdiff;
    int diff;
    int32_t largerdiff;
    for(int i=0;i<3;++i) {
        // largerdiff = ((((unsigned char*) &settings.RGB_colour)[i]-((unsigned char*) &settings.RGB_colour_2)[i]) * (settings.RGB_brightness >> 8)) / 64;
        diff = ((unsigned char*) &settings.RGB_colour)[i]-((unsigned char*) &settings.RGB_colour_2)[i];
        largerdiff = ((diff * settings.RGB_brightness) / (settings.RGB_speed << 10));
        if(goingup)
            curcol[i] += (largerdiff);
        else
            curcol[i] -= (largerdiff);
        if(curcol[i] < 0)
            curcol[i] = 0;
        if(curcol[i] > 16383)
            curcol[i] = 16383;
        ledc_set_duty(LEDC_LOW_SPEED_MODE, i+4, curcol[i]);
        ledc_update_duty(LEDC_LOW_SPEED_MODE, i+4);
    }
    // if(((curcol[0]) <= ((settings.RGB_colour_2.pixelR * (settings.RGB_brightness >> 8))) && !goingup) || 
    //     (((curcol[0]) >= ((settings.RGB_colour.pixelR * (settings.RGB_brightness >> 8))) && goingup)))
    curcycle++;
    if(curcycle == (settings.RGB_speed << 2)) {
        goingup = !goingup;
        curcycle = 0;
    }
}

static void rainbow_rgb_callback(void* arg) {
    static char goingup = 1;
    static char curchan = 1;
    static int curval = 0;
    // curval += goingup ? (settings.RGB_speed/3) : -(settings.RGB_speed/3);
    curval += goingup ? (12288 / settings.RGB_speed) : -(12288 / settings.RGB_speed);
    if(curval <= 0 || curval >= 16383) {
        goingup = !goingup;
        curchan = (curchan + 2) % 3;
    }
    if(curval > 16383)
        curval = 16383;
    if(curval < 0)
        curval = 0;
    ledc_set_duty(LEDC_LOW_SPEED_MODE, curchan+4, (curval * settings.RGB_brightness) >> 14);
    ledc_update_duty(LEDC_LOW_SPEED_MODE, curchan+4);
}

static bool rainbow_callback(gptimer_handle_t timer, const gptimer_alarm_event_data_t* edata, void* user_ctx) {
    rainbow_rgb_callback(NULL);
    return pdTRUE;
}

static bool fade_callback(gptimer_handle_t timer, const gptimer_alarm_event_data_t* edata, void* user_ctx) {
    fade_rgb_callback(NULL);
    return pdTRUE;
}

void rgb_update() {
    gptimer_stop(rainbow_timer);
    gptimer_stop(fade_timer);
    switch(settings.RGB_mode) {
    case RGB_MODE_FADE:
        curcol[0] = ((settings.RGB_colour.pixelR) * (settings.RGB_brightness)) >> 8;
        curcol[1] = ((settings.RGB_colour.pixelG) * (settings.RGB_brightness)) >> 8;
        curcol[2] = ((settings.RGB_colour.pixelB) * (settings.RGB_brightness)) >> 8;
        curcycle = 0;
        gptimer_start(fade_timer);
        break;
    case RGB_MODE_RAINBOW:
        gptimer_start(rainbow_timer);
        break;
    case RGB_MODE_OFF:
        for(int i=0;i<3;++i)
            ledc_stop(LEDC_LOW_SPEED_MODE, i+4, 0);
        break;
    case RGB_MODE_SOLID:
        for(int i=0;i<3;++i) {
            ledc_set_duty(LEDC_LOW_SPEED_MODE, i+4, ((char*) &settings.RGB_colour.pixelR)[i] * (settings.RGB_brightness >> 8));
            ledc_update_duty(LEDC_LOW_SPEED_MODE, i+4);
        }
        break;
    }
}

void rgb_init() {
    rainbow_timer = NULL;
    fade_timer = NULL;
    gptimer_config_t timer_config = {
        .clk_src = GPTIMER_CLK_SRC_DEFAULT,
        .direction = GPTIMER_COUNT_UP,
        .resolution_hz = 1 * 1000 * 1000, // 1MHz, 1 tick = 1us
    };
    ESP_ERROR_CHECK(gptimer_new_timer(&timer_config, &rainbow_timer));
    ESP_ERROR_CHECK(gptimer_new_timer(&timer_config, &fade_timer));
    gptimer_alarm_config_t alarm_config = {
        .reload_count = 0, // counter will reload with 0 on alarm event
        .alarm_count = 20000, // period = 1s @resolution 1MHz
        .flags.auto_reload_on_alarm = true, // enable auto-reload
    };   
    ESP_ERROR_CHECK(gptimer_set_alarm_action(rainbow_timer, &alarm_config));
    ESP_ERROR_CHECK(gptimer_set_alarm_action(fade_timer, &alarm_config));
    gptimer_event_callbacks_t cbs = { .on_alarm = rainbow_callback };
    ESP_ERROR_CHECK(gptimer_register_event_callbacks(rainbow_timer, &cbs, NULL));
    gptimer_event_callbacks_t cbf = { .on_alarm = fade_callback };
    ESP_ERROR_CHECK(gptimer_register_event_callbacks(fade_timer, &cbf, NULL));
    ESP_ERROR_CHECK(gptimer_enable(rainbow_timer));
    ESP_ERROR_CHECK(gptimer_enable(fade_timer));
    ESP_ERROR_CHECK(gptimer_start(fade_timer));
}
