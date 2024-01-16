#ifndef BOARD_CONFIG_H
#define BOARD_CONFIG_H

// for pin numbers regarding the SPI bus
#include "ILIDriver.h"

#define PIN_NUM_SW0 0
#define PIN_NUM_SW1 4
#define PIN_NUM_ENC_A 34
#define PIN_NUM_ENC_B 35
#define PIN_NUM_ENC_BTN 36
#define SPRITE_LIMIT 255

const spi_bus_config_t buscfg={
    .miso_io_num=PIN_NUM_MISO,
    .mosi_io_num=PIN_NUM_MOSI,
    .sclk_io_num=PIN_NUM_CLK,
    .quadwp_io_num=-1,
    .quadhd_io_num=-1,
    .max_transfer_sz=240*320*3+8
};
const gpio_config_t btn_conf = {
    .pin_bit_mask = ((1ULL << PIN_NUM_SW0) |
                    (1ULL << PIN_NUM_SW1)),
    .mode = GPIO_MODE_INPUT,
    .pull_up_en = true,
};
const spi_device_interface_config_t devcfg={
    .clock_speed_hz=40*1000*1000,
    .mode=0,                        // SPI mode 0
    .spics_io_num=PIN_NUM_CS,       // CS pin
    .queue_size=8,                  // queuing 7 transactions at a time
    .pre_cb=lcd_spi_pre_transfer_callback,
    // Specify pre-transfer callback to handle D/C line
};
const esp_vfs_littlefs_conf_t conf = {
    .base_path = "/mainfs",
    .partition_label = "filesystem",
    .format_if_mount_failed = true,
    .dont_mount = false,
};
const wifi_config_t wifi_config = {
    .ap = {
        .ssid = "pumpy wifi",
        .ssid_len = strlen("pumpy wifi"),
        .channel = 1,
        .password = "pumperslol",
        .max_connection = 4,
#ifdef CONFIG_ESP_WIFI_SOFTAP_SAE_SUPPORT
        .authmode = WIFI_AUTH_WPA3_PSK,
        .sae_pwe_h2e = WPA3_SAE_PWE_BOTH,
#else /* CONFIG_ESP_WIFI_SOFTAP_SAE_SUPPORT */
        .authmode = WIFI_AUTH_WPA2_PSK,
#endif
        .pmf_cfg = {
                .required = true,
        // .authmode = WIFI_AUTH_OPEN; // if no password
        },
    },
};

#endif
