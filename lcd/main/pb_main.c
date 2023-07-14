#include "ILIDriver.c"
#include "decode_image.h"
#include "wibbly_effect.c"
#include "freetype2/ft2build.h"
#include FT_FREETYPE_H

extern const uint8_t MeiryoUI_ttf_start[] asm("_binary_MeiryoUI_ttf_start");
extern const uint8_t MeiryoUI_ttf_end[] asm("_binary_MeiryoUI_ttf_end");

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
    ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &buscfg, SPI_DMA_CH_AUTO));
    ESP_ERROR_CHECK(spi_bus_add_device(LCD_HOST, &devcfg, &spi));
    lcd_init(spi);
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

    FT_Library lib;
    FT_Face typeFace;
    FT_GlyphSlot slot;
    FT_Vector offset;
    FT_Error error;
    char text[] = "嗚呼";
    int textLen;
    FT_Long fontSize = MeiryoUI_ttf_end - MeiryoUI_ttf_start;
    int startX = 0;
    int startY = 0;

    const FT_Open_Args openArgs = {
        .flags = 0x2 | 0x8,
        .memory_base = MeiryoUI_ttf_start,
        .memory_size = fontSize,
        //.pathname = unused,
        //.stream = unused,
        .driver = NULL, //TODO: Set to "truetype" and deal with the structs like a big boy
        //.num_params = unused,
        //.params = unused,
    };

    error = FT_Init_FreeType(&lib);
    error = FT_Open_Face (lib, &openArgs, 0, &typeFace);
    error = FT_Set_Char_Size (typeFace, fontSize * 64, 0, 100, 0); // 0 = copy last value
    slot = typeFace->glyph;
    offset.x = startX * 64;
    offset.y = startY * 64;

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
