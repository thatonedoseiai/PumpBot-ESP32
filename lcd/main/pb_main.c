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

#include "esp_wifi.h"
#include "nvs_flash.h"

#include "esp_littlefs.h"
#include "board_config.h"
#include "esp_event.h"

#include "http.h"
#include "file_server.h"
#include "esp_timer.h"
#include "rgb_fade.h"
#include "pwm_output.h"
#include "system_status.h"
#include "socket.h"
#include <pthread.h>

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

uint8_t system_flags = 0;
spi_device_handle_t spi;
lua_State* L;
FT_Face typeFace; // because lua is required to use this, it must remain global
rotary_encoder_info_t* infop; // also because of lua
QueueHandle_t* button_events = NULL; // also because of lua
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
        system_flags &= ~(FLAG_WIFI_TIMED_OUT | FLAG_WIFI_CONNECTED);
        wifi_restart_counter = 0;
        esp_wifi_connect();
        ets_printf("starting connection...\n");
    } else if(event_base == WIFI_EVENT && event_id == WIFI_EVENT_STA_DISCONNECTED) {
        ets_printf("retrying connection...\n");
        wifi_restart_counter++;
        if(wifi_restart_counter < 10) {
            system_flags &= ~FLAG_WIFI_CONNECTED;
            esp_wifi_connect();
        } else {
            system_flags |= FLAG_WIFI_TIMED_OUT;
            wifi_restart_counter = 0;
            ets_printf("failed to connect!\n");
        }
    } else if(event_base == IP_EVENT && event_id == IP_EVENT_STA_GOT_IP) {
        ip_event_got_ip_t* event = (ip_event_got_ip_t*) event_data;
        ets_printf("got ip: %d.%d.%d.%d\n", IP2STR(&event->ip_info.ip));
        system_flags |= FLAG_WIFI_CONNECTED;
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
    ESP_ERROR_CHECK(ledc_fade_func_install(ESP_INTR_FLAG_INTRDISABLED));

    (*btn_events) = button_init(PIN_BIT(PIN_NUM_SW0) | PIN_BIT(PIN_NUM_SW1) | PIN_BIT(PIN_NUM_ENC_BTN));
    button_events = btn_events;

    int error;
	FT_ERR_HANDLE(FT_Init_FreeType(lib), "FT_Init_Freetype");
	FT_ERR_HANDLE(FT_New_Face(*lib, "/mainfs/PB-Sans-1.1.1_01a.ttf", 0, typeFace), "FT_New_Face");
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

void* connect_server_thread_start(void* arg) {
    pthread_detach(pthread_self());

    if(connect_to_server(*(uint32_t*) &settings.server_ip, settings.server_port)) {
        system_flags &= ~FLAG_SERVER_CONNECTED;
    } else {
        system_flags |= FLAG_SERVER_CONNECTED;
    }
    ets_printf("%d\n", system_flags & FLAG_SERVER_CONNECTED);

    pthread_exit(NULL);
}

void app_main(void) {
	static FT_Library lib;
	static FT_Error error;
	static rotary_encoder_info_t info = { 0 };
	static QueueHandle_t btn_events;
    background_color = (uint24_RGB*) &fillColor;
    foreground_color = (uint24_RGB*) &WHITE;

	// initializations
	esp_err_t ret = inits(&spi, &info, &btn_events, &lib, &typeFace);
    L = lua_init();

	if(ret!=ESP_OK) {
		ets_printf("initializations failed!\n");
		return;
	}

    ledc_set_duty(LEDC_LOW_SPEED_MODE, 7, 0x3fff);
	ledc_update_duty(LEDC_LOW_SPEED_MODE, 7);

    ESP_ERROR_CHECK(esp_netif_init());
    esp_netif_create_default_wifi_sta();
    esp_netif_create_default_wifi_ap();

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_APSTA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_AP, (wifi_config_t*) &ap_wifi_config));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());

    rgb_init();
    init_pb_output_info();

    char setup_flag = 0;
    if(read_from_file(&settings)) {
        set_settings_to_default(&settings);
        setup_flag = 1;
        FILE* colfile = fopen("/mainfs/colors", "r");
        if(colfile) {
            fscanf(colfile, "#%2hhx%2hhx%2hhx\n#%2hhx%2hhx%2hhx\n%x,%hx,%x", &settings.RGB_colour.pixelR, &settings.RGB_colour.pixelB, &settings.RGB_colour.pixelG, &settings.RGB_colour_2.pixelR, &settings.RGB_colour_2.pixelG, &settings.RGB_colour_2.pixelB, &settings.RGB_mode, &settings.RGB_brightness, &settings.RGB_speed);
            fclose(colfile);
        }
    } else {
        strncpy((char*)sta_wifi_config.sta.ssid, (char*)&settings.wifi_name[0], 32);
        strncpy((char*)sta_wifi_config.sta.password, (char*)&settings.wifi_pass[0], 64);
        ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, (wifi_config_t*) &sta_wifi_config));
        settings.server_port = 7867;
        esp_wifi_connect();
        while(!(system_flags & (FLAG_WIFI_CONNECTED | FLAG_WIFI_TIMED_OUT)));
        if(system_flags & FLAG_WIFI_CONNECTED) {
            pthread_t ptid;
            pthread_create(&ptid, NULL, &connect_server_thread_start, NULL);
        } else {
            ets_printf("failed to connect to wifi!\n");
        }
    }
    rgb_update();
    assign_theme_from_settings();

    button_event_t event;
    if(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 3) {
        char* buf = calloc(256, sizeof(char));
        if(!read_default_app(buf, 256))
            (void) luaL_dofile(L, buf);
        free(buf);
    } else if(event.pin == 18) {
        // encoder switch

        int old_rgb = settings.RGB_mode;
        uint24_RGB old_rgbcol = settings.RGB_colour;
        settings.RGB_mode = RGB_MODE_SOLID;
        settings.RGB_colour.pixelR = 255;
        settings.RGB_colour.pixelG = 0;
        settings.RGB_colour.pixelB = 0;
        for(int i=0;i<4;++i)
            output_set_value(i, 0);
        rgb_update();
        send_color(spi, &settings.RGB_colour);
        while(!(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18 && event.event == BUTTON_DOWN));
        settings.RGB_colour.pixelR = 0;
        settings.RGB_colour.pixelG = 255;
        for(int i=0;i<4;++i)
            output_set_value(i, 4096);
        rgb_update();
        send_color(spi, &settings.RGB_colour);
        while(!(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18 && event.event == BUTTON_DOWN));
        settings.RGB_colour.pixelG = 0;
        settings.RGB_colour.pixelB = 255;
        for(int i=0;i<4;++i)
            output_set_value(i, 8192);
        rgb_update();
        send_color(spi, &settings.RGB_colour);
        while(!(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18 && event.event == BUTTON_DOWN));
        settings.RGB_colour.pixelB = 0;
        for(int i=0;i<4;++i)
            output_set_value(i, 12288);
        rgb_update();
        send_color(spi, &settings.RGB_colour);
        while(!(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18 && event.event == BUTTON_DOWN));
        for(int i=0;i<4;++i)
            output_set_value(i, 16383);
        while(!(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18 && event.event == BUTTON_DOWN));
        settings.RGB_mode = old_rgb;
        settings.RGB_colour = old_rgbcol;
        for(int i=0;i<4;++i)
            output_set_value(i, 0);
        rgb_update();
        while(!(xQueueReceive(*button_events, &event, 10/portTICK_PERIOD_MS) == pdTRUE && event.pin == 18 && event.event == BUTTON_UP));
    }

    if(setup_flag) {
        (void) start_menu_tree(0, false);
    }
    rgb_update();
    assign_theme_from_settings();

    while(true) {
        send_color(spi, background_color);
        error = draw_menu_elements(&menuhome[0], typeFace, 14); 
        draw_all_sprites(spi);
        delete_all_sprites();
        if (error)
            ets_printf("draw menu element\n");
        (void) luaL_dofile(L, "/mainfs/test.lua");
        ets_printf("%s\n", lua_tostring(L,-1));
        delete_all_sprites();
        (void) start_menu_tree(11, true);
        flush_text_cache();
    }

    done_pb_output_info();
    ESP_ERROR_CHECK(esp_wifi_stop());
    free(framebuf);
	ESP_ERROR_CHECK(rotary_encoder_uninit(&info));
    ESP_ERROR_CHECK(esp_wifi_deinit());
    esp_vfs_littlefs_unregister(conf.partition_label);
	FT_Done_Face (typeFace);
	FT_Done_FreeType(lib);
}

// vim: foldmethod=marker
