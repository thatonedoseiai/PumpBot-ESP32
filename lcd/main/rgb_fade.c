// #include "esp_timer.h"
#include "settings.h"
#include "driver/ledc.h"
#include "rom/ets_sys.h"
#include "driver/gptimer.h"
#include "rgb_fade.h"

extern SETTINGS_t settings;

// static esp_timer_handle_t rgb_timer;
int curcol[] = {0, 0, 0};
gptimer_handle_t rainbow_timer;
gptimer_handle_t fade_timer;

static void fade_rgb_callback(void* arg) {
    static char goingup = 0;
    // ets_printf("%d %d %d\n", curcol[0], curcol[1], curcol[2]);
    int largerdiff;
    for(int i=0;i<3;++i) {
        largerdiff = ((unsigned char*) &settings.RGB_colour)[i]-((unsigned char*) &settings.RGB_colour_2)[i];
        if(goingup)
            curcol[i] += largerdiff / 4;
        else
            curcol[i] -= largerdiff / 4;
        ledc_set_duty(LEDC_LOW_SPEED_MODE, i+4, curcol[i]);
        ledc_update_duty(LEDC_LOW_SPEED_MODE, i+4);
    }
    if(((curcol[0] >> 6) == settings.RGB_colour_2.pixelR && !goingup) || (((curcol[0] >> 6) == settings.RGB_colour.pixelR) && goingup))
        goingup = !goingup;
}

static void rainbow_rgb_callback(void* arg) {
    static char goingup = 1;
    static char curchan = 1;
    static char curval = 0;
    curval += goingup ? 1 : -1;
    if(curval == 0 || curval == 255) {
        goingup = !goingup;
        curchan = (curchan + 2) % 3;
    }
    ledc_set_duty(LEDC_LOW_SPEED_MODE, curchan+4, curval << 6);
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
        curcol[0] = (settings.RGB_colour.pixelR) << 6;
        curcol[1] = (settings.RGB_colour.pixelG) << 6;
        curcol[2] = (settings.RGB_colour.pixelB) << 6;
        gptimer_start(fade_timer);
        break;
    case RGB_MODE_RAINBOW:
        gptimer_start(rainbow_timer);
        break;
    default:
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

// esp_err_t update_rgb_mode(char started) {
//     esp_timer_dump(stdout);
//     if(started) {
//         if(esp_timer_is_active(rgb_timer)) {
//             ets_printf("stopping timer!\n");
//             ESP_ERROR_CHECK(esp_timer_stop(rgb_timer));
//         }
//         ESP_ERROR_CHECK(esp_timer_delete(rgb_timer));
//     }
//     ets_printf("timer running: %d\n", esp_timer_is_active(rgb_timer));
//     esp_timer_dump(stdout);
//     esp_timer_create_args_t rgb_timer_args;
//     rgb_timer_args.name = "rgb timer";
//     // rgb_timer_args.dispatch_method = ESP_TIMER_ISR;
//     switch(settings.RGB_mode) {
//     case RGB_MODE_OFF:
//         break;
//     case RGB_MODE_SOLID:
//         break;
//     case RGB_MODE_FADE:
//         rgb_timer_args.callback = &fade_rgb_callback;
//         curcol[0] = settings.RGB_colour.pixelR << 6;
//         curcol[1] = settings.RGB_colour.pixelG << 6;
//         curcol[2] = settings.RGB_colour.pixelB << 6;
//         break;
//     case RGB_MODE_RAINBOW:
//         rgb_timer_args.callback = &rainbow_rgb_callback;
//         break;
//     }
//     esp_timer_create(&rgb_timer_args, &rgb_timer);
//     if(!esp_timer_is_active(rgb_timer))
//         return esp_timer_start_periodic(rgb_timer, 100000);
//     else
//         return ESP_ERR_INVALID_ARG;
//     // return ESP_OK;
// }
