#include "ILIDriver.c"
#include "decode_image.h"
#include "wibbly_effect.c"
#include "freetype2/ft2build.h"
#include <stdio.h>
#include <rom/ets_sys.h>
#include FT_FREETYPE_H

#define SPRITE_LIMIT 16
extern const uint8_t MeiryoUI_ttf_start[] asm("_binary_MeiryoUImin_ttf_start");
extern const uint8_t MeiryoUI_ttf_end[] asm("_binary_MeiryoUImin_ttf_end");
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
    ets_printf("Reached app_main!\n");
    spi_device_handle_t spi;
    ets_printf("1\n");
    ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &buscfg, SPI_DMA_CH_AUTO));
    ets_printf("2\n");
    ESP_ERROR_CHECK(spi_bus_add_device(LCD_HOST, &devcfg, &spi));
    ets_printf("3\n");
    lcd_init(spi);
    ets_printf("SPI bus initialized!\n");
    FT_Library lib;
    FT_Face typeFace;
    FT_GlyphSlot slot;
    FT_Vector offset;
    FT_Error error;
    char text[] = "b";//"嗚呼";
    int textLen = 1;
    int fontSize = 18;
    FT_Long fontBinSize = MeiryoUI_ttf_end - MeiryoUI_ttf_start;
    int startX = 20;
    int startY = 20;

    ets_printf("strlen: %d\n", textLen);

    uint16_t* screenbuf = (uint16_t*) malloc(320*240*sizeof(uint16_t));
    for(int i=0;i<320*240;++i) {
	screenbuf[i] = 0;
    }
    // ets_printf("FreeType assignments made!\n");
    // const FT_Open_Args openArgs = {
    //     .flags = 0x2 | 0x8,
    //     .memory_base = MeiryoUI_ttf_start,
    //     .memory_size = fontBinSize,
    //     //.pathname = unused,
    //     //.stream = unused,
    //     .driver = NULL, //TODO: Set to "truetype" and deal with the structs like a big boy
    //     //.num_params = unused,
    //     //.params = unused,
    // };
    //     ets_printf("FreeType Init Starting...\n");

    error = FT_Init_FreeType(&lib);
    if(error!=0) ets_printf("%s %d\n", "Error occured @FT_Init_FreeType! Error:", (int)error);
    // error = FT_Open_Face (lib, &openArgs, 0, &typeFace);
    // if(error!=0) ets_printf("%s %d\n", "Error occured @FT_Open_Face! Error:", (int)error); 
    error = FT_New_Memory_Face(lib, MeiryoUI_ttf_start, fontBinSize, 0, &typeFace);
    if(error!=0) ets_printf("%s %d\n", "Error occured @FT_New_Memory_Face! Error:", (int)error);
    error = FT_Set_Char_Size (typeFace, fontSize << 6, 0, 100, 0); // 0 = copy last value
    if(error!=0) ets_printf("%s %d\n", "Error occured @FT_Set_Char_Size! Error:", (int)error);

    slot = typeFace->glyph;
    offset.x = startX << 6;
    offset.y = startY << 6;

    
    for(int n=0;n<textLen;n++) {
    //     ets_printf("Rendering character...\n");
        FT_Set_Transform(typeFace, NULL, &offset);
        error = FT_Load_Char(typeFace, text[n], FT_LOAD_RENDER);
        if(error!=0) ets_printf("%s %d\n", "Error occured @FT_Load_Char! Error:", (int)error);
    //     //stuff is now in slot -> bitmap
    //    add_char_oam(oam, slot->bitmap); 

        FT_Int bmp_top = 240 - slot->bitmap_top;

	ets_printf("bitmap top: %d\n", slot->bitmap_top);
        ets_printf("BITMAP WIDTH: %d\n", slot->bitmap.width);
        for(FT_Int q=0,j=bmp_top; j<bmp_top+slot->bitmap.rows; j++, q++) {
            for(FT_Int p=0,i=slot->bitmap_left; i<slot->bitmap_left+slot->bitmap.width; i++, p++) {
            //ets_printf("%s\n", "Started for loop 1");
                //ets_printf("%s\n", "Started for loop 2");
                if(i<0||j<0||i>=320||j>=240) continue;
		//ets_printf("loc: %d, %d, %x\n", p, q, slot->bitmap.buffer[q*slot->bitmap.width+p]);
		//ets_printf("%d\n", i);
                screenbuf[i+320*j] |= slot->bitmap.buffer[q*slot->bitmap.width+p];
                //ets_printf("%s\n", "Ended for loop 2");
            }
            //ets_printf("%s\n", "Ended for loop 1");
        }

        offset.x += slot->advance.x;
        offset.y += slot->advance.y;
    //     ets_printf("Character rendered!\n");
    }

    ets_printf("Rendering characters done!\n");
    //FT_Done_Face (typeFace);
    //FT_Done_FreeType(lib);
    ets_printf("Starting to send to display...\n");
    for(int y=0;y<240;y+=PARALLEL_LINES) {
        send_lines(spi, y, screenbuf+320*y);
        ets_printf("%s %d\n", "sent line", y);
        send_line_finish(spi);
    }
    //free(screenbuf);
    ets_printf("finished sending display data!\n");
}

// vim: foldmethod=marker
