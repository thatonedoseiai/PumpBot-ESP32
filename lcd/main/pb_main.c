#include "ILIDriver.h"
#include "settings.h"
#include "button.h"
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

#include "http.h"
#include "file_server.h"
#include "esp_timer.h"
#include "rgb_fade.h"

// #include "esp_http_client.h"
// #include "esp_crt_bundle.h"
// #include "esp_tls.h"

#include "lua_exports.h"

#define FT_ERR_HANDLE(code, loc) error = code; if(error) ets_printf("Error occured at %s! Error: %d\n", loc, (int) error);

const uint24_RGB fillColor = {
	.pixelR = 0x10,
	.pixelG = 0,
	.pixelB = 0x30,
};

// OAM STUFF
extern uint8_t text_cache_size;
extern uint24_RGB* background_color;
extern uint24_RGB* foreground_color;
extern const uint24_RGB WHITE;
//

char connect_flag = 0;
spi_device_handle_t spi;
// static char redraw_flag = 0;
// static int nums[5] = {0,0,0,0,0};
static lua_State* L;
FT_Face typeFace; // because lua is required to use this, it must remain global
rotary_encoder_info_t* infop; // also because of lua
QueueHandle_t* button_events = NULL; // also because of lua
pwm_fade_info_t pfade_channels[8]; // this one too
SETTINGS_t settings;

unsigned char wifi_restart_counter = 0;
static void wifi_event_handler(void* arg, esp_event_base_t event_base,
                                    int32_t event_id, void* event_data)
{
    // if (event_id == WIFI_EVENT_AP_STACONNECTED) {
    //     wifi_event_ap_staconnected_t* event = (wifi_event_ap_staconnected_t*) event_data;
        // ets_printf("station %s join, AID=%d", MAC2STR(event->mac), event->aid);
    // } else if (event_id == WIFI_EVENT_AP_STADISCONNECTED) {
    //     wifi_event_ap_stadisconnected_t* event = (wifi_event_ap_stadisconnected_t*) event_data;
        // ets_printf("station %s leave, AID=%d", MAC2STR(event->mac), event->aid);
    // }
    if(event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_CONNECTED) {
        wifi_event_sta_connected_t* event = (wifi_event_sta_connected_t*) event_data;
        ets_printf("connected to station \"%s\"!\n", event->ssid);
        wifi_restart_counter = 0;
    } else if(event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_START) {
        esp_wifi_connect();
        ets_printf("starting connection...\n");
    } else if(event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED) {
        ets_printf("retrying connection...\n");
        wifi_restart_counter++;
        if(wifi_restart_counter < 10) {
            esp_wifi_connect();
        } else {
            connect_flag = 1;
        }
    } else if(event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP) {
        ip_event_got_ip_t* event = (ip_event_got_ip_t*) event_data;
        ets_printf("got ip:\n", IP2STR(&event->ip_info.ip));
        connect_flag = 1;
    }
}

int inits(spi_device_handle_t* spi, rotary_encoder_info_t* info, QueueHandle_t* btn_events, FT_Library* lib, FT_Face* typeFace) {
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
    QueueHandle_t queue = rotary_encoder_create_queue();
    ESP_ERROR_CHECK(rotary_encoder_set_queue(info, queue));
    infop = info;
    wifi_init_config_t wificfg = WIFI_INIT_CONFIG_DEFAULT();
    ESP_ERROR_CHECK(esp_wifi_init(&wificfg));
    
    // if (strlen(EXAMPLE_ESP_WIFI_PASS) == 0) {
    //     wifi_config.ap.authmode = WIFI_AUTH_OPEN;
    // }
    ESP_ERROR_CHECK(esp_event_loop_create_default());
    ESP_ERROR_CHECK(esp_event_handler_instance_register(ESP_EVENT_ANY_BASE,
                                                    ESP_EVENT_ANY_ID,
                                                    &wifi_event_handler,
                                                    NULL,
                                                    NULL));

    ESP_ERROR_CHECK(ledc_timer_config(&timer_config_0));
    for(int i=0;i<LEDC_CHANNEL_MAX;++i) {
        channel_config.gpio_num = pwm_gpio_nums[i];
        channel_config.channel = i;
        ESP_ERROR_CHECK(ledc_channel_config(&channel_config));
    }

    (*btn_events) = button_init(PIN_BIT(PIN_NUM_SW0) | PIN_BIT(PIN_NUM_SW1) | PIN_BIT(PIN_NUM_ENC_BTN));
    button_events = btn_events;
	// gpio_config(&btn_conf);

    int error;
	FT_ERR_HANDLE(FT_Init_FreeType(lib), "FT_Init_Freetype");
	FT_ERR_HANDLE(FT_New_Face(*lib, "/mainfs/MeiryoUImid.ttf", 0, typeFace), "FT_New_Face");
	FT_ERR_HANDLE(FT_Select_Charmap(*typeFace, FT_ENCODING_UNICODE), "FT_Select_Charmap");

	size_t total = 0, used = 0;
	ret = esp_littlefs_info(conf.partition_label, &total, &used);
	if (ret != ESP_OK) {
		ets_printf("Failed to get LittleFS partition information (%d)\n", ret);
	} else {
		ets_printf("Partition size: total: %d, used: %d\n", total, used);
	}
done:
	return ret;
}

// int exampleCallback(rotary_encoder_state_t* state, void* args, char isNotEvent) {
//     // (void) args;
//     static char numBuf[6];
//     static int old_position = -1;
//     static int newNums[5] = {0,0,0,0,0};
//     // int err;

//     FT_Face t_face = (FT_Face) args;

//     if(isNotEvent) {
//         ets_printf("Poll: position %d, direction %s\n", state->position,
//                 state->direction ? (state->direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE ? "CW" : "CCW") : "NOT_SET");
//         if(old_position != state->position) {
//             old_position = state->position;
//             sprintf(numBuf, "%03d%%", old_position);
//             draw_text(40, 176, numBuf, t_face, newNums, &WHITE, &fillColor);
//             for(int i=0;i<5;++i) {
//                 if(nums[i] != 0 && nums[i] > 0)
//                     delete_sprite(nums[i]);
//                 nums[i] = newNums[i];
//             }
//             redraw_flag = 1;
//             return strlen(numBuf);
//         }
//     } else {
//         ets_printf("Event: position %d, direction %s\n", state->position,
//                 state->direction ? (state->direction == ROTARY_ENCODER_DIRECTION_CLOCKWISE ? "CW" : "CCW") : "NOT_SET");
//     }
//     return 0;
// }

// int rotaryAction(QueueHandle_t event_queue, rotary_encoder_info_t* info, rotary_encoder_event_t* event, rotary_encoder_state_t* state, int(*callback)(rotary_encoder_state_t*, void*, char), void* args) {
//     if(xQueueReceive(event_queue, event, 50/portTICK_PERIOD_MS) == pdTRUE) {
//         return callback(&(event->state), args, 0);
//     } else {
//         ESP_ERROR_CHECK(rotary_encoder_get_state(info, state));
//         return callback(state, args, 1);
//     }
// }

void app_main(void) {
	static FT_Library lib;
	static FT_Error error;
	static rotary_encoder_info_t info = { 0 };
	static QueueHandle_t btn_events;
	// static rotary_encoder_event_t event = { 0 };
	// static rotary_encoder_state_t state = { 0 };
    background_color = (uint24_RGB*) &fillColor;
    foreground_color = (uint24_RGB*) &WHITE;

	// initializations
	esp_err_t ret = inits(&spi, &info, &btn_events, &lib, &typeFace);
    // event_queue = rotary_encoder_create_queue(); 
    L = lua_init();
    for(int i=0;i<LEDC_CHANNEL_MAX;++i) {
        init_pwm_fade_info(&pfade_channels[i], i);
    }

	if(ret!=ESP_OK) {
		ets_printf("initializations failed!\n");
		return;
	}


    // pwm_setup_fade(&pfade_channels[5], 0, 16300, 100);
    // for(int i=0;i<100;++i) {
    //     // ledc_set_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0, i*163);
    //     // ledc_update_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0);
    //     pwm_step_fade(&pfade_channels[5]);
    //     vTaskDelay(10);
    // }

    // pwm_setup_fade(&pfade_channels[5], 16300, 1630, 90);
    // for(int i=100;i>9;--i) {
    //     // ledc_set_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0, i*163);
    //     // ledc_update_duty(LEDC_LOW_SPEED_MODE, LEDC_CHANNEL_0);
    //     pwm_step_fade(&pfade_channels[5]);
    //     vTaskDelay(10);
    // }
    ledc_set_duty(LEDC_LOW_SPEED_MODE, pfade_channels[7].channel, 0x3fff);
	ledc_update_duty(LEDC_LOW_SPEED_MODE, pfade_channels[7].channel);

	// send_color(spi, background_color);
    // buffer_fillcolor(background_color);

    ESP_ERROR_CHECK(esp_netif_init());
    esp_netif_create_default_wifi_sta();
    esp_netif_create_default_wifi_ap();

	// ets_printf("sw0 level: %d\n", gpio_get_level(PIN_NUM_SW0));
	// ets_printf("sw1 level: %d\n", gpio_get_level(PIN_NUM_SW1));

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_APSTA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_AP, (wifi_config_t*) &ap_wifi_config));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());

//     ESP_ERROR_CHECK(esp_wifi_scan_start(NULL, true));
    
//     ESP_ERROR_CHECK(esp_wifi_scan_get_ap_records(&aprecnum, ap_info));
//     ESP_ERROR_CHECK(esp_wifi_scan_get_ap_num(&ap_count));

//     for (int i = 0; i < aprecnum; i++) {
//         ets_printf("SSID \t\t%s\n", ap_info[i].ssid);
//         ets_printf("RSSI \t\t%d\n", ap_info[i].rssi);
//         // print_auth_mode(ap_info[i].authmode);
//         // if (ap_info[i].authmode != WIFI_AUTH_WEP) {
//         //     print_cipher_type(ap_info[i].pairwise_cipher, ap_info[i].group_cipher);
//         // }
//         ets_printf("Channel \t\t%d\n", ap_info[i].primary);
//     }

//     connect_flag = 0;
//     ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    // uint16_t aprecnum = 10;
    // wifi_ap_record_t ap_info[10];
    // uint16_t ap_count = 0;
    // memset(ap_info, 0, sizeof(ap_info));
    // connect_flag = 0;

    // connect_flag = 0;
    // ESP_ERROR_CHECK(example_start_file_server("/mainfs"));
    // button_event_t btnevent;
    // while(!connect_flag);
    // stop_file_server();

//     char wifiname[] = "hidden";
//     char pskey[] = "";
//     strcpy((char*) wifi_config.sta.ssid, wifiname);
//     strcpy((char*) wifi_config.sta.password, pskey);
//     ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &wifi_config));
//     ESP_ERROR_CHECK(esp_wifi_connect());

//     ets_printf("waiting for connection...\n");
//     while(!connect_flag);

//     ets_printf("connected!\n");
//     esp_err_t http_err;
//     do {
//         http_err = http_wget("https://raw.githubusercontent.com/Tortus-exe/APL-2021-advent-of-code/main/script.apl", "/mainfs/script.apl");
//     } while(http_err != ESP_OK);


// 	FILE *f = fopen("/mainfs/script.apl", "r");
// 	if(f==NULL) {
// 		ets_printf("failed to open file!\n");
// 		return;
// 	}
// 	char* line = malloc(128);
//     ets_printf("contents:\n");
//     while( fgets(line, 128, f) != NULL ) {
//         ets_printf(line);
//     }
// 	fclose(f);

    // int len = strlen(line);
    // int spriteArray[len];
	// FT_ERR_HANDLE(FT_Set_Char_Size (typeFace, fontSize << 6, 0, 100, 0), "FT_Set_Char_Size"); // 0 = copy last value
    // FT_ERR_HANDLE(draw_text(startX, startY, line, typeFace, &spriteArray[0]), "draw_sprite");
    // center_sprite_group_x(spriteArray, len);
    // error = draw_menu_elements(&text_test[0], typeFace, 17); 
    // error = draw_menu_elements(&menusetup0[0], typeFace, 8);
    // draw_all_sprites(spi);
    // delete_all_sprites();

    rgb_init();

    // settings.RGB_colour.pixelR = 0;
    // settings.RGB_colour.pixelB = 255;
    // update_rgb_mode(true);
    // vTaskDelay(1000);

    // settings.RGB_colour.pixelR = 255;
    // settings.RGB_colour.pixelB = 255;
    // update_rgb_mode(true);

    // curcol[0] = settings.RGB_colour.pixelR << 6;
    // curcol[1] = settings.RGB_colour.pixelG << 6;
    // curcol[2] = settings.RGB_colour.pixelB << 6;
    
    // const esp_timer_create_args_t periodic_timer_args = {
    //     // .callback = &rainbow_rgb_callback,
    //     .callback = &fade_rgb_callback
    // };

    // esp_timer_handle_t periodic_timer;
    // ESP_ERROR_CHECK(esp_timer_create(&periodic_timer_args, &periodic_timer));
    // ESP_ERROR_CHECK(esp_timer_start_periodic(periodic_timer, 100000));



    // ESP_ERROR_CHECK(esp_wifi_start());
    // (void) start_menu_tree(5);

    // ESP_ERROR_CHECK(esp_timer_init());
    char setup_flag = 0;
    if(read_from_file(&settings)) {
        settings.disp_brightness = 255;
        settings.disp_theme = 0;
        setup_flag = 1;
        memset(settings.pwm_min_limit, 0, 4*sizeof(uint16_t));
        for(int i=0;i<4;++i)
            settings.pwm_max_limit[i] = 0x3fff;
        settings.RGB_brightness = 0x3fff;
        settings.RGB_speed = 32;
    }
    rgb_update();
    assign_theme_from_settings();
    // if(setup_flag) {
    //     (void) start_menu_tree(0);
    //     // write_to_file(&settings);
    // }
    (void) start_menu_tree(11, true);
    // (void) start_menu_tree(14, true);
    // ESP_ERROR_CHECK(esp_timer_deinit());
    // ets_printf("%s\n", &settings.wifi_name);




    send_color(spi, background_color);
    error = draw_menu_elements(&menuhome[0], typeFace, 14); 
    draw_all_sprites(spi);
    delete_all_sprites();
    if (error)
        ets_printf("draw menu element\n");
    (void) luaL_dofile(L, "/mainfs/test.lua");

    // ets_printf("cache size: %d\n", text_cache_size);

	// draw_all_sprites(spi);

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

    ESP_ERROR_CHECK(esp_wifi_stop());
    free(framebuf);
	ESP_ERROR_CHECK(rotary_encoder_uninit(&info));
    ESP_ERROR_CHECK(esp_wifi_deinit());
    esp_vfs_littlefs_unregister(conf.partition_label);
	FT_Done_Face (typeFace);
	FT_Done_FreeType(lib);
}

// vim: foldmethod=marker
