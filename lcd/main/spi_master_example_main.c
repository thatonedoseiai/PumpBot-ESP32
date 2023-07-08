#include "ILIDriver.h"
#include "decode_image.h"

extern void runTest();

const spi_bus_config_t buscfg={
    .miso_io_num=PIN_NUM_MISO,
    .mosi_io_num=PIN_NUM_MOSI,
    .sclk_io_num=PIN_NUM_CLK,
    .quadwp_io_num=-1,
    .quadhd_io_num=-1,
    .max_transfer_sz=PARALLEL_LINES*320*2+8
};

const spi_device_interface_config_t devcfg={
#ifdef CONFIG_LCD_OVERCLOCK
    .clock_speed_hz=40*1000*1000,           //Clock out at 40 MHz
#else
    .clock_speed_hz=26*1000*1000,           //Clock out at 26.667 MHz
#endif
    .mode=0,                                //SPI mode 0
    .spics_io_num=PIN_NUM_CS,               //CS pin
    .queue_size=7,                          //We want to be able to queue 7 transactions at a time
    .pre_cb=lcd_spi_pre_transfer_callback,  //Specify pre-transfer callback to handle D/C line
};

void app_main(void) {
    spi_device_handle_t spi;
    //Initialize the SPI bus
    ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &buscfg, SPI_DMA_CH_AUTO));
    //Attach the LCD to the SPI bus
    ESP_ERROR_CHECK(spi_bus_add_device(LCD_HOST, &devcfg, &spi));
    //Initialize the LCD
    lcd_init(spi);
    //Initialize the effect displayed
    /* ESP_ERROR_CHECK(pretty_effect_init()); */
    uint16_t* px;
    ESP_ERROR_CHECK(decode_image(&px));

    runTest();

    //Go do nice stuff.
    /* display_pretty_colors(spi); */
}

// vim: foldmethod=marker
