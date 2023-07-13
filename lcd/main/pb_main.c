#include "ILIDriver.c"
#include "decode_image.h"
#include "wibbly_effect.c"
#include "freetype/freetype_extern_declarations.h"

extern const uint8_t Meiryo_ttf_start[] asm("_binary_Meiryo_ttf_start");
extern const uint8_t Meiryo_ttf_end[] asm("_binary_Meiryo_ttf_end");

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

    /*uint16_t* gradient;
    gradient = calloc(320*240, sizeof(uint16_t));

    uint16_t gradR = 0; //32 steps in 565 color space (16 bit)
    uint16_t gradB = 0; //32 steps in 565 color space (16 bit)

    for (int gradY = 0; gradY < 320; ++gradY) {
    gradB = gradY / 10;    // bit ..... ...... XXXXX
        for (int gradX = 0; gradX < 240; ++gradX) {
        gradR = gradX / 7.5; 
        gradR = gradR << 5;  //move to bit XXXXX ...... .....
        *(gradient + gradY + gradX * 320) = gradB | gradR; // RRRRR ...... BBBBB
      }

    }*/



















    for(int y=0;y<240;y+=PARALLEL_LINES) {
        send_lines(spi, y, px + 320*y);
        send_line_finish(spi);
    //    send_lines(spi, y, gradient + 320*y);
    //    send_line_finish(spi);
    }
    printf("%s\n", "finished sending display data!");
    //Go do nice stuff.
    //display_pretty_colors(spi); 
}

// vim: foldmethod=marker
