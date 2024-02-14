#include "board_config.h"
#include "ILIDriver.h"
#include "esp_littlefs.h"
#include "esp_wifi.h"
#include "nvs_flash.h"
#include <string.h>

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
const ledc_timer_config_t timer_config_0 = {
    .speed_mode = LEDC_LOW_SPEED_MODE,
    .freq_hz = 1000,
    .duty_resolution = LEDC_TIMER_14_BIT,
    .timer_num = LEDC_TIMER_0,
    .clk_cfg = LEDC_USE_APB_CLK,
};
ledc_channel_config_t channel_config = {
    // .gpio_num = 14,
    .speed_mode = LEDC_LOW_SPEED_MODE,
    .channel = LEDC_CHANNEL_0,
    .intr_type = LEDC_INTR_DISABLE,
    .timer_sel = LEDC_TIMER_0,
    // .duty = 1638,
    .duty = 0,
    .hpoint = 0,
};

const int pwm_gpio_nums[8] = {4, 5, 6, 7, 14, 21, 47, 13};
