#include "ILIDriver.c"
#include "freetype2/ft2build.h"
#include <stdio.h>
#include <rom/ets_sys.h>
#include FT_FREETYPE_H
#include "oam.c"

#define SPRITE_LIMIT 16
extern const uint8_t MeiryoUI_ttf_start[] asm("_binary_MeiryoUImin_ttf_start");
extern const uint8_t MeiryoUI_ttf_end[] asm("_binary_MeiryoUImin_ttf_end");
const spi_bus_config_t buscfg={
	.miso_io_num=PIN_NUM_MISO,
	.mosi_io_num=PIN_NUM_MOSI,
	.sclk_io_num=PIN_NUM_CLK,
	.quadwp_io_num=-1,
	.quadhd_io_num=-1,
	.max_transfer_sz=240*320*3+8
};

const spi_device_interface_config_t devcfg={
#ifdef CONFIG_LCD_OVERCLOCK
	.clock_speed_hz=40*1000*1000,			//Clock out at 40 MHz
#else
	.clock_speed_hz=26*1000*1000,			//Clock out at 26.667 MHz
#endif
	.mode=0,								//SPI mode 0
	.spics_io_num=PIN_NUM_CS,				//CS pin
	.queue_size=7,							//We want to be able to queue 7 transactions at a time
	.pre_cb=lcd_spi_pre_transfer_callback,	//Specify pre-transfer callback to handle D/C line
};

void app_main(void) {
	ets_printf("Reached app_main!\n");
	spi_device_handle_t spi;
	ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &buscfg, SPI_DMA_CH_AUTO));
	ESP_ERROR_CHECK(spi_bus_add_device(LCD_HOST, &devcfg, &spi));
	lcd_init(spi);
	init_oam();
	ets_printf("SPI and OAM initialized!\n");
	static FT_Library lib;
	static FT_Face typeFace; // = *(FT_Face*)malloc(sizeof(FT_Face));
	static FT_GlyphSlot slot;
	static FT_Vector offset;
	static FT_Error error;
	static FT_ULong text[] = {0x547C, 0x55DA};//"嗚呼";
	const int textLen = 2;
	const int fontSize = 32;
	const FT_Long fontBinSize = MeiryoUI_ttf_end - MeiryoUI_ttf_start;
	const int startX = 20;
	const int startY = 20;
	const uint24_RGB fillColor = {
		.pixelR = 0x10,
		.pixelG = 0,
		.pixelB = 0x30,
	};
	const uint24_RGB color = {
		.pixelR = 0x80,
		.pixelG = 0,
		.pixelB = 0x10,
	};

	send_color(spi, fillColor);

	error = FT_Init_FreeType(&lib);
	if(error!=0) ets_printf("%s %d\n", "Error occured @FT_Init_FreeType! Error:", (int)error);
	error = FT_New_Memory_Face(lib, MeiryoUI_ttf_start, fontBinSize, 0, &typeFace);
	if(error!=0) ets_printf("%s %d\n", "Error occured @FT_New_Memory_Face! Error:", (int)error);
	error = FT_Select_Charmap(typeFace, FT_ENCODING_UNICODE);
	if(error!=0) ets_printf("%s %d\n", "Error occured @FT_Select_Charmap! Error:", (int)error);	
	error = FT_Set_Char_Size (typeFace, fontSize << 6, 0, 100, 0); // 0 = copy last value
	if(error!=0) ets_printf("%s %d\n", "Error occured @FT_Set_Char_Size! Error:", (int)error);

	slot = typeFace->glyph;
	ets_printf("slot = %p\n", slot);
	//heap_caps_print_heap_info(MALLOC_CAP_8BIT);
	offset.x = startX << 6;
	offset.y = startY << 6;

	uint24_RGB* colorbuf = malloc(sizeof(uint24_RGB) * 10000);
		for (int i = 0; i < 10000; ++i) colorbuf[i] = color;
	init_sprite(colorbuf, 130, 130, 100, 100, false, false, true);

	for(int n=0;n<textLen;n++) {
		FT_Set_Transform(typeFace, NULL, &offset);
		ets_printf("Rendering character 0x%x...\n", text[n]);
		error = FT_Load_Char(typeFace, text[n], FT_LOAD_RENDER | FT_LOAD_TARGET_LCD_V);
		ets_printf("slot = %p\n", slot);
		if(error!=0) ets_printf("%s %d\n", "Error occured @FT_Load_Char! Error:", (int)error);
		//heap_caps_print_heap_info(MALLOC_CAP_8BIT);
		uint24_RGB* spriteBuf = (uint24_RGB*) malloc(slot->bitmap.rows * slot->bitmap.width);
		//heap_caps_print_heap_info(MALLOC_CAP_8BIT);
		FT_Int bmp_top = 240 - slot->bitmap_top;
		int sz = slot->bitmap.rows*slot->bitmap.width / 3;
		for(int p=0;p<sz;p++) {
			//ets_printf("spriteBuf = %d\n", p);
			spriteBuf[p].pixelB = slot->bitmap.buffer[p/(slot->bitmap.width)*slot->bitmap.width*3+(p%slot->bitmap.width)];
			spriteBuf[p].pixelG = slot->bitmap.buffer[p/(slot->bitmap.width)*slot->bitmap.width*3+(p%slot->bitmap.width)+slot->bitmap.width];
			spriteBuf[p].pixelR = slot->bitmap.buffer[p/(slot->bitmap.width)*slot->bitmap.width*3+(p%slot->bitmap.width)+slot->bitmap.width*2];
		}
		init_sprite(spriteBuf, slot->bitmap_left, bmp_top, slot->bitmap.width, slot->bitmap.rows/3, false, false, true);
		ets_printf("%d x %dpx\n", slot->bitmap.rows / 3, slot->bitmap.width);
		ets_printf("%d bytes\n", slot->bitmap.rows * slot->bitmap.width);

		offset.x += slot->advance.x;
		offset.y += slot->advance.y;
	}

	ets_printf("Rendering characters done!\n");
	FT_Done_Face (typeFace);
	FT_Done_FreeType(lib);
	ets_printf("%s\n", "Sending image data...");
	draw_all_sprites(spi);
	ets_printf("finished sending display data!\n");

}

// vim: foldmethod=marker
