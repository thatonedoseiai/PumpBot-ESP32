#include "ILIDriver.h"
#include "freetype2/ft2build.h"
#include <stdio.h>
#include <rom/ets_sys.h>
#include FT_FREETYPE_H
#include "oam.h"
#include "rotenc.h"
#include "stack_ddt.h"
#include "utf8.h"

#include "esp_littlefs.h"
#include "board_config.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);

const int fontSize = 20;
const int startX = 20;
const int startY = 20;
const uint24_RGB fillColor = {
	.pixelR = 0x10,
	.pixelG = 0,
	.pixelB = 0x30,
};

int inits(spi_device_handle_t* spi, rotary_encoder_info_t* info, FT_Library* lib, FT_Face* typeFace) {
	ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &buscfg, SPI_DMA_CH_AUTO));
	ESP_ERROR_CHECK(spi_bus_add_device(LCD_HOST, &devcfg, spi));
	ESP_ERROR_CHECK(gpio_install_isr_service(0));
	lcd_init(*spi);
	init_oam();
	// prg_init(&loaded_prg);

	esp_err_t ret = esp_vfs_littlefs_register(&conf);
	if (ret)
		goto done;

	ESP_ERROR_CHECK(rotary_encoder_init(info, PIN_NUM_ENC_A, PIN_NUM_ENC_B, PIN_NUM_ENC_BTN));
	ESP_ERROR_CHECK(rotary_encoder_enable_half_steps(info, false));
	ESP_ERROR_CHECK(rotary_encoder_flip_direction(info));

	gpio_config(&btn_conf);

    int error;
	FT_ERR_HANDLE(FT_Init_FreeType(lib), "FT_Init_Freetype");
	FT_ERR_HANDLE(FT_New_Face(*lib, "/mainfs/MeiryoUImin.ttf", 0, typeFace), "FT_New_Face");
	FT_ERR_HANDLE(FT_Select_Charmap(*typeFace, FT_ENCODING_UNICODE), "FT_Select_Charmap");

	// size_t total = 0, used = 0;
	// ret = esp_littlefs_info(conf.partition_label, &total, &used);
	// if (ret != ESP_OK) {
	// 	ets_printf("Failed to get LittleFS partition information (%d)\n", ret);
	// } else {
	// 	ets_printf("Partition size: total: %d, used: %d\n", total, used);
	// }
done:
	return ret;
}

int exampleCallback(rotary_encoder_state_t* state, void* args, char isNotEvent) {
    (void) args;

    if(isNotEvent) {
        ets_printf("Poll: position %d, direction %s\n", state->position,
               state->direction ? (state->direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE ? "CW" : "CCW") : "NOT_SET");
    } else {
        ets_printf("Event: position %d, direction %s\n", state->position,
                state->direction ? (state->direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE ? "CW" : "CCW") : "NOT_SET");
    }
    return 0;
}

int rotaryAction(QueueHandle_t event_queue, rotary_encoder_info_t* info, rotary_encoder_event_t* event, rotary_encoder_state_t* state, int(*callback)(rotary_encoder_state_t*, void*, char), void* args) {
    if(xQueueReceive(event_queue, event, 50/portTICK_PERIOD_MS) == pdTRUE) {
        return callback(&(event->state), args, 0);
    } else {
        ESP_ERROR_CHECK(rotary_encoder_get_state(info, state));
        return callback(state, args, 1);
    }
}

int draw_text(int startX, int startY, char* string, FT_Face typeFace, uint24_RGB** spriteBuf) {
    FT_Vector offset;
    FT_GlyphSlot slot;

	slot = typeFace->glyph;
	offset.x = startX << 6;
	offset.y = startY << 6;

    char* reader_head = string; // so that there is no modification
    int err;
    while (*reader_head != 0) {
		FT_Set_Transform(typeFace, NULL, &offset);
		err = FT_Load_Char(typeFace, decode_code_point(&reader_head), FT_LOAD_RENDER | FT_LOAD_TARGET_LCD_V);
        if(err)
            return err;
		(*spriteBuf) = (uint24_RGB*) malloc(slot->bitmap.rows * slot->bitmap.width);
		int sz = slot->bitmap.rows*slot->bitmap.width / 3;
		for(int p=0;p<sz;p++) {
			(*spriteBuf)[p].pixelB = slot->bitmap.buffer[p/(slot->bitmap.width)*slot->bitmap.width*3+(p%slot->bitmap.width)];
			(*spriteBuf)[p].pixelG = slot->bitmap.buffer[p/(slot->bitmap.width)*slot->bitmap.width*3+(p%slot->bitmap.width)+slot->bitmap.width];
			(*spriteBuf)[p].pixelR = slot->bitmap.buffer[p/(slot->bitmap.width)*slot->bitmap.width*3+(p%slot->bitmap.width)+slot->bitmap.width*2];
		}
		FT_Int bmp_top = 240 - slot->bitmap_top;
		init_sprite(*spriteBuf, slot->bitmap_left, bmp_top, slot->bitmap.width, slot->bitmap.rows/3, false, false, true);

		offset.x += slot->advance.x;
		offset.y += slot->advance.y;
	}

    return 0;
}

// static FT_ULong text[] = {0x547C, 0x55DA, 0x0000};//"嗚呼";
void app_main(void) {
	static FT_Library lib;
	static FT_Face typeFace; // = *(FT_Face*)malloc(sizeof(FT_Face));
	static FT_Error error;
	static rotary_encoder_info_t info = { 0 };
	// static PRG loaded_prg;
	static QueueHandle_t event_queue;
	static rotary_encoder_event_t event = { 0 };
	static rotary_encoder_state_t state = { 0 };

	spi_device_handle_t spi;

	// initializations
	esp_err_t ret = inits(&spi, &info, &lib, &typeFace);
    event_queue = rotary_encoder_create_queue(); 

	if(ret!=ESP_OK) {
		ets_printf("initializations failed!\n");
		return;
	}


	FILE *f = fopen("/mainfs/the_best_medicine_is", "r");
	if(f==NULL) {
		ets_printf("failed to open file!\n");
		return;
	}
	char line[64];
	fgets(line, sizeof(line), f);
	fclose(f);
	char *pos = strchr(line, '\n');
	if (pos) {
		*pos = '\0';
	}


	send_color(spi, fillColor);

	// ets_printf("sw0 level: %d\n", gpio_get_level(PIN_NUM_SW0));
	// ets_printf("sw1 level: %d\n", gpio_get_level(PIN_NUM_SW1));

	while(gpio_get_level(PIN_NUM_SW0)) {
		rotaryAction(event_queue, &info, &event, &state, exampleCallback, NULL);
	}


    uint24_RGB* spriteBuf;
	FT_ERR_HANDLE(FT_Set_Char_Size (typeFace, fontSize << 6, 0, 100, 0), "FT_Set_Char_Size"); // 0 = copy last value
    FT_ERR_HANDLE(draw_text(startX, startY, line, typeFace, &spriteBuf), "draw_sprite");

	esp_vfs_littlefs_unregister(conf.partition_label);
	ESP_ERROR_CHECK(rotary_encoder_uninit(&info));
	FT_Done_Face (typeFace);
	FT_Done_FreeType(lib);
	ets_printf("%s\n", "Rendering characters done! Sending image data...");
	draw_all_sprites(spi);
	ets_printf("finished sending display data!\n");
}

// vim: foldmethod=marker
