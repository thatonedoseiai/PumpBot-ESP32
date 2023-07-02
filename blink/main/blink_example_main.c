/* Blink Example

   This example code is in the Public Domain (or CC0 licensed, at your option.)

   Unless required by applicable law or agreed to in writing, this
   software is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
   CONDITIONS OF ANY KIND, either express or implied.
*/
#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "driver/gpio.h"
#include "esp_log.h"
#include "led_strip.h"
#include "sdkconfig.h"
#include "esp_lcd_ili9341.h"
#include "testimg.h"

static const char *TAG = "example";

/* Use project configuration menu (idf.py menuconfig) to choose the GPIO to blink,
   or you can edit the following line and set a number here.
*/
#define BLINK_GPIO CONFIG_BLINK_GPIO
#define LCD_RST -1
#define LCD_HOST SPI2_HOST

static uint8_t s_led_state = 0;

#ifdef CONFIG_BLINK_LED_RMT

static led_strip_handle_t led_strip;
static esp_lcd_panel_handle_t lcd;
const esp_lcd_panel_io_handle_t lcd_io_handle = NULL;
const esp_lcd_panel_dev_config_t panel_config = {
    .reset_gpio_num = LCD_RST,
    .rgb_endian = LCD_RGB_ENDIAN_BGR,
    .bits_per_pixel = 16,
};
static spi_bus_config_t buscfg = {
    .sclk_io_num = EXAMPLE_PIN_NUM_SCLK,
    .mosi_io_num = EXAMPLE_PIN_NUM_MOSI,
    .miso_io_num = EXAMPLE_PIN_NUM_MISO,
    .quadwp_io_num = -1, // Quad SPI LCD driver is not yet supported
    .quadhd_io_num = -1, // Quad SPI LCD driver is not yet supported
    .max_transfer_sz = EXAMPLE_LCD_H_RES * 80 * sizeof(uint16_t), // transfer 80 lines of pixels (assume pixel is RGB565) at most in one SPI transaction
};
esp_lcd_panel_io_spi_config_t io_config = {
    .dc_gpio_num = 16,
    .cs_gpio_num = 18,
    .pclk_hz = 6350000,
    .lcd_cmd_bits = 8,
    .lcd_param_bits = 8,
    .spi_mode = 3,
    .trans_queue_depth = 10,
};
/* static i2c_config_t i2c_conf = { */
/*     .mode = I2C_MODE_MASTER, */
/*     .sda_io_num = EXAMPLE_PIN_NUM_SDA, */
/*     .scl_io_num = EXAMPLE_PIN_NUM_SCL, */
/*     .sda_pullup_en = GPIO_PULLUP_ENABLE, */
/*     .scl_pullup_en = GPIO_PULLUP_ENABLE, */
/*     .master.clk_speed = EXAMPLE_LCD_PIXEL_CLOCK_HZ, */
/* }; */
/* esp_lcd_panel_io_i2c_config_t io_config = { */
/*     .dev_addr = EXAMPLE_I2C_HW_ADDR, */
/*     .control_phase_bytes = 1, // refer to LCD spec */
/*     .dc_bit_offset = 6,       // refer to LCD spec */
/*     .lcd_cmd_bits = 8, */
/*     .lcd_param_bits = 8, */
/* }; */

static void blink_led(void)
{
    /* If the addressable LED is enabled */
    if (s_led_state) {
        /* Set the LED pixel using RGB from 0 (0%) to 255 (100%) for each color */
        led_strip_set_pixel(led_strip, 0, 16, 16, 16);
        /* Refresh the strip to send data */
        led_strip_refresh(led_strip);
    } else {
        /* Set all LED off to clear all pixels */
        led_strip_clear(led_strip);
    }
}

static void configure_led(void)
{
    ESP_LOGI(TAG, "Example configured to blink addressable LED!");
    /* LED strip initialization with the GPIO and pixels number*/
    led_strip_config_t strip_config = {
        .strip_gpio_num = BLINK_GPIO,
        .max_leds = 1, // at least one LED on board
    };
    led_strip_rmt_config_t rmt_config = {
        .resolution_hz = 10 * 1000 * 1000, // 10MHz
    };
    ESP_ERROR_CHECK(led_strip_new_rmt_device(&strip_config, &rmt_config, &led_strip));
    /* Set all LED off to clear all pixels */
    led_strip_clear(led_strip);
}

#elif CONFIG_BLINK_LED_GPIO

static void blink_led(void)
{
    /* Set the GPIO level according to the state (LOW or HIGH)*/
    gpio_set_level(BLINK_GPIO, s_led_state);
}

static void configure_led(void)
{
    ESP_LOGI(TAG, "Example configured to blink GPIO LED!");
    gpio_reset_pin(BLINK_GPIO);
    /* Set the GPIO as a push/pull output */
    gpio_set_direction(BLINK_GPIO, GPIO_MODE_OUTPUT);
}

#endif

void start_lcd() {
    ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &buscfg, SPI_DMA_CH_AUTO)); // Enable the DMA feature
                                                                        ESP_ERROR_CHECK(esp_lcd_new_panel_io_spi((esp_lcd_spi_bus_handle_t)LCD_HOST, &io_config, &lcd_io_handle));
    /* ESP_ERROR_CHECK(i2c_param_config(I2C_HOST, &i2c_conf)); */
    /* ESP_ERROR_CHECK(i2c_driver_install(I2C_HOST, I2C_MODE_MASTER, 0, 0, 0)); */
    /* ESP_ERROR_CHECK(esp_lcd_new_panel_io_i2c((esp_lcd_i2c_bus_handle_t)I2C_HOST, &io_config, &lcd_io_handle)); */
    ESP_ERROR_CHECK(esp_lcd_new_panel_ili9341(lcd_io_handle, &panel_config, &lcd));
}

void app_main(void)
{

    /* Configure the peripheral according to the LED type */
    /* configure_led(); */
    start_lcd();
    /* void* bitmap; */

    ESP_ERROR_CHECK(panel_ili9341_init(&lcd));
    ESP_ERROR_CHECK(panel_ili9341_disp_on_off(&lcd, false));
    ESP_ERROR_CHECK(panel_ili9341_draw_bitmap(&lcd, 0, 0, 320, 240, 0x8a + (&testimg_bmp[0])));

    /* while (1) { */
        /* ESP_LOGI(TAG, "Turning the LED %s!", s_led_state == true ? "ON" : "OFF"); */
        /* blink_led(); */
        /* /1* Toggle the LED state *1/ */
        /* s_led_state = !s_led_state; */
        /* vTaskDelay(CONFIG_BLINK_PERIOD / portTICK_PERIOD_MS); */
    /* } */
}
