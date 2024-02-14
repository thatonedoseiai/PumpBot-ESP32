#ifndef BOARD_CONFIG_H
#define BOARD_CONFIG_H

// for pin numbers regarding the SPI bus
#include "ILIDriver.h"
#include "esp_littlefs.h"
#include "esp_wifi.h"
#include "nvs_flash.h"
#include "driver/ledc.h"

#define PIN_NUM_SW0 0
#define PIN_NUM_SW1 3
#define PIN_NUM_ENC_A 17
#define PIN_NUM_ENC_B 8
#define PIN_NUM_ENC_BTN 18
#define SPRITE_LIMIT 255

extern const spi_bus_config_t buscfg;
extern const gpio_config_t btn_conf;
extern const spi_device_interface_config_t devcfg;
extern const esp_vfs_littlefs_conf_t conf;
extern const wifi_config_t wifi_config;
extern const ledc_timer_config_t timer_config_0;
extern ledc_channel_config_t channel_config;
extern const int pwm_gpio_nums[8];

#endif
