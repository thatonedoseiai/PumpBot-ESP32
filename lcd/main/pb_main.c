#include "ILIDriver.h"
#include "freetype2/ft2build.h"
#include <stdio.h>
#include <rom/ets_sys.h>
#include FT_FREETYPE_H
#include "oam.h"
#include "rotenc.h"
#include "menus.h"
#include "menu_data.h"
#include "pwm_fade.h"

#include "esp_wifi.h"
#include "nvs_flash.h"

#include "esp_littlefs.h"
#include "board_config.h"
#include "esp_event.h"

#include "lua_exports.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);

const uint24_RGB fillColor = {
	.pixelR = 0x10,
	.pixelG = 0,
	.pixelB = 0x30,
};
const uint24_RGB WHITE = {
    .pixelR = 0xff,
    .pixelG = 0xff,
    .pixelB = 0xff,
};

// OAM STUFF
extern uint8_t text_cache_size;
extern uint24_RGB* background_color;
//

static char connect_flag = 0;
spi_device_handle_t spi;
static char redraw_flag = 0;
static int nums[5] = {0,0,0,0,0};
static lua_State* L;
FT_Face typeFace; // because lua is required to use this, it must remain global
rotary_encoder_info_t* infop; // also because of lua

static void wifi_event_handler(void* arg, esp_event_base_t event_base,
                                    int32_t event_id, void* event_data)
{
    // if (event_id == WIFI_EVENT_AP_STACONNECTED) {
    //     wifi_event_ap_staconnected_t* event = (wifi_event_ap_staconnected_t*) event_data;
    //     // ets_printf("station %s join, AID=%d", MAC2STR(event->mac), event->aid);
    // } else if (event_id == WIFI_EVENT_AP_STADISCONNECTED) {
    //     wifi_event_ap_stadisconnected_t* event = (wifi_event_ap_stadisconnected_t*) event_data;
    //     // ets_printf("station %s leave, AID=%d", MAC2STR(event->mac), event->aid);
    // }
    connect_flag = 1;
}

int inits(spi_device_handle_t* spi, rotary_encoder_info_t* info, FT_Library* lib, FT_Face* typeFace) {
    esp_err_t k = nvs_flash_init();
    if (k == ESP_ERR_NVS_NO_FREE_PAGES || k == ESP_ERR_NVS_NEW_VERSION_FOUND) {
      ESP_ERROR_CHECK(nvs_flash_erase());
      k = nvs_flash_init();
    }
    ESP_ERROR_CHECK(k);

	ESP_ERROR_CHECK(spi_bus_initialize(LCD_HOST, &buscfg, SPI_DMA_CH_AUTO));
	ESP_ERROR_CHECK(spi_bus_add_device(LCD_HOST, &devcfg, spi));
	ESP_ERROR_CHECK(gpio_install_isr_service(0));
	lcd_init(*spi);
	init_oam();

	esp_err_t ret = esp_vfs_littlefs_register(&conf);
	if (ret)
		goto done;

	ESP_ERROR_CHECK(rotary_encoder_init(info, PIN_NUM_ENC_A, PIN_NUM_ENC_B, PIN_NUM_ENC_BTN));
	ESP_ERROR_CHECK(rotary_encoder_enable_half_steps(info, false));
	ESP_ERROR_CHECK(rotary_encoder_flip_direction(info));
    infop = info;
    wifi_init_config_t wificfg = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&wificfg));
    // if (strlen(EXAMPLE_ESP_WIFI_PASS) == 0) {
    //     wifi_config.ap.authmode = WIFI_AUTH_OPEN;
    // }
    ESP_ERROR_CHECK(esp_event_loop_create_default());
    ESP_ERROR_CHECK(esp_event_handler_instance_register(WIFI_EVENT,
                                                    ESP_EVENT_ANY_ID,
                                                    &wifi_event_handler,
                                                    NULL,
                                                    NULL));

    ESP_ERROR_CHECK(ledc_timer_config(&timer_config_0));
    ESP_ERROR_CHECK(ledc_channel_config(&channel_config));

	gpio_config(&btn_conf);

    int error;
	FT_ERR_HANDLE(FT_Init_FreeType(lib), "FT_Init_Freetype");
	FT_ERR_HANDLE(FT_New_Face(*lib, "/mainfs/MeiryoUImid.ttf", 0, typeFace), "FT_New_Face");
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
    // (void) args;
    static char numBuf[6];
    static int old_position = -1;
    static int newNums[5] = {0,0,0,0,0};
    // int err;

    FT_Face t_face = (FT_Face) args;

    if(isNotEvent) {
        ets_printf("Poll: position %d, direction %s\n", state->position,
                state->direction ? (state->direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE ? "CW" : "CCW") : "NOT_SET");
        if(old_position != state->position) {
            old_position = state->position;
            sprintf(numBuf, "%03d%%", old_position);
            draw_text(40, 176, numBuf, t_face, newNums, &WHITE, &fillColor);
            for(int i=0;i<5;++i) {
                if(nums[i] != 0 && nums[i] > 0)
                    delete_sprite(nums[i]);
                nums[i] = newNums[i];
            }
            redraw_flag = 1;
            return strlen(numBuf);
        }
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

void app_main(void) {
	static FT_Library lib;
	static FT_Error error;
	static rotary_encoder_info_t info = { 0 };
	static QueueHandle_t event_queue;
	// static rotary_encoder_event_t event = { 0 };
	// static rotary_encoder_state_t state = { 0 };
    static pwm_fade_info_t pfade_info_0;
    background_color = &fillColor;

	// initializations
	esp_err_t ret = inits(&spi, &info, &lib, &typeFace);
    event_queue = rotary_encoder_create_queue(); 
    L = lua_init();
    init_pwm_fade_info(&pfade_info_0, LEDC_CHANNEL_0);

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

    // pwm_setup_fade(&pfade_info_0, 0, 16300, 100);
    // for(int i=0;i<100;++i) {
    //     // ledc_set_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0, i*163);
    //     // ledc_update_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0);
    //     pwm_step_fade(&pfade_info_0);
    //     vTaskDelay(10);
    // }

    // pwm_setup_fade(&pfade_info_0, 16300, 1630, 90);
    // for(int i=100;i>9;--i) {
    //     // ledc_set_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0, i*163);
    //     // ledc_update_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0);
    //     pwm_step_fade(&pfade_info_0);
    //     vTaskDelay(10);
    // }

	send_color(spi, background_color);
    buffer_fillcolor(background_color);

	// ets_printf("sw0 level: %d\n", gpio_get_level(PIN_NUM_SW0));
	// ets_printf("sw1 level: %d\n", gpio_get_level(PIN_NUM_SW1));

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_AP));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_AP, (wifi_config_t*) &wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());

    ESP_ERROR_CHECK(esp_wifi_stop());

    // int len = strlen(line);
    // int spriteArray[len];
	// FT_ERR_HANDLE(FT_Set_Char_Size (typeFace, fontSize << 6, 0, 100, 0), "FT_Set_Char_Size"); // 0 = copy last value
    // FT_ERR_HANDLE(draw_text(startX, startY, line, typeFace, &spriteArray[0]), "draw_sprite");
    // center_sprite_group_x(spriteArray, len);
    // error = draw_menu_elements(&text_test[0], typeFace, 17); 
    error = draw_menu_elements(&menuhome[0], typeFace, 14); 
    draw_all_sprites(spi);
    delete_all_sprites();

    // error = draw_menu_elements(&menuabcde[0], typeFace, 4); 
    if (error)
        ets_printf("draw menu element\n");

    ets_printf("cache size: %d\n", text_cache_size);

	// draw_all_sprites(spi);

    (void) luaL_dofile(L, "/mainfs/test.lua");

    // connect_flag = 0;
    // int txtln = 0;
    // FT_Set_Char_Size (typeFace, 48 << 6, 0, 100, 0); // 0 = copy last value
	// while(gpio_get_level(PIN_NUM_SW0) && (connect_flag == 0)) {
		// txtln = rotaryAction(event_queue, &info, &event, &state, exampleCallback, (void*) typeFace);
    //     if(redraw_flag) {
    //         center_sprite_group_x(nums, txtln);
    //         draw_sprites(spi, nums, txtln);
    //         redraw_flag = 0;
    //     }
	// }

    // error = draw_menu_elements(&text_test[0], typeFace, 17); 
    // if (error)
    //     ets_printf("draw menu element\n");
    // buffer_all_sprites();
    // delete_all_sprites();

    // for(int i=0;i<320;i+=8) {
    //     scroll_buffer(spi, i, i==0);
    //     vTaskDelay(8 / portTICK_PERIOD_MS);
    // }
    // scroll_buffer(spi, 0, true);

    free(framebuf);
	ESP_ERROR_CHECK(rotary_encoder_uninit(&info));
    ESP_ERROR_CHECK(esp_wifi_deinit());
    esp_vfs_littlefs_unregister(conf.partition_label);
	FT_Done_Face (typeFace);
	FT_Done_FreeType(lib);
}

// vim: foldmethod=marker
