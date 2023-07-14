#include "ILIDriver.c"
#include "decode_image.h"
#include "wibbly_effect.c"
#include "freetype2/ft2build.h"
#include FT_FREETYPE_H

#define SPRITE_LIMIT 16

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


// OAM STUFF {{{
typedef struct oam_handle_ {
    FT_Bitmap* oam [SPRITE_LIMIT]
} OAM_HANDLE;


OAM_HANDLE* init_OAM() {
    OAM_HANDLE* ret = (OAM_HANDLE*) malloc(sizeof(OAM_HANDLE));
    return ret;
}

void add_char_oam(OAM_HANDLE* oam, FT_Bitmap bmp) {
    FT_Bitmap* new_bmp = (FT_Bitmap*) malloc(sizeof(FT_Bitmap));
    /* oam->oam = new_bmp; */
}
// }}}

void app_main(void) {
    spi_device_handle_t spi;
    ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &buscfg, SPI_DMA_CH_AUTO));
    ESP_ERROR_CHECK(spi_bus_add_device(LCD_HOST, &devcfg, &spi));
    lcd_init(spi);

    FT_Library lib;
    FT_Face typeFace;
    FT_GlyphSlot slot;
    FT_Vector offset;
    FT_Error error;
    char text[] = "嗚呼";
    int textLen = 2;
    FT_Long fontSize = MeiryoUI_ttf_end - MeiryoUI_ttf_start;
    int startX = 0;
    int startY = 0;

    uint16_t screenbuf[320*240];

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

    for(int n=0;n<textLen;n++) {
        FT_Set_Transform(typeFace, NULL, &offset);
        error = FT_Load_Char(typeFace, text[n], FT_LOAD_RENDER);
        if (error) printf("an error occured when loading the character!");
        //stuff is now in slot -> bitmap
        /* add_char_oam(oam, slot->bitmap); */

        FT_Int bmp_top = 240 - slot->bitmap_top;

        for(FT_Int p=0,i=slot->bitmap_left; i<slot->bitmap_left+slot->bitmap.width; i++) {
            for(FT_Int q=0,j=bmp_top; j<bmp_top+slot->bitmap.rows; j++) {
                if(i<0||j<0||i>=320||j>=240) continue;
                screenbuf[i+320*j] |= slot->bitmap.buffer[q*slot->bitmap.width+p];
            }
        }

        offset.x += slot->advance.x;
        offset.y += slot->advance.y;
    }

    for(int y=0;y<240;y+=PARALLEL_LINES) {
        send_lines(spi, y, screenbuf+320*y);
        send_line_finish(spi);
    }
    printf("%s\n", "finished sending display data!");
}

// vim: foldmethod=marker
